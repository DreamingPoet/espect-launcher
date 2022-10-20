#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

mod client_data;
mod savefile;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use std::{env, fs};

use client_data::ClientFunc;
use futures::{
    stream::{SplitSink, StreamExt},
    SinkExt,
};
use tauri::{AppHandle, CustomMenuItem, SystemTrayMenu};
use tauri::{Manager, SystemTray, SystemTrayEvent};

use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    time::sleep,
};

use crate::client_data::ClientApp;

type ServerSender = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
//  对共享数据（webSocket server）的所有的操作
enum DataOperation {
    // send data to ws-server
    Send { msg: String },
    // set ws-server when connected
    Set { server: ServerSender },
    // close ws-server
    Close {},
    // check connect state
    Check { resp: Responder<bool> },
}

// 对共享数据中服务器操作的响应
type Responder<T> = oneshot::Sender<T>;

#[derive(serde::Serialize)]
struct CustomResponse {
    message: String,
}

// 对数据操作的响应
// type Responder<T> = oneshot::Sender<T>;

#[tokio::main]
async fn main() {
    // ======================= tauri start =======================
    // Create the SystemTrayMenu:
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let tray_menu = SystemTrayMenu::new().add_item(quit);

    // Initialize a new tray instance
    // Add the tray menu to the SystemTray instance:
    let system_tray = SystemTray::new().with_menu(tray_menu);
    // ======================= tauri end =======================

    // ======================= websocket start =======================

    // 创建一个访问和操作共享数据的通道
    let (tx, rx) = mpsc::channel(32);
    let tx2 = tx.clone();
    let tx_check = tx.clone();
    let tx_tauri_command = tx.clone(); // for tauri::command

    // ======================= websocket end =======================

    let local_apps = client_data::get_local_apps();

    tauri::Builder::default()
        .setup(|app| {
            let app_launcher = app.app_handle();
            tokio::spawn(handle_data_channel(rx));
            tokio::spawn(handle_websocket(tx2, local_apps));
            tokio::spawn(check_connect_state(tx_check, app_launcher));

            Ok(())
        })
        // 传递参数给 tauri::command
        .manage(tx_tauri_command)
        // 注册调用的事件
        .invoke_handler(tauri::generate_handler![
            get_saved_host,
            open_app_folder,
            reconnect,
        ])
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                println!("system tray received a left click");
            }
            SystemTrayEvent::RightClick {
                position: _,
                size: _,
                ..
            } => {
                println!("system tray received a right click");
            }
            SystemTrayEvent::DoubleClick {
                position: _,
                size: _,
                ..
            } => {
                for i in app.windows().iter() {
                    let _ = &i.1.show().unwrap();
                    println!("window name = {}", i.0);
                }
                println!("system tray received a double click");
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}

#[tauri::command]
fn get_saved_host() -> String {
    let config = savefile::SaveFile::new("./myconfig.yml");
    let s = config.get_host();
    println!("config = {}", s);
    s
}

#[tauri::command]
fn open_app_folder() {
    // let o = Path::new("a.rs");
    // let b = o.is_file();
    let path = env::current_dir().unwrap();

    let path = path.as_path();
    let path = path.to_str().unwrap().to_owned();
    let path = path + "\\apps";
    println!("current path  = {:?}", &path);

    if !Path::new("./apps").exists() {
        // 创建一个目录
        fs::create_dir("./apps").unwrap();
    }

    println!("Opening");
    Command::new("explorer")
        .arg(path) // <- Specify the directory you'd like to open.
        .spawn()
        .unwrap();
}

async fn start_app(app: String) {
    let app_exe = app + &".exe";
    let path = env::current_dir().unwrap().join("apps");
    let path = path.join(&app_exe);

    println!("start_app_path  = {:?}", path);

    println!("start_app_exe  = {:?}", app_exe);

    if client_data::is_app_running(&app_exe) {
        return;
    }

    Command::new(&path).spawn().unwrap();
}

fn get_local_data() -> String {
    let func = ClientFunc {
        func_name: "on_get_client_data".to_string(),
        data: client_data::get_local_data(),
    };
    serde_json::to_string_pretty(&func).unwrap()
}

fn get_update_data(local_apps: &Vec<ClientApp>) -> String {
    let func = ClientFunc {
        func_name: "on_update_client".to_string(),
        data: client_data::get_update_data(local_apps),
    };

    serde_json::to_string_pretty(&func).unwrap()
}

// 获取IP

// 获取本机名称

