#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod client_data;

use crypto::md5::Md5;
use crypto::digest::Digest;

use std::{
    collections::{HashMap, HashSet},
    default,
    time::Duration,
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, TypedHeader,
    },
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Router,
};
use futures::{
    stream::{SplitSink, StreamExt},
    SinkExt,
};
use std::{net::SocketAddr, path::PathBuf};
use tauri::{App, AppHandle, Manager};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    time::sleep,
};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::client_data::{ClientFunc, AllClientsData};
use crate::client_data::{ClientData, StartApp};

//  对共享数据（ClientMap）的所有的操作
enum DataOperation {
    Send {
        key: usize,
        msg: String,
    },
    Get {
        resp: Responder<Option<HashSet<String>>>,
    },
    Add {
        key: usize,
        client: SplitSink<WebSocket, Message>,
    },
    Remove {
        key: usize,
    },
    AddClientInfo {
        key: usize,
        info: String,
    },
    // 客户端（一般是移动端），请求所有客户端信息的时候，直接返回给它
    RetuenAllClientInfo {
        key: usize,
        data_hash: String,
    },
}

// 对客户端操作的响应
type Responder<T> = oneshot::Sender<T>;

// 全局唯一的UUID (也可以放到 redis 中)
static NEXT_USERID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

type ClientMap = HashMap<usize, SplitSink<WebSocket, Message>>;

#[derive(Clone, serde::Serialize)]
struct Payload {
  args: Vec<String>,
  cwd: String,
}

#[tokio::main]
async fn main() {
    // 创建一个访问和操作共享数据的通道
    let (tx, rx) = mpsc::channel(32);
    let tx2 = tx.clone();
    let tx3 = tx.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);

            app.emit_all("single-instance", Payload { args: argv, cwd }).unwrap();
        }))
        .setup(|app| {
            let app_launcher = app.app_handle();
            tokio::spawn(handle_data_channel(rx));
            // tokio::spawn(printclients(tx2));
            tokio::spawn(start_axum(tx, app_launcher));
            Ok(())
        })
        .manage(tx3)
        .invoke_handler(tauri::generate_handler![greet, start_app, update_client, get_local_ip])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[tauri::command]
fn get_local_ip() -> String {
    let mut ip = "".to_string();
    if let Some(ip_) = local_ipaddress::get() {
        ip = ip_;
    }
    ip 
}

#[derive(serde::Serialize)]
struct CustomResponse {
    message: String,
    other_val: usize,
}

#[tauri::command]
async fn start_app(
    id: i32,
    start: bool,
    app: String,
    tx: tauri::State<'_, Sender<DataOperation>>,
) -> Result<CustomResponse, String> {
    start_app_common(id, start, app, tx.inner().clone()).await
}

async fn start_app_common(
    id: i32,
    start: bool,
    app: String,
    tx: Sender<DataOperation>,
) -> Result<CustomResponse, String> {
    let client_func: ClientFunc;
    if start {
        client_func = ClientFunc {
            func_name: "start_app".to_string(),
            data: app,
        };
    } else {
        client_func = ClientFunc {
            func_name: "stop_app".to_string(),
            data: app,
        };
    }
    let client_func = serde_json::to_string(&client_func).unwrap();

    let op = DataOperation::Send {
        key: id as usize,
        msg: client_func,
    };
    if tx.send(op).await.is_err() {
        println!("get data failed!");
        Err("No result".into())
    } else {
        Ok(CustomResponse {
            message: "".to_string(),
            other_val: 0,
        })
    }
}

// 去客户端请求需要更新的数据
#[tauri::command]
async fn update_client(
    id: i32,
    clientid: i32,
    tx: tauri::State<'_, Sender<DataOperation>>,
) -> Result<String, String> {
    update_client_common(id, clientid, tx.inner().clone()).await
}

// clientid (-1 表示是 服务器请求的，其他编号指的是，移动端的请求，谁请求，就转发给谁)
async fn update_client_common(
    id: i32,
    clientid: i32,
    tx: Sender<DataOperation>,
) -> Result<String, String> {
    println!("call update_client_common 1, clientid {}", clientid);
    let client_func = ClientFunc {
        func_name: "update_client".to_string(),
        data: clientid.to_string(),
    };
    let client_func = serde_json::to_string(&client_func).unwrap();

    let op = DataOperation::Send {
        key: id as usize,
        msg: client_func,
    };
    println!("call update_client_common 2, clientid {}", clientid);
    if tx.send(op).await.is_err() {
        println!("get data failed!");
        Err("".into())
    } else {
        Ok("".to_string())
    }
}

