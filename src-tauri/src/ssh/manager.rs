use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;

use crate::models::connection::ConnectionProfile;
use crate::ssh::session::SharedSession;

pub struct SshManager {
    sessions: Arc<RwLock<HashMap<String, SshSessionHandle>>>,
}

pub struct SshSessionHandle {
    stdin_tx: mpsc::UnboundedSender<Vec<u8>>,
    handle: JoinHandle<()>,
    exec_tx: mpsc::UnboundedSender<(String, tokio::sync::oneshot::Sender<Result<String, String>>)>,
    resize_tx: mpsc::UnboundedSender<(u16, u16)>,
    pub session: SharedSession,
}

impl SshManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn connect(
        &self,
        profile: ConnectionProfile,
        app_handle: tauri::AppHandle,
    ) -> Result<String, String> {
        let session_id = uuid::Uuid::new_v4().to_string();

        let (session, stdout_rx) =
            crate::ssh::session::SshSession::connect(profile.clone(), app_handle.clone(), session_id.clone())
                .await?;

        let shared = session
            .session_rx
            .await
            .map_err(|_| "SSH session not established".to_string())?;

        let mut sessions = self.sessions.write().await;
        sessions.insert(
            session_id.clone(),
            SshSessionHandle {
                stdin_tx: session.stdin_tx.clone(),
                handle: session.handle,
                exec_tx: session.exec_tx,
                resize_tx: session.resize_tx,
                session: shared,
            },
        );

        let app = app_handle.clone();
        let sid = session_id.clone();
        tokio::spawn(async move {
            let mut rx = stdout_rx;
            while let Some(data) = rx.recv().await {
                let _ = app.emit(
                    "terminal-data",
                    serde_json::json!({
                        "session_id": sid,
                        "data": data,
                    }),
                );
            }
        });

        Ok(session_id)
    }

    pub async fn disconnect(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        if let Some(handle) = sessions.remove(session_id) {
            handle.handle.abort();
        }
        Ok(())
    }

    pub async fn write(&self, session_id: &str, data: Vec<u8>) -> Result<(), String> {
        let sessions = self.sessions.read().await;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| "Session not found".to_string())?;
        handle
            .stdin_tx
            .send(data)
            .map_err(|_| "Failed to write to session".to_string())
    }

    pub async fn exec_command(
        &self,
        session_id: &str,
        command: &str,
    ) -> Result<String, String> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();

        {
            let sessions = self.sessions.read().await;
            let handle = sessions
                .get(session_id)
                .ok_or_else(|| "Session not found".to_string())?;
            handle
                .exec_tx
                .send((command.to_string(), reply_tx))
                .map_err(|_| "Failed to send exec command".to_string())?;
        }

        reply_rx
            .await
            .map_err(|_| "Exec command failed".to_string())?
    }

    pub async fn get_session(&self, session_id: &str) -> Result<SharedSession, String> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|h| h.session.clone())
            .ok_or_else(|| "Session not found".to_string())
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
            .ok_or_else(|| "Session not found".to_string())?;
        handle
            .resize_tx
            .send((cols, rows))
            .map_err(|_| "Failed to send resize command".to_string())
    }
}
