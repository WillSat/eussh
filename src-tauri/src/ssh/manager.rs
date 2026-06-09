use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{mpsc, Mutex, OnceCell};
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

use crate::models::connection::ConnectionProfile;
use crate::ssh::session::SharedSession;

pub struct SshManager {
    sessions: Arc<Mutex<HashMap<String, SshSessionHandle>>>,
    app_handle: tauri::AppHandle,
    /// Lazily started traffic emitter — can only be started from async context
    emitter_started: OnceCell<()>,
}

pub struct SshSessionHandle {
    stdin_tx: mpsc::UnboundedSender<Vec<u8>>,
    handle: JoinHandle<()>,
    exec_tx: mpsc::UnboundedSender<(String, tokio::sync::oneshot::Sender<Result<String, String>>)>,
    resize_tx: mpsc::UnboundedSender<(u16, u16)>,
    pub session: SharedSession,
    pub upload_bytes: Arc<AtomicU64>,
    pub download_bytes: Arc<AtomicU64>,
}

impl SshManager {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        let sessions: Arc<Mutex<HashMap<String, SshSessionHandle>>> =
            Arc::new(Mutex::new(HashMap::new()));

        Self {
            sessions,
            app_handle,
            emitter_started: OnceCell::new(),
        }
    }

    /// Start the traffic-stats background emitter. Must be called from an async context
    /// (i.e., inside a Tauri command handler) where the Tokio runtime is active.
    /// Safe to call multiple times — only the first call takes effect.
    async fn ensure_traffic_emitter(&self) {
        self.emitter_started.get_or_init(|| async {
            let sessions = self.sessions.clone();
            let app = self.app_handle.clone();
            tokio::spawn(async move {
                let mut tick = interval(Duration::from_secs(1));
                loop {
                    tick.tick().await;
                    let sessions = sessions.lock().await;
                    let mut stats = serde_json::Map::new();
                    for (sid, h) in sessions.iter() {
                        let up = h.upload_bytes.swap(0, Ordering::Relaxed);
                        let down = h.download_bytes.swap(0, Ordering::Relaxed);
                        stats.insert(sid.clone(), serde_json::json!({
                            "upload": up,
                            "download": down,
                        }));
                    }
                    drop(sessions);
                    // Always emit so the frontend can reset to 0 when traffic stops
                    let _ = app.emit("traffic-stats", serde_json::Value::Object(stats));
                }
            });
        }).await;
    }

    pub async fn connect(
        &self,
        profile: ConnectionProfile,
        app_handle: tauri::AppHandle,
    ) -> Result<String, String> {
        // Start the traffic emitter on first connect (async runtime is active here)
        self.ensure_traffic_emitter().await;

        let session_id = uuid::Uuid::new_v4().to_string();

        let (session, stdout_rx) =
            crate::ssh::session::SshSession::connect(profile.clone(), app_handle.clone(), session_id.clone())
                .await?;

        let shared = session
            .session_rx
            .await
            .map_err(|_| "SSH session not established".to_string())?;

        let upload_bytes = Arc::new(AtomicU64::new(0));
        let download_bytes = Arc::new(AtomicU64::new(0));
        let down_clone = download_bytes.clone();

        let mut sessions = self.sessions.lock().await;
        sessions.insert(
            session_id.clone(),
            SshSessionHandle {
                stdin_tx: session.stdin_tx.clone(),
                handle: session.handle,
                exec_tx: session.exec_tx,
                resize_tx: session.resize_tx,
                session: shared,
                upload_bytes,
                download_bytes,
            },
        );

        let app = app_handle.clone();
        let sid = session_id.clone();
        tokio::spawn(async move {
            let mut rx = stdout_rx;
            while let Some(data) = rx.recv().await {
                down_clone.fetch_add(data.len() as u64, Ordering::Relaxed);
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
        let mut sessions = self.sessions.lock().await;
        if let Some(handle) = sessions.remove(session_id) {
            handle.handle.abort();
        }
        Ok(())
    }

    pub async fn write(&self, session_id: &str, data: Vec<u8>) -> Result<(), String> {
        let sessions = self.sessions.lock().await;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| "Session not found".to_string())?;
        handle.upload_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
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
            let sessions = self.sessions.lock().await;
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
        let sessions = self.sessions.lock().await;
        sessions
            .get(session_id)
            .map(|h| h.session.clone())
            .ok_or_else(|| "Session not found".to_string())
    }

    /// Record traffic from file transfers (which open their own channels and bypass
    /// the normal write() / stdout paths).
    pub async fn add_traffic(&self, session_id: &str, upload: u64, download: u64) {
        let sessions = self.sessions.lock().await;
        if let Some(handle) = sessions.get(session_id) {
            if upload > 0 { handle.upload_bytes.fetch_add(upload, Ordering::Relaxed); }
            if download > 0 { handle.download_bytes.fetch_add(download, Ordering::Relaxed); }
        }
    }

    pub async fn resize(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> Result<(), String> {
        let sessions = self.sessions.lock().await;
        let handle = sessions
            .get(session_id)
            .ok_or_else(|| "Session not found".to_string())?;
        handle
            .resize_tx
            .send((cols, rows))
            .map_err(|_| "Failed to send resize command".to_string())
    }
}
