use tauri::{AppHandle, Emitter};

use crate::{connection::SocketClient, storage, AppState};

#[tauri::command]
pub async fn comment_connect(
    uid: u64,
    room: u32,
    token: String,
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut conn_state = state.lock().await;
    if let Some(handle) = &conn_state.handle {
        if !handle.is_finished() {
            return Err("Already connected".to_string());
        }
    }

    let handle = tokio::spawn(async move {
        let client = SocketClient::new(app_handle);
        if let Err(e) = client.connection(uid, room, token).await {
            eprintln!("Connection error: {}", e);
        }
    });

    conn_state.handle = Some(handle);

    Ok("Connecting...".to_string())
}

#[tauri::command]
pub async fn comment_disconnect(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let mut conn_state = state.lock().await;

    if let Some(handle) = conn_state.handle.take() {
        handle.abort();
    }

    // Let every window (main and floating comments) clear its local state.
    let _ = app_handle.emit(
        "connection-closed",
        serde_json::json!({ "status": "disconnected" }),
    );

    Ok("Disconnected".to_string())
}

#[tauri::command]
pub async fn comment_is_connected(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let conn_state = state.lock().await;

    Ok(match &conn_state.handle {
        Some(handle) => !handle.is_finished(),
        None => false,
    })
}

#[tauri::command]
pub async fn load_app_config(app: AppHandle) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || storage::load_config(&app))
        .await
        .map_err(|e| format!("config load task failed: {e}"))?
}

#[tauri::command]
pub async fn save_app_config(app: AppHandle, contents: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || storage::save_config(&app, &contents))
        .await
        .map_err(|e| format!("config save task failed: {e}"))?
}

#[tauri::command]
pub async fn load_app_secrets(app: AppHandle) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || storage::load_secrets(&app))
        .await
        .map_err(|e| format!("secret load task failed: {e}"))?
}

#[tauri::command]
pub async fn save_app_secrets(app: AppHandle, contents: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || storage::save_secrets(&app, &contents))
        .await
        .map_err(|e| format!("secret save task failed: {e}"))?
}

#[tauri::command]
pub async fn clear_app_secrets(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || storage::clear_secrets(&app))
        .await
        .map_err(|e| format!("secret clear task failed: {e}"))?
}