// 断开重连
#[tauri::command]
async fn reconnect(
    host: String,
    tx: tauri::State<'_, Sender<DataOperation>>,
) -> Result<CustomResponse, String> {
    // 保存到配置文件
    let mut config = savefile::SaveFile::new("./myconfig.yml");
    config.set_host(&host.to_owned());
    let local_apps = client_data::get_local_apps();
    handle_websocket(tx.inner().clone(), local_apps).await;
    Ok(CustomResponse {
        message: "".to_string(),
    })
}

// 连接到服务器,并开始监听
async fn handle_websocket(tx: Sender<DataOperation>, local_apps: Vec<ClientApp>) {
    let saved_host = &get_saved_host();
    if saved_host.is_empty() {
        return;
    }

    let connect_addr = "ws://".to_string() + saved_host + &"/ws".to_string();

    let url = url::Url::parse(&connect_addr).unwrap();

    if let Ok((ws_stream, _msg)) = connect_async(url).await {
        println!("WebSocket handshake has been successfully completed");

        let (sender, mut receiver) = ws_stream.split();

        // 设置数据管道中的数据
        let op = DataOperation::Set { server: sender };
        if tx.send(op).await.is_err() {
            println!("send data failed!");
        } else {
            println!("send ok");
        }

        // 发生客户端数据到服器

        let op = DataOperation::Send {
            msg: get_local_data(),
        };
        let _ = tx.send(op).await;

        // 监听来自服务器的数据
        tokio::spawn(async move {
            loop {
                if let Some(msg) = receiver.next().await {
                    if let Ok(msg) = msg {
                        match msg {
                            Message::Text(t) => {
                                println!("on get server str: {:?}", t);

                                // 接收到 Json 字符串
                                if let Ok(client_func) = serde_json::from_str::<ClientFunc>(&t) {
                                    // 匹配函数名称
                                    match &client_func.func_name as &str {
                                        // 启动应用
                                        "start_app" => {
                                            start_app(client_func.data).await;
                                        }
                                        // 关闭应用
                                        "stop_app" => {
                                            client_data::kill_app(&client_func.data);
                                        }
                                        // 获取自己需要更新的数据
                                        "update_client" => {
                                            // 发生客户端数据到服器

                                            let op = DataOperation::Send {
                                                msg: get_update_data(&local_apps),
                                            };
                                            let _ = tx.send(op).await;
                                        }
                                        _ => {}
                                    }
                                } else {
                                    println!("not a function!")
                                }
                            }
                            _ => {}
                        }
                    } else {
                        let op = DataOperation::Close {};
                        let _ = tx.send(op).await;
                        println!("disconnected to server!::1");
                        return;
                    }
                } else {
                    let op = DataOperation::Close {};
                    let _ = tx.send(op).await;

                    println!("disconnected to server!::2 ");
                    return;
                }
            }
        });
    }
}

// 处理数据
async fn handle_data_channel(mut rx: Receiver<DataOperation>) {
    let mut data: Option<ServerSender> = None;

    loop {
        if let Some(op) = rx.recv().await {
            match op {
                DataOperation::Send { msg } => {
                    println!("send data to server! ");
                    if let Some(ref mut sender) = data {
                        if sender.send(Message::Text(msg)).await.is_err() {
                            println!("send data failed!");
                        }
                    }
                }
                DataOperation::Set { server } => {
                    if let Some(ref mut current_server) = data {
                        let _ = current_server.close().await;
                    }
                    data = Some(server);
                }
                DataOperation::Close {} => {
                    if let Some(ref mut server) = data {
                        let _ = server.close().await;
                        data = None;
                    }
                }
                DataOperation::Check { resp } => {
                    if let Some(ref mut _current_server) = data {
                        let _ = resp.send(true);
                    } else {
                        let _ = resp.send(false);
                    }
                }
            }
        } else {
            println!("recv failed!");
        }
    }
}

// 定时检查连接状态
async fn check_connect_state(tx: Sender<DataOperation>, app_handle: AppHandle) {
    loop {
        // println!("checking 1... ...");
        sleep(Duration::from_millis(1000)).await;

        // 临时接受管道
        let (resp_tx, resp_rx) = oneshot::channel();

        let op = DataOperation::Check { resp: resp_tx };

        let _ = tx.send(op).await;

        let res = resp_rx.await;
        // println!("checking 2... ...");

        if let Ok(check_res) = res {
            match check_res {
                true => {
                    app_handle.emit_all("check_connect_state", true).unwrap();
                }
                false => {
                    app_handle.emit_all("check_connect_state", false).unwrap();
                    let local_apps = client_data::get_local_apps();
                    handle_websocket(tx.clone(), local_apps).await;
                }
            }
        } else {
            app_handle.emit_all("check_connect_state", false).unwrap();
            let local_apps = client_data::get_local_apps();
            handle_websocket(tx.clone(), local_apps).await;
        }
    }
}
