use tauri::State;
use crate::state::AppState;
use crate::models::connection::ConnectionProfile;

#[tauri::command]
pub async fn connect(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    profile: ConnectionProfile,
) -> Result<String, String> {
    state.ssh_manager.connect(profile, app_handle).await
}

#[tauri::command]
pub async fn disconnect(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    state.ssh_manager.disconnect(&session_id).await
}

#[tauri::command]
pub async fn terminal_write(
    state: State<'_, AppState>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    state.ssh_manager.write(&session_id, data).await
}

#[tauri::command]
pub async fn terminal_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    state.ssh_manager.resize(&session_id, cols, rows).await
}

#[tauri::command]
pub async fn exec_command(
    state: State<'_, AppState>,
    session_id: String,
    command: String,
) -> Result<String, String> {
    state.ssh_manager.exec_command(&session_id, &command).await
}

#[tauri::command]
pub fn clipboard_read() -> Result<String, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.get_text().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clipboard_write(text: String) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(&text).map_err(|e| e.to_string())
}
