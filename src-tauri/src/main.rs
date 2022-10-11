#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::time::Duration;

use tauri::Manager;
use tokio::time::sleep;
// use tokio::{net::TcpListener, io::{AsyncReadExt, AsyncWriteExt, BufReader, AsyncBufReadExt}, sync::broadcast};

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let app_launcher = app.app_handle();
      tauri::async_runtime::spawn(async move {
        loop {
            tauri::async_runtime::
            sleep(Duration::from_millis(1000)).await;
            println!("looping ...");
            app_launcher.emit_all("keep-alive", "123").unwrap();
        }
      });
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