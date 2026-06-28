use crate::terminal::session::LocalPtySession;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::RwLock;
use uuid::Uuid;

struct LocalPtyHandle {
    stdin_tx: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
    resize_tx: tokio::sync::mpsc::UnboundedSender<(u16, u16)>,
    read_handle: tokio::task::JoinHandle<()>,
    write_handle: tokio::task::JoinHandle<()>,
    resize_handle: tokio::task::JoinHandle<()>,
}

pub struct LocalTerminalManager {
    sessions: Arc<RwLock<HashMap<String, LocalPtyHandle>>>,
}

impl LocalTerminalManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn spawn(
        &self,
        app_handle: tauri::AppHandle,
        cols: u16,
        rows: u16,
    ) -> Result<String, String> {
        let session_id = Uuid::new_v4().to_string();

        let session = LocalPtySession::spawn(
            app_handle.clone(),
            session_id.clone(),
            cols,
            rows,
        )?;

        let handle = LocalPtyHandle {
            stdin_tx: session.stdin_tx,
            resize_tx: session.resize_tx,
            read_handle: session.read_handle,
            write_handle: session.write_handle,
            resize_handle: session.resize_handle,
        };

        self.sessions
            .write()
            .await
            .insert(session_id.clone(), handle);

        let _ = app_handle.emit(
            "connection-status",
            serde_json::json!({ "session_id": session_id, "status": "connected" }),
        );

        Ok(session_id)
    }

    pub async fn write(&self, session_id: &str, data: Vec<u8>) -> Result<(), String> {
        let sessions = self.sessions.read().await;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| format!("Local terminal session not found: {}", session_id))?;
        handle
            .stdin_tx
            .send(data)
            .map_err(|e| format!("Failed to write to local terminal: {}", e))?;
        Ok(())
    }

    pub async fn resize(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), String> {
        let sessions = self.sessions.read().await;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| format!("Local terminal session not found: {}", session_id))?;
        handle
            .resize_tx
            .send((cols, rows))
            .map_err(|e| format!("Failed to resize local terminal: {}", e))?;
        Ok(())
    }

    pub async fn kill(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        if let Some(handle) = sessions.remove(session_id) {
            handle.read_handle.abort();
            handle.write_handle.abort();
            handle.resize_handle.abort();
        }
        Ok(())
    }
}
