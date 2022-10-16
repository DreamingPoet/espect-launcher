#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod savefile;
mod client_data;
use std::{fs, env};
use std::path::Path;
use std::process::Command;

use client_data::ClientFunc;
use tauri::{CustomMenuItem, SystemTrayMenu};
use tauri::{Manager, SystemTray, SystemTrayEvent};

fn main() {
    // Create the SystemTrayMenu:
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let tray_menu = SystemTrayMenu::new().add_item(quit);

    // Initialize a new tray instance
    // Add the tray menu to the SystemTray instance:
    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        // 注册调用的事件
        .invoke_handler(tauri::generate_handler![
            get_saved_host,
            get_local_data,
            open_app_folder,

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

    println!( "Opening" );
    Command::new( "explorer" )
        .arg( path ) // <- Specify the directory you'd like to open.
        .spawn( )
        .unwrap( );

}


// let dir = path.as_path().read_dir().unwrap();
// for x in dir {
//    if let Ok(path) = x {
//        println!("{:?}", path.file_name()); // 该路径下所有文件和文件夹名称
//        // 是否存在某个文件
//        if path.file_name().eq("Cargo.toml") {
//             println!("存在 [`Cargo.toml`] 文件!");
//        }
//     }
// }