// 处理数据
async fn handle_data_channel(mut rx: Receiver<DataOperation>) {
    let mut data: ClientMap = HashMap::default();
    let mut client_info: HashMap<usize, String> = HashMap::default();
    println!("into data handle");
    loop {
        if let Some(op) = rx.recv().await {
            match op {
                DataOperation::Send { key, msg } => {
                    // let sender = data.get_mut(&key);
                    // sender.unwrap().send(  Message::Text(String::from("Username already taken."))  );
                    // println!("send data to client! {}", &msg);
                    if let Some(sender) = data.get_mut(&key) {
                        if sender.send(Message::Text(msg)).await.is_err() {
                            println!("send data failed!");
                            // return;
                        }
                    }
                }
                DataOperation::Add { key, client } => {
                    data.insert(key, client);
                }
                DataOperation::Get { resp } => {
                    let mut res: HashSet<String> = HashSet::default();
                    for i in data.iter() {
                        res.insert(i.0.to_string());
                    }
                    let _ = resp.send(Some(res));
                }
                DataOperation::Remove { key } => {
                    data.remove(&key);
                    client_info.remove(&key);
                }

                DataOperation::AddClientInfo { key, info } => {
                    client_info.insert(key, info);
                }

                DataOperation::RetuenAllClientInfo { key , data_hash} => {
                     
                    let mut res: HashSet<String> = HashSet::default();
                    for i in client_info.iter() {
                        res.insert(i.1.to_string());
                    }
                    
                    if let Ok(res_string) = serde_json::to_string_pretty(&res) {
                        
                        // 检查数据是否有变化

                        // 对data进行取MD5
                        let mut hasher = Md5::new();
                        hasher.input_str(&res_string.to_owned());
                        let current_data_hash = hasher.result_str();

                        // 如果数据有变化
                        if current_data_hash != data_hash {
                            println!("data has changed {}", hasher.result_str());

                            if let Some(sender) = data.get_mut(&key) {
                                let all_clients_data = AllClientsData{
                                    data_hash: current_data_hash,
                                    all_clients: res };

                            if let Ok(res_string) = serde_json::to_string_pretty(&all_clients_data) {

                                let func = ClientFunc {
                                    func_name: "on_get_all_clients".to_string(),
                                    data: res_string,
                                };
                                let fun_str = serde_json::to_string_pretty(&func).unwrap();
                                
                                // println!("RetuenAllClientInfo send data to client = {}", &fun_str);
                                if sender.send(Message::Text(fun_str)).await.is_err() {
                                    println!("send data failed!");
                                }

                            }
                            }

                        } else {
                            println!("data has not changed");
                        }

                    } else {
                        println!("serde_json failed!");
                    }
                }
            }
        } else {
            println!("recv failed!");
        }
    }
}

async fn printclients(tx: Sender<DataOperation>) {
    println!("into get");
    loop {
        sleep(Duration::from_millis(5000)).await;
        // 临时接受管道
        let (resp_tx, resp_rx) = oneshot::channel();

        let op = DataOperation::Get { resp: resp_tx };

        println!("start get data");
        if tx.send(op).await.is_err() {
            println!("get data failed!");
            // return;
        }
        let res = resp_rx.await;

        // println!("data = {:?}", res);

        for i in res.unwrap().unwrap().iter() {
            let id = i.parse::<usize>().unwrap();
            println!("client id = {}", id);
            let op = DataOperation::Send {
                key: id,
                msg: String::from("hello"),
            };
            if tx.send(op).await.is_err() {
                println!("send data failed!");
                // return;
            }
        }
    }
}
// init a background process on the command, and emit periodic events only to the window that used the command
async fn start_axum(tx: Sender<DataOperation>, app_launcher: AppHandle) {
    sleep(Duration::from_millis(5000)).await;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    // build our application with some routes
    let app = Router::with_state((tx, app_launcher))
        // 添加静态文件路径为 webdist
        .nest(
            "",
            get_service(ServeDir::new("./webdist")).handle_error(
                |error: std::io::Error| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    )
                },
            ),
        )
        .fallback_service(
            get_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
                .handle_error(|error: std::io::Error| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    )
                }),
        )
        // routes are matched from bottom to top, so we have to put `nest` at the
        // top since it matches all routes
        .route("/ws", get(ws_handler))
        .route("/home", get(http_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State((tx, app_handle)): State<(Sender<DataOperation>, AppHandle)>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }
    ws.on_upgrade(|socket| handle_socket(socket, tx, app_handle))
}

