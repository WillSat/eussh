use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};

use crate::models::connection::ConnectionProfile;
use crate::ssh::host_key::HostKeyVerificationManager;
use crate::ssh::session::SharedSession;

const DEFAULT_EXEC_TIMEOUT: Duration = Duration::from_secs(30);
const PING_COOLDOWN_MS: u64 = 2_000; // minimum 2s between pings per session

pub struct SshManager {
    sessions: Arc<RwLock<HashMap<String, SshSessionHandle>>>,
    pub host_key_verification: Arc<HostKeyVerificationManager>,
}

pub struct SshSessionHandle {
    stdin_tx: mpsc::UnboundedSender<Vec<u8>>,
    handle: JoinHandle<()>,
    exec_tx: mpsc::UnboundedSender<(String, tokio::sync::oneshot::Sender<Result<String, String>>)>,
    resize_tx: mpsc::UnboundedSender<(u16, u16)>,
    pub session: SharedSession,
    last_ping_ms: AtomicU64,
}

impl SshManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            host_key_verification: Arc::new(HostKeyVerificationManager::new()),
        }
    }

    pub async fn connect(
        &self,
        profile: ConnectionProfile,
        app_handle: tauri::AppHandle,
    ) -> Result<String, String> {
        let session_id = uuid::Uuid::new_v4().to_string();

        let _ = app_handle.emit("debug-event", serde_json::json!({
            "session_id": session_id,
            "level": "info",
            "source": "Manager",
            "message": format!("Connecting to {}@{}:{}", profile.username, profile.host, profile.port),
        }));

        let (session, stdout_rx) =
            crate::ssh::session::SshSession::connect(profile.clone(), app_handle.clone(), session_id.clone(), self.host_key_verification.clone())
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
                last_ping_ms: AtomicU64::new(0),
            },
        );
        let session_count = sessions.len();

        let _ = app_handle.emit("debug-event", serde_json::json!({
            "session_id": session_id,
            "level": "info",
            "source": "Manager",
            "message": format!("Session registered ({} active)", session_count),
        }));

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
        // Note: disconnect debug events are emitted by session.rs spawn
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

        timeout(DEFAULT_EXEC_TIMEOUT, reply_rx)
            .await
            .map_err(|_| format!("Exec command timed out after {}s", DEFAULT_EXEC_TIMEOUT.as_secs()))?
            .map_err(|_| "Exec command failed".to_string())?
    }

    pub async fn get_session(&self, session_id: &str) -> Result<SharedSession, String> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|h| h.session.clone())
            .ok_or_else(|| "Session not found".to_string())
    }

    /// Rate-limited ping gate. Returns Ok(()) if a ping is allowed now,
    /// Err("rate-limited") if the session was pinged less than PING_COOLDOWN_MS ago.
    pub async fn try_claim_ping(&self, session_id: &str) -> Result<(), String> {
        let now_ms = {
            // Avoid calling Instant::now() in a non-trivial way in statics —
            // we compute elapsed from a base Instant each time, but for the
            // atomic store we use epoch millis via SystemTime as a monotonic proxy.
            // In practice, std Instant is monotonic but not storable atomically.
            // We store elapsed from process start instead.
            use std::time::SystemTime;
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        };
        let sessions = self.sessions.read().await;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| "Session not found".to_string())?;
        let last = handle.last_ping_ms.swap(now_ms, Ordering::AcqRel);
        if last > 0 && now_ms.saturating_sub(last) < PING_COOLDOWN_MS {
            return Err("rate-limited".to_string());
        }
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
            .ok_or_else(|| "Session not found".to_string())?;
        handle
            .resize_tx
            .send((cols, rows))
            .map_err(|_| "Failed to send resize command".to_string())
    }
}
