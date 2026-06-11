use serde::Serialize;
use tauri::{Emitter, State};
use crate::state::AppState;
use crate::models::connection::ConnectionProfile;
use russh::ChannelMsg;

#[derive(Serialize, Clone, Debug)]
pub struct TrafficStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

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
pub async fn ping(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    session_id: String,
) -> Result<(), String> {
    // Dedicated ping: opens its own SSH channel directly, bypassing the exec queue.
    // Session lock is released immediately after channel open so concurrent operations
    // (other pings, file transfers, exec commands) are never blocked.
    let t0 = std::time::Instant::now();
    let shared = state.ssh_manager.get_session(&session_id).await?;
    let mut ch = {
        let sess = shared.lock().await;
        sess.channel_open_session()
            .await
            .map_err(|e| format!("ping channel open: {}", e))?
    }; // lock released here
    ch.exec(true, b"echo 1")
        .await
        .map_err(|e| format!("ping exec: {}", e))?;
    // Drain output until channel closes
    loop {
        match ch.wait().await {
            Some(ChannelMsg::Data { .. }) => {}
            Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => break,
            _ => {}
        }
    }
    let elapsed = t0.elapsed().as_millis();
    if elapsed > 500 {
        let _ = app_handle.emit("debug-event", serde_json::json!({
            "session_id": session_id,
            "level": "warn",
            "source": "Ping",
            "message": format!("Ping took {}ms", elapsed),
            "elapsed_ms": elapsed,
        }));
    }
    Ok(())
}

/// Query server network interface traffic from /proc/net/dev.
/// Sums all non-loopback interfaces. Returns cumulative RX/TX byte counters.
/// Frontend computes delta between consecutive calls to get speed.
#[tauri::command]
pub async fn server_traffic(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<TrafficStats, String> {
    let out = state.ssh_manager.exec_command(&session_id, "cat /proc/net/dev 2>/dev/null || echo ''").await?;
    let mut rx: u64 = 0;
    let mut tx: u64 = 0;
    for line in out.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 { continue; }
        // Skip loopback
        let iface = parts[0].trim_end_matches(':');
        if iface == "lo" { continue; }
        rx += parts[1].parse::<u64>().unwrap_or(0);
        tx += parts[9].parse::<u64>().unwrap_or(0);
    }
    Ok(TrafficStats { rx_bytes: rx, tx_bytes: tx })
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

/// Execute a command on multiple sessions concurrently and return per-session results.
#[derive(Serialize, Clone, Debug)]
pub struct BatchExecResult {
    pub session_id: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub elapsed_ms: u64,
}

#[tauri::command]
pub async fn batch_exec(
    state: State<'_, AppState>,
    session_ids: Vec<String>,
    command: String,
) -> Result<Vec<BatchExecResult>, String> {
    let mut handles = Vec::new();
    for sid in &session_ids {
        let ssh = state.ssh_manager.clone();
        let sid = sid.clone();
        let cmd = command.clone();
        handles.push(tokio::spawn(async move {
            let t0 = std::time::Instant::now();
            match ssh.exec_command(&sid, &cmd).await {
                Ok(out) => BatchExecResult {
                    session_id: sid,
                    output: Some(out),
                    error: None,
                    elapsed_ms: t0.elapsed().as_millis() as u64,
                },
                Err(e) => BatchExecResult {
                    session_id: sid,
                    output: None,
                    error: Some(e),
                    elapsed_ms: t0.elapsed().as_millis() as u64,
                },
            }
        }));
    }
    let mut results = Vec::new();
    for h in handles {
        match h.await {
            Ok(r) => results.push(r),
            Err(_) => {} // join error — skip
        }
    }
    Ok(results)
}

/// Frontend response to a host-key verification prompt.
/// `remember` controls whether the accepted key is persisted to known_hosts.
#[tauri::command]
pub async fn confirm_host_key(
    state: State<'_, AppState>,
    request_id: String,
    accepted: bool,
    remember: bool,
) -> Result<(), String> {
    state.ssh_manager.host_key_verification
        .confirm(&request_id, accepted, remember)
        .await
}