//  接收来自客户端的信息
async fn handle_socket(socket: WebSocket, tx: Sender<DataOperation>, app_handle: AppHandle) {
    let user_id = NEXT_USERID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // By splitting we can send and receive at the same time.
    let (sender, mut receiver) = socket.split();

    println!("user_id = {}", user_id);

    let op = DataOperation::Add {
        key: user_id,
        client: sender,
    };

    println!("start add data");
    if tx.send(op).await.is_err() {
        println!("add data failed!");
    }

    loop {
        if let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(t) => {
                        println!("on get client str: {:?}", t);

                        // 接收到 Json 字符串
                        if let Ok(client_func) = serde_json::from_str::<ClientFunc>(&t) {
                            // 匹配函数名称
                            match &client_func.func_name as &str {
                                // 获取客户端本地数据
                                "on_get_client_data" => {
                                    // 解成Json , 设置 id 再转回 string
                                    let mut client_data: ClientData =
                                        serde_json::from_str(&client_func.data).unwrap();
                                    client_data.id = user_id as i32;
                                    // println!("user_id = {}", client_data.ip);
                                    // let client_func: ClientFunc = serde_json::from_str(&t).unwrap();
                                    let client_str =
                                        serde_json::to_string_pretty(&client_data).unwrap();
                                    app_handle
                                        .emit_all("on_get_client_data", &client_str)
                                        .unwrap();

                                    // 保存客户端信息
                                    let op = DataOperation::AddClientInfo {
                                        key: user_id,
                                        info: client_str,
                                    };
                                    let _ = tx.send(op).await;
                                }
                                // 当收到来自客户端的更新信息
                                // 同时转发到移动端
                                "on_update_client" => {
                                    // 解成Json , 获取 id 再 决定需要转发给谁
                                    let update_client_data: client_data::ClientUpdateData =
                                        serde_json::from_str(&client_func.data).unwrap();

                                    let caller_id = update_client_data.caller_id;
                                    if caller_id < 0 {
                                        app_handle
                                            .emit_all("on_update_client", &client_func.data)
                                            .unwrap();
                                    } else {
                                        // 发送到请求客户端
                                        let op = DataOperation::Send { key: caller_id as usize, msg: t };
                                        let _ = tx.send(op).await;
                                    }
                                }
                                // 移动端定时获取所有的客户端的基本信息
                                "get_all_clients" => {
                                    let op = DataOperation::RetuenAllClientInfo { key: user_id , data_hash:client_func.data };
                                    let _ = tx.send(op).await;
                                }
                                // 移动端点击启动app
                                "start_app" => {
                                    let start_app_data: StartApp =
                                        serde_json::from_str(&client_func.data).unwrap();
                                    let _ = start_app_common(
                                        start_app_data.id,
                                        start_app_data.start,
                                        start_app_data.app,
                                        tx.clone(),
                                    )
                                    .await;
                                }
                                // 移动端请求客户端信息
                                "update_client" => {
                                    println!("call update_client 3, {}", &client_func.data);
                                    let clientid: client_data::UpdateClient = serde_json::from_str(&client_func.data).unwrap();
                                    println!("call update_client 4, {}", &clientid.id);
                                    // let clientid: i32 = client_func.data.parse::<i32>().unwrap();
                                    let _  = update_client_common(clientid.id, user_id as i32, tx.clone()).await;
                                }

                                // StartApp
                                _ => {}
                            }
                        } else {
                            println!("not a function!")
                        }
                    }
                    _ => {}
                }
            } else {
                println!("client disconnected id = {}", &user_id);

                let op = DataOperation::Remove { key: user_id };
                if tx.send(op).await.is_err() {
                    println!("add data failed!");
                }

                // UI 移除
                app_handle.emit_all("remove_client_data", &user_id).unwrap();

                return;
            }
        } else {
            println!("client disconnected id = {}", &user_id);

            let op = DataOperation::Remove { key: user_id };
            if tx.send(op).await.is_err() {
                println!("add data failed!");
            }

            // UI 移除
            app_handle.emit_all("remove_client_data", &user_id).unwrap();

            return;
        }
    }
}

async fn http_handler() -> Html<&'static str> {
    Html(std::include_str!("../webdist/index.html"))
}
