use crate::state::AppState;
use russh::{Channel, ChannelMsg};
use serde::{Deserialize, Serialize};
use std::io::Read;
use tauri::{Emitter, State};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: u64,
    pub perms: String,
    pub owner: String,
    pub group: String,
}

#[tauri::command]
pub async fn file_list(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<Vec<FileEntry>, String> {
    // Use find with printf for reliable machine-parseable output
    // Format: type\t\size\t\ts\tperms\towner\tgroup\tname
    let cmd = format!(
        "find -L {} -mindepth 1 -maxdepth 1 -not -name '.' -not -name '..' -printf '%Y\\t%s\\t%T@\\t%#m\\t%u\\t%g\\t%f\\n' 2>/dev/null",
        shell_escape(&path)
    );
    let out = match state.ssh_manager.exec_command(&session_id, &cmd).await {
        Ok(o) => o,
        Err(_) => {
            // Fallback to ls -la for systems without GNU find (BSD/macOS)
            let fb = format!(
                "ls -la {} 2>/dev/null | tail -n +4",
                shell_escape(&path)
            );
            state.ssh_manager.exec_command(&session_id, &fb).await?
        }
    };

    let mut entries = Vec::new();
    let use_find = out.contains('\t');

    if use_find {
        for line in out.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            let parts: Vec<&str> = line.splitn(7, '\t').collect();
            if parts.len() < 7 { continue; }
            let is_dir = parts[0] == "d";
            let size: u64 = parts[1].parse().unwrap_or(0);
            let ts_str = parts[2].split('.').next().unwrap_or("0");
            let modified: u64 = ts_str.parse().unwrap_or(0);
            let perms = parts[3].to_string();
            let owner = parts[4].to_string();
            let group = parts[5].to_string();
            let name = parts[6].to_string();
            entries.push(FileEntry { name, is_dir, size, modified, perms, owner, group });
        }
    } else {
        // Parse ls -la fallback output
        // Format: perms links owner group size month day time/year name...
        for line in out.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 9 { continue; }
            let is_dir = parts[0].starts_with('d');
            let size: u64 = parts[4].parse().unwrap_or(0);
            let modified = 0u64; // can't reliably parse ls date to timestamp
            let perms = parts[0].to_string();
            let owner = parts[2].to_string();
            let group = parts[3].to_string();
            let name = parts[8..].join(" ");
            entries.push(FileEntry { name, is_dir, size, modified, perms, owner, group });
        }
    }

    // Sort: folders first, then case-insensitive A-Z
    entries.sort_by(|a, b| {
        if a.is_dir != b.is_dir {
            return if a.is_dir { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
        }
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });
    Ok(entries)
}

