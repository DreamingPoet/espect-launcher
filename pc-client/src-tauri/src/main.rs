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
use sysinfo::{ProcessExt, System, SystemExt};
use tauri::{AppHandle, CustomMenuItem, SystemTrayMenu};
use tauri::{Manager, SystemTray, SystemTrayEvent};

use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    time::sleep,
};

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

    tauri::Builder::default()
        .setup(|app| {
            let app_launcher = app.app_handle();
            tokio::spawn(handle_data_channel(rx));
            tokio::spawn(handle_websocket(tx2));
            tokio::spawn(check_connect_state(tx_check, app_launcher));

            Ok(())
        })
        // 传递参数给 tauri::command
        .manage(tx_tauri_command)
        // 注册调用的事件
        .invoke_handler(tauri::generate_handler![
            get_saved_host,
            get_local_data,
            open_app_folder,
            start_app,
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
fn get_local_data() -> String {
    let func = ClientFunc {
        func_name: "on_get_client_data".to_string(),
        data: client_data::get_local_data(),
    };
    serde_json::to_string_pretty(&func).unwrap()
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

#[tauri::command]
fn start_app(app: String) {
    println!("start_app  = {:?}", app);

    let s = System::new_all();
    let path = Path::new(&app);
    println!("start_app_exe  = {:?}", path.file_name());

    for process in s.processes_by_exact_name(path.file_name().unwrap().to_str().unwrap()) {
        println!("{} {}", process.pid(), process.name());
        return;
    }

    Command::new(&app).spawn().unwrap();
}

// 断开重连
#[tauri::command]
async fn reconnect(
    host: String,
    tx: tauri::State<'_, Sender<DataOperation>>,
) -> Result<CustomResponse, String> {
    // 保存到配置文件
    let mut config = savefile::SaveFile::new("./myconfig.yml");
    config.set_host(&host.to_owned());

    handle_websocket(tx.inner().clone()).await;
    Ok(CustomResponse {
        message: "".to_string(),
    })
}

// 连接到服务器,并开始监听
async fn handle_websocket(tx: Sender<DataOperation>) {
    let saved_host = &get_saved_host();
    if saved_host.is_empty() {
        return;
    }

    let connect_addr = "ws://".to_string() + saved_host + &"/ws".to_string();

    let url = url::Url::parse(&connect_addr).unwrap();

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (sender, mut receiver) = ws_stream.split();

    let op = DataOperation::Set { server: sender };
    if tx.send(op).await.is_err() {
        println!("send data failed!");
    } else {
        println!("send ok");
    }

    loop {
        if let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(t) => {
                        println!("on get client str: {:?}", t);

                        // // 接收到 Json 字符串
                        // if let Ok(client_func) = serde_json::from_str::<ClientFunc>(&t) {
                        //     // 匹配函数名称
                        //     match &client_func.func_name as &str {
                        //         // 注册客户端
                        //         "regist_client" => {}
                        //         // 获取客户端本地数据
                        //         "on_get_client_data" => {
                        //             // 解成Json , 设置 id 再转回 string
                        //             let mut client_data: ClientData =
                        //                 serde_json::from_str(&client_func.data).unwrap();
                        //             client_data.id = user_id as i32;
                        //             // println!("user_id = {}", client_data.ip);
                        //             // let client_func: ClientFunc = serde_json::from_str(&t).unwrap();
                        //             let client_str =
                        //                 serde_json::to_string_pretty(&client_data).unwrap();
                        //             app_handle
                        //                 .emit_all("on_get_client_data", &client_str)
                        //                 .unwrap();
                        //         }
                        //         _ => {}
                        //     }
                        // } else {
                        //     println!("not a function!")
                        // }
                    }
                    _ => {}
                }
            } else {
                println!("disconnected to server!::1");
                return;
            }
        } else {
            println!("disconnected to server!::2 ");
            return;
        }
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
                    // if let Some(sender) = data.get_mut(&key) {
                    //     if sender.send(Message::Text(msg)).await.is_err() {
                    //         println!("send data failed!");
                    //         // return;
                    //     }
                    // }
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
                    if let Some(ref mut current_server) = data {
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

async fn check_connect_state(tx: Sender<DataOperation>, app_handle: AppHandle) {
    loop {
        sleep(Duration::from_millis(1000)).await;

        // 临时接受管道
        let (resp_tx, resp_rx) = oneshot::channel();

        let op = DataOperation::Check { resp: resp_tx };

        println!("start get data");
        if tx.send(op).await.is_err() {
            println!("get data failed!");
        }
        let res = resp_rx.await;
        if let Ok(check_res) = res {
            match check_res {
                true => {
                    app_handle.emit_all("check_connect_state", true).unwrap();
                }
                false => {
                    app_handle.emit_all("check_connect_state", false).unwrap();
                }
            }
        }
    }
}

/// ```
/// #[tokio::main]
/// async fn main() {
///     let connect_addr =
///         env::args().nth(1).unwrap_or_else(|| panic!("this program requires at least one argument"));

///     let url = url::Url::parse(&connect_addr).unwrap();

// 创建一个数据通道，只能转发数据，不能处理数据
///     let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
///     // 写通道给函数 read_stdin
///     tokio::spawn(read_stdin(stdin_tx));

///     let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
///     println!("WebSocket handshake has been successfully completed");

///     let (write, read) = ws_stream.split();
// 不断把数据通道中的接受到的数据 通过 write 写入到ws_socket
///     let stdin_to_ws = stdin_rx.map(Ok).forward(write);
///
///     let ws_to_stdout = {
///         read.for_each(|message| async {
///             let data = message.unwrap().into_data();
///             tokio::io::stdout().write_all(&data).await.unwrap();
///         })
///     };

///     pin_mut!(stdin_to_ws, ws_to_stdout);
///     future::select(stdin_to_ws, ws_to_stdout).await;
/// }

/// // Our helper method which will read data from stdin and send it along the
/// // sender provided.
/// async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
///     let mut stdin = tokio::io::stdin();
///     loop {
///         let mut buf = vec![0; 1024];
///         let n = match stdin.read(&mut buf).await {
///             Err(_) | Ok(0) => break,
///             Ok(n) => n,
///         };
///         buf.truncate(n);
///
///         // 发送数据到通道中
///         tx.unbounded_send(Message::binary(buf)).unwrap();
///     }
/// }

/// ```
struct TestWS {}
