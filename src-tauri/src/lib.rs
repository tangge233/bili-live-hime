mod commands;
mod connection;
mod storage;
mod utils;

use crate::commands::{
    clear_app_secrets, comment_connect, comment_disconnect, comment_is_connected, load_app_config,
    load_app_secrets, save_app_config, save_app_secrets,
};
use std::sync::Arc;
use tauri::{Manager, PhysicalPosition, RunEvent, WindowEvent};
use tokio::sync::Mutex;

pub struct ConnectionState {
    handle: Option<tokio::task::JoinHandle<()>>,
}

pub type AppState = Arc<Mutex<ConnectionState>>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .manage(AppState::new(Mutex::new(ConnectionState { handle: None })))
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Create the config directory in Rust so the WebView does not need
            // broad filesystem permissions.
            storage::ensure_config_dir(app.handle()).map_err(std::io::Error::other)?;

            if let Some(window) = app.get_webview_window("comments") {
                if let Some(monitor) = window.primary_monitor()? {
                    let monitor_position = monitor.position();
                    let monitor_size = monitor.size();
                    let window_size = window.outer_size()?;
                    let margin = 20_i32;
                    let available_width =
                        monitor_size.width.saturating_sub(window_size.width) as i32;
                    let x = monitor_position.x + (available_width - margin).max(0);
                    let y = monitor_position.y + margin;
                    window.set_position(PhysicalPosition::new(x, y))?;
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            comment_connect,
            comment_disconnect,
            comment_is_connected,
            load_app_config,
            save_app_config,
            load_app_secrets,
            save_app_secrets,
            clear_app_secrets
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(move |_app_handle, _event| match &_event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { .. },
            ..
        } if label == "main" => {
            _app_handle.exit(0);
        }
        RunEvent::ExitRequested { .. } => {
            let app_state = _app_handle.state::<AppState>();
            tauri::async_runtime::block_on(async {
                let mut conn_state = app_state.lock().await;
                if let Some(handle) = conn_state.handle.take() {
                    handle.abort();
                    println!("Connection task aborted");
                }
            });
        }
        _ => (),
    });
}