#[tauri::command]
pub async fn file_mkdir(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<(), String> {
    let cmd = format!("mkdir -p {}", shell_escape(&path));
    state.ssh_manager.exec_command(&session_id, &cmd).await?;
    Ok(())
}

#[tauri::command]
pub async fn file_remove(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
    is_dir: bool,
) -> Result<(), String> {
    ensure_safe_remote_path(&path, "remove")?;
    let cmd = if is_dir {
        format!("rm -rf {}", shell_escape(&path))
    } else {
        format!("rm -f {}", shell_escape(&path))
    };
    state.ssh_manager.exec_command(&session_id, &cmd).await?;
    Ok(())
}

#[tauri::command]
pub async fn file_rename(
    state: State<'_, AppState>,
    session_id: String,
    old_path: String,
    new_path: String,
) -> Result<(), String> {
    ensure_safe_remote_path(&old_path, "rename")?;
    ensure_safe_remote_path(&new_path, "rename")?;
    let cmd = format!("mv {} {}", shell_escape(&old_path), shell_escape(&new_path));
    state.ssh_manager.exec_command(&session_id, &cmd).await?;
    Ok(())
}

#[tauri::command]
pub async fn file_copy(
    state: State<'_, AppState>,
    session_id: String,
    src: String,
    dst: String,
) -> Result<(), String> {
    ensure_safe_remote_path(&src, "copy")?;
    ensure_safe_remote_path(&dst, "copy")?;
    let cmd = format!("cp -r {} {}", shell_escape(&src), shell_escape(&dst));
    state.ssh_manager.exec_command(&session_id, &cmd).await?;
    Ok(())
}

#[tauri::command]
pub async fn file_exists(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
) -> Result<bool, String> {
    let cmd = format!("test -e {} && echo yes || echo no", shell_escape(&path));
    let out = state.ssh_manager.exec_command(&session_id, &cmd).await?;
    Ok(out.trim() == "yes")
}

#[tauri::command]
pub async fn file_read(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    session_id: String,
    remote_path: String,
) -> Result<Vec<u8>, String> {
    // Get file size for progress reporting (best-effort)
    let stat_cmd = format!(
        "stat -c %s {} 2>/dev/null || echo 0",
        shell_escape(&remote_path)
    );
    let total_bytes: usize = state
        .ssh_manager
        .exec_command(&session_id, &stat_cmd)
        .await
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or(0);

    let shared = state.ssh_manager.get_session(&session_id).await?;
    let mut ch: Channel<russh::client::Msg> = {
        let sess = shared.lock().await;
        sess.channel_open_session()
            .await
            .map_err(|e| format!("Channel open failed: {}", e))?
    }; // lock released — don't block other operations during transfer
    ch.exec(true, format!("cat {}", shell_escape(&remote_path)).as_bytes())
        .await
        .map_err(|e| format!("Exec failed: {}", e))?;
    let mut data = Vec::new();
    let sid = session_id.clone();
    let rp = remote_path.clone();
    let app = app_handle.clone();
    loop {
        match ch.wait().await {
            Some(ChannelMsg::Data { data: d }) => {
                data.extend_from_slice(&d);
                let _ = app.emit("sftp-progress", serde_json::json!({
                    "session_id": sid,
                    "operation": "download",
                    "path": rp,
                    "bytes_transferred": data.len(),
                    "total_bytes": total_bytes,
                }));
            }
            Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => break,
            _ => {}
        }
    }
    Ok(data)
}

#[tauri::command]
pub async fn file_write(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    session_id: String,
    remote_path: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let total = data.len();
    let shared = state.ssh_manager.get_session(&session_id).await?;
    let ch: Channel<russh::client::Msg> = {
        let sess = shared.lock().await;
        sess.channel_open_session()
            .await
            .map_err(|e| format!("Channel open failed: {}", e))?
    }; // lock released — don't block other operations during transfer
    ch.exec(true, format!("cat > {}", shell_escape(&remote_path)).as_bytes())
        .await
        .map_err(|e| format!("Exec failed: {}", e))?;

    // Stream data in 32KB chunks with progress
    let chunk_size = 32768;
    let sid = session_id.clone();
    let rp = remote_path.clone();
    let app = app_handle.clone();
    for (i, chunk) in data.chunks(chunk_size).enumerate() {
        ch.data(&chunk[..])
            .await
            .map_err(|e| format!("Write failed: {}", e))?;
        let transferred = ((i + 1) * chunk_size).min(total);
        let _ = app.emit(
            "sftp-progress",
            serde_json::json!({
                "session_id": sid,
                "operation": "upload",
                "path": rp,
                "bytes_transferred": transferred,
                "total_bytes": total,
            }),
        );
    }
    // Close stdin (send EOF)
    ch.eof().await.map_err(|e| format!("EOF failed: {}", e))?;
    Ok(())
}

fn shell_escape(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    if s == "/" {
        return "/".to_string();
    }
    // Replace single quotes with '\'' and wrap in single quotes
    let escaped = s.replace('\'', "'\\''");
    format!("'{}'", escaped)
}

fn ensure_safe_remote_path(path: &str, operation: &str) -> Result<(), String> {
    let normalized = normalize_remote_path(path);
    if normalized.is_empty() || normalized == "/" {
        return Err(format!("Refusing to {} root or empty path", operation));
    }
    Ok(())
}

/// Collapse redundant slashes and strip trailing slashes so that
/// `//`, `///`, `/./`, and `/dir//` all resolve to their canonical form.
fn normalize_remote_path(path: &str) -> String {
    let mut result = String::with_capacity(path.len());
    for (i, ch) in path.chars().enumerate() {
        if ch == '/' {
            if i > 0 && result.ends_with('/') {
                // collapse "//" → "/"
                continue;
            }
        }
        result.push(ch);
    }
    // Strip trailing "/" unless it's the root "/" itself
    if result.len() > 1 && result.ends_with('/') {
        result.pop();
    }
    result
}

fn validate_chmod_mode(mode: &str) -> Result<(), String> {
    let m = mode.trim();
    let valid_len = m.len() == 3 || m.len() == 4;
    if valid_len && m.bytes().all(|b| (b'0'..=b'7').contains(&b)) {
        Ok(())
    } else {
        Err("Permissions must be an octal mode like 644 or 0755".to_string())
    }
}

#[tauri::command]
pub async fn file_download_dir(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    session_id: String,
    remote_path: String,
) -> Result<Vec<u8>, String> {
    let shared = state.ssh_manager.get_session(&session_id).await?;
    let mut ch: Channel<russh::client::Msg> = {
        let sess = shared.lock().await;
        sess.channel_open_session()
            .await
            .map_err(|e| format!("Channel open failed: {}", e))?
    }; // lock released — don't block other operations during transfer
    let cmd = format!("tar czf - {} 2>/dev/null", shell_escape(&remote_path));
    ch.exec(true, cmd.as_bytes())
        .await
        .map_err(|e| format!("Exec failed: {}", e))?;

    let mut data = Vec::new();
    let sid = session_id.clone();
    let rp = remote_path.clone();
    let app = app_handle.clone();
    loop {
        match ch.wait().await {
            Some(ChannelMsg::Data { data: d }) => {
                data.extend_from_slice(&d);
                let _ = app.emit("sftp-progress", serde_json::json!({
                    "session_id": sid,
                    "operation": "download",
                    "path": rp,
                    "bytes_transferred": data.len(),
                    "total_bytes": 0,
                }));
            }
            Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => break,
            _ => {}
        }
    }
    // Emit completion event so StatusBar can show success
    let total = data.len();
    let _ = app.emit("sftp-progress", serde_json::json!({
        "session_id": sid,
        "operation": "download",
        "path": rp,
        "bytes_transferred": total,
        "total_bytes": total,
    }));
    Ok(data)
}

#[tauri::command]
pub async fn file_chmod(
    state: State<'_, AppState>,
    session_id: String,
    path: String,
    mode: String,
) -> Result<(), String> {
    validate_chmod_mode(&mode)?;
    ensure_safe_remote_path(&path, "chmod")?;
    let cmd = format!("chmod {} {}", mode.trim(), shell_escape(&path));
    state.ssh_manager.exec_command(&session_id, &cmd).await?;
    Ok(())
}

async fn upload_dir(
    state: &crate::state::AppState,
    app_handle: &tauri::AppHandle,
    session_id: &str,
    local_path: &str,
    remote_dir: &str,
) -> Result<(), String> {
    let dir_name = std::path::Path::new(local_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("uploaded_dir");
    let new_remote = if remote_dir == "/" {
        format!("/{}", dir_name)
    } else {
        format!("{}/{}", remote_dir, dir_name)
    };
    let mkdir_cmd = format!("mkdir -p {}", shell_escape(&new_remote));
    state.ssh_manager.exec_command(session_id, &mkdir_cmd).await?;

    let entries = std::fs::read_dir(local_path).map_err(|e| format!("Cannot read dir: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
        let path = entry.path();
        let path_str = path.to_str().unwrap_or("");
        if path.is_dir() {
            Box::pin(upload_dir(state, app_handle, session_id, path_str, &new_remote)).await?;
        } else {
            upload_file_raw(state, app_handle, session_id, path_str, &new_remote).await?;
        }
    }
    Ok(())
}

async fn upload_file_raw(
    state: &crate::state::AppState,
    app_handle: &tauri::AppHandle,
    session_id: &str,
    local_path: &str,
    remote_dir: &str,
) -> Result<(), String> {
    let name = std::path::Path::new(local_path).file_name().and_then(|n| n.to_str()).unwrap_or("uploaded");
    let remote_path = if remote_dir == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", remote_dir, name)
    };
    let mut file = std::fs::File::open(local_path).map_err(|e| format!("Cannot read: {}", e))?;
    let total = file.metadata().map_err(|e| format!("Cannot stat: {}", e))?.len() as usize;
    let shared = state.ssh_manager.get_session(session_id).await?;
    let ch: Channel<russh::client::Msg> = {
        let sess = shared.lock().await;
        sess.channel_open_session().await.map_err(|e| format!("Channel open: {}", e))?
    }; // lock released — don't block other operations during transfer
    ch.exec(true, format!("cat > {}", shell_escape(&remote_path)).as_bytes())
        .await.map_err(|e| format!("Exec: {}", e))?;
    let chunk_size = 32768;
    let sid = session_id.to_string();
    let app = app_handle.clone();
    let mut transferred = 0usize;
    let mut buffer = vec![0u8; chunk_size];
    loop {
        let n = file.read(&mut buffer).map_err(|e| format!("Read: {}", e))?;
        if n == 0 {
            break;
        }
        ch.data(&buffer[..n]).await.map_err(|e| format!("Write: {}", e))?;
        transferred = (transferred + n).min(total);
        let _ = app.emit("sftp-progress", serde_json::json!({
            "session_id": sid, "operation": "upload", "path": name,
            "bytes_transferred": transferred, "total_bytes": total,
        }));
    }
    ch.eof().await.map_err(|e| format!("EOF: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn file_upload_path(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    session_id: String,
    local_path: String,
    remote_dir: String,
) -> Result<(), String> {
    let meta = std::fs::metadata(&local_path).map_err(|e| format!("Cannot access: {}", e))?;
    // Clone state out of the State wrapper for recursive use
    let inner = state.inner();
    if meta.is_dir() {
        upload_dir(&inner, &app_handle, &session_id, &local_path, &remote_dir).await
    } else {
        upload_file_raw(&inner, &app_handle, &session_id, &local_path, &remote_dir).await
    }
}
