#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use futures::{stream::{SplitSink, StreamExt}, SinkExt};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    time::sleep,
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
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//  对客户端的所有的操作
enum ClientOperation {
    Send {
        key: usize,
        msg: String,
    },
    Add {
        key: usize,
        client: SplitSink<WebSocket, Message>,
    },
    Get {
        resp: Responder<Option<HashSet<String>>>,
    },
}

// 对客户端操作的响应
type Responder<T> = oneshot::Sender<T>;

// 全局唯一的UUID (也可以放到 redis 中)
static NEXT_USERID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

type ClientMap = HashMap<usize, SplitSink<WebSocket, Message>>;

#[tokio::main]
async fn main() {
    // 创建一个客户端的集合数据
    
    let (tx, rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    tauri::Builder::default()
        .setup(|_app| {
            tokio::spawn(handle_data_channel(rx));
            tokio::spawn(printclients(tx2));
            tokio::spawn(start_axum(tx));

            //     let app_launcher = app.app_handle();
            //     tauri::async_runtime::spawn(async move {
            //         loop {
            //             sleep(Duration::from_millis(1000)).await;
            //             // println!("looping ...");
            //             app_launcher.emit_all("keep-alive", "123").unwrap();
            //         }
            //     });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// 处理数据
async fn handle_data_channel(mut rx: Receiver<ClientOperation>) {
    let mut data: ClientMap = HashMap::default();
    println!("into data handle");
    loop {
        if let Some(op) = rx.recv().await {
            match op {
                ClientOperation::Send { key, msg } => {
                    // let sender = data.get_mut(&key);
                    // sender.unwrap().send(  Message::Text(String::from("Username already taken."))  );
                    println!("send data to client! ");
                    if let Some(sender) = data.get_mut(&key) {
                        if sender.send(Message::Text(msg)).await.is_err() {
                            println!("send data failed!");
                            // return;
                        }
                    }
                    
                }
                ClientOperation::Add { key, client } => {
                    data.insert(key, client);
                }
                ClientOperation::Get { resp } => {
                    let mut res: HashSet<String> = HashSet::default();
                    for i in data.iter() {
                        res.insert(i.0.to_string());
                    }
                    let _ = resp.send(Some(res));
                }
            }
        } else {
            println!("recv failed!");
        }
    }
}

async fn printclients(tx: Sender<ClientOperation>) {
    println!("into get");
    loop {
        sleep(Duration::from_millis(5000)).await;
        // 临时接受管道
        let (resp_tx, resp_rx) = oneshot::channel();

        let op = ClientOperation::Get { resp: resp_tx };

        println!("start get data");
        if tx.send(op).await.is_err() {
            println!("get data failed!");
            // return;
        }
        let res = resp_rx.await;


        // println!("data = {:?}", res);
        
        for i in res.unwrap().unwrap().iter() {
            let id =  i.parse::<usize>().unwrap();
            println!("client id = {}", id);
            let op = ClientOperation::Send { key: id, msg: String::from("hello") };
            if tx.send(op).await.is_err() {
                println!("send data failed!");
                // return;
            }
        }

    }
}
// init a background process on the command, and emit periodic events only to the window that used the command
async fn start_axum(tx: Sender<ClientOperation>) {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    // build our application with some routes
    let app = Router::with_state(tx)
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
    State(tx): State<Sender<ClientOperation>>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }
    ws.on_upgrade(|socket| handle_socket(socket, tx))
}

async fn handle_socket(socket: WebSocket, tx: Sender<ClientOperation>) {
    let user_id = NEXT_USERID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // By splitting we can send and receive at the same time.
    let (sender, mut receiver) = socket.split();

    println!("user_id = {}", user_id);

    let op = ClientOperation::Add {
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
                        println!("client sent str: {:?}", t);
                    }
                    Message::Binary(_) => {
                        println!("client sent binary data");
                    }
                    Message::Ping(_) => {
                        println!("socket ping");
                    }
                    Message::Pong(_) => {
                        println!("socket pong");
                    }
                    Message::Close(_) => {
                        println!("client disconnected");
                        return;
                    }
                }
            } else {
                println!("client disconnected");
                return;
            }
        }
    }
}

async fn http_handler() -> Html<&'static str> {
    Html(std::include_str!("../webdist/index.html"))
}
