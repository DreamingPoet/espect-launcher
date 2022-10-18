#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tokio::io;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio_tungstenite::tungstenite::Message;

mod client_data;
mod savefile;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use std::{env, fs, default};

use client_data::ClientFunc;
use sysinfo::{ProcessExt, System, SystemExt};
use tauri::{CustomMenuItem, SystemTrayMenu};
use tauri::{Manager, SystemTray, SystemTrayEvent};
use futures::{
    stream::{SplitSink, StreamExt},
    SinkExt,
};

use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        oneshot,
    },
    time::sleep,
};

//  对共享数据（webSocket server）的所有的操作
enum DataOperation {
    // send data to ws-server
    Send {
        msg: String,
    },
    // set ws-server when connected
    Set {
        server: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    },
    // close ws-server
    Close {
    }
}

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
    let tx_tauri_command = tx.clone(); // for tauri::command

    // ======================= websocket end =======================

    tauri::Builder::default()
        .setup(|app| {
            let app_launcher = app.app_handle();
            tokio::spawn(handle_data_channel(rx));
            tokio::spawn(handle_websocket(tx2));
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
    // String::from("127.0.0.1:3000")
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

#[tauri::command]
async fn reconnect(tx: tauri::State<'_, Sender<DataOperation>>,) -> Result<CustomResponse, String> {

    let op = DataOperation::Close{};

    println!("reconnect");

    if tx.send(op).await.is_err() {
        println!("send data failed!");
        Err("No result".into())
    } else {
        println!("send ok");
        Ok(CustomResponse {
            message: "".to_string()
        })
    }
}



async fn handle_websocket(tx: Sender<DataOperation>) {
    let connect_addr = "ws://192.168.0.33:3000/ws".to_string();

    // let connect_addr = get_saved_host();
    // if let

    let url = url::Url::parse(&connect_addr).unwrap();

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    // let op = DataOperation::Set { server: ws_stream };
    // if tx.send(op).await.is_err() {
    //     println!("send data failed!");
    // } else {
    //     println!("send ok");
    // }

    let (mut write, read) = ws_stream.split();

    sleep(Duration::from_millis(4000)).await;
    write.close().await;
    
    // // 循环接收来自服务器的数据
    // read.for_each(|msg| async {
    //     if let Ok(msg) = msg {
    //         match msg {
    //             Message::Text(t) => {
    //                 println!("on get client str: {:?}", t);

    //                 // // 接收到 Json 字符串
    //                 // if let Ok(client_func) = serde_json::from_str::<ClientFunc>(&t) {
    //                 //     // 匹配函数名称
    //                 //     match &client_func.func_name as &str {
    //                 //         // 注册客户端
    //                 //         "regist_client" => {}
    //                 //         // 获取客户端本地数据
    //                 //         "on_get_client_data" => {
    //                 //             // 解成Json , 设置 id 再转回 string
    //                 //             let mut client_data: ClientData =
    //                 //                 serde_json::from_str(&client_func.data).unwrap();
    //                 //             client_data.id = user_id as i32;
    //                 //             // println!("user_id = {}", client_data.ip);
    //                 //             // let client_func: ClientFunc = serde_json::from_str(&t).unwrap();
    //                 //             let client_str =
    //                 //                 serde_json::to_string_pretty(&client_data).unwrap();
    //                 //             app_handle
    //                 //                 .emit_all("on_get_client_data", &client_str)
    //                 //                 .unwrap();
    //                 //         }
    //                 //         _ => {}
    //                 //     }
    //                 // } else {
    //                 //     println!("not a function!")
    //                 // }
    //             }
    //             _ => {}
    //         }
    //     } else {
    //         println!("disconnected to server");
    //         return;
    //     }
    // })
    // .await;



}


// 处理数据
async fn handle_data_channel(mut rx: Receiver<DataOperation>) {
    let mut data:Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>> = None;
    
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
                        let _ = current_server.close(None).await;
                    }
                    data = Some(server);
                }
                DataOperation::Close {  } => {
                    if let Some(ref mut server) = data {
                        let _ =  server.close(None).await;
                        data = None;
                    }
                }
            }
        } else {
            println!("recv failed!");
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
