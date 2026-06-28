use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn local_terminal_spawn(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    cols: u16,
    rows: u16,
) -> Result<String, String> {
    state
        .local_terminal_manager
        .spawn(app_handle, cols, rows)
        .await
}

#[tauri::command]
pub async fn local_terminal_write(
    state: State<'_, AppState>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    state
        .local_terminal_manager
        .write(&session_id, data)
        .await
}

#[tauri::command]
pub async fn local_terminal_resize(
    state: State<'_, AppState>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    state
        .local_terminal_manager
        .resize(&session_id, cols, rows)
        .await
}

#[tauri::command]
pub async fn local_terminal_kill(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    state
        .local_terminal_manager
        .kill(&session_id)
        .await
}
