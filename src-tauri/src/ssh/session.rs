use russh::client::{self, Handler};
use russh::keys::PrivateKeyWithHashAlg;
use russh::keys::ssh_key::{HashAlg, PublicKey};
use russh::{Channel, ChannelMsg};
use std::pin::Pin;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;

use crate::models::connection::{AuthMethod, ConnectionProfile};

pub(crate) struct ClientHandler;

impl Handler for ClientHandler {
    type Error = russh::Error;

    #[allow(refining_impl_trait)]
    fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<bool, Self::Error>> + Send + '_>> {
        Box::pin(async { Ok(true) })
    }
}

pub type SharedSession = Arc<Mutex<russh::client::Handle<ClientHandler>>>;

pub struct SshSession {
    pub stdin_tx: mpsc::UnboundedSender<Vec<u8>>,
    pub handle: JoinHandle<()>,
    pub exec_tx: mpsc::UnboundedSender<(String, tokio::sync::oneshot::Sender<Result<String, String>>)>,
    pub resize_tx: mpsc::UnboundedSender<(u16, u16)>,
    pub session_rx: tokio::sync::oneshot::Receiver<SharedSession>,
}

impl SshSession {
    pub async fn connect(
        profile: ConnectionProfile,
        app_handle: tauri::AppHandle,
        session_id: String,
    ) -> Result<(Self, mpsc::UnboundedReceiver<Vec<u8>>), String> {
        let profile_clone = profile.clone();

        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (stdout_tx, stdout_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (exec_tx, mut exec_rx) = mpsc::unbounded_channel::<(String, tokio::sync::oneshot::Sender<Result<String, String>>)>();
        let (resize_tx, mut resize_rx) = mpsc::unbounded_channel::<(u16, u16)>();

        let (session_tx, session_rx) = tokio::sync::oneshot::channel();

        let app = app_handle.clone();

        let handle = tokio::spawn(async move {
            let result = async {
                let addr = format!("{}:{}", profile_clone.host, profile_clone.port);
                let config = Arc::new(client::Config::default());

                let resolved = tokio::net::lookup_host(&addr)
                    .await
                    .map_err(|e| format!("DNS lookup failed: {}", e))?
                    .next()
                    .ok_or_else(|| format!("No address found for {}", addr))?;

                let mut session = client::connect(config, resolved, ClientHandler)
                    .await
                    .map_err(|e| format!("SSH connection failed: {}", e))?;

                match &profile_clone.auth_method {
                    AuthMethod::Password { password } => {
                        session
                            .authenticate_password(&profile_clone.username, password)
                            .await
                            .map_err(|e| format!("Password auth failed: {}", e))?;
                    }
                    AuthMethod::PrivateKey {
                        private_key_path,
                        passphrase,
                    } => {
                        let key_pair = russh::keys::load_secret_key(
                            private_key_path,
                            passphrase.as_deref(),
                        )
                        .map_err(|e| format!("Failed to load private key: {}", e))?;

                        let key_with_hash = PrivateKeyWithHashAlg::new(
                            Arc::new(key_pair),
                            Some(HashAlg::Sha512),
                        );

                        session
                            .authenticate_publickey(
                                &profile_clone.username,
                                key_with_hash,
                            )
                            .await
                            .map_err(|e| format!("Public key auth failed: {}", e))?;
                    }
                }

                let channel: Channel<client::Msg> = session
                    .channel_open_session()
                    .await
                    .map_err(|e| format!("Channel open failed: {}", e))?;

                channel
                    .request_pty(true, "xterm-256color", 80, 24, 0, 0, &[])
                    .await
                    .map_err(|e| format!("PTY request failed: {}", e))?;

                channel
                    .request_shell(true)
                    .await
                    .map_err(|e| format!("Shell start failed: {}", e))?;

                Ok::<_, String>((session, channel))
            };

            let (session, channel) = match result.await {
                Ok(v) => v,
                Err(e) => {
                    let _ = app.emit(
                        "connection-status",
                        serde_json::json!({
                            "session_id": session_id,
                            "status": "error",
                            "message": e,
                        }),
                    );
                    return;
                }
            };

            let _ = app.emit(
                "connection-status",
                serde_json::json!({
                    "session_id": session_id.clone(),
                    "status": "connected",
                }),
            );

            let session = Arc::new(Mutex::new(session));
            let _ = session_tx.send(session.clone());

            // Spawn stdin writer
            let (mut read_half, write_half) = channel.split();
            let write_half = Arc::new(Mutex::new(write_half));

            let wh = write_half.clone();
            tokio::spawn(async move {
                while let Some(data) = stdin_rx.recv().await {
                    let writer = wh.lock().await;
                    let cursor = std::io::Cursor::new(data);
                    if writer.data(cursor).await.is_err() {
                        break;
                    }
                }
            });

            // Spawn resize handler
            let wh_resize = write_half.clone();
            tokio::spawn(async move {
                while let Some((cols, rows)) = resize_rx.recv().await {
                    let writer = wh_resize.lock().await;
                    if writer.window_change(cols as u32, rows as u32, 0, 0).await.is_err() {
                        break;
                    }
                }
            });

            // Spawn exec command handler
            // Opens a new channel per command and releases the session lock immediately
            // so that concurrent exec commands (ping, monitor, file ops) don't block each other.
            let exec_session = session.clone();
            tokio::spawn(async move {
                while let Some((cmd, reply)) = exec_rx.recv().await {
                    let result = async {
                        let mut exec_channel = {
                            let sess = exec_session.lock().await;
                            sess.channel_open_session()
                                .await
                                .map_err(|e| format!("exec channel failed: {}", e))?
                        }; // session lock released here — other execs can now proceed
                        exec_channel
                            .exec(true, cmd.as_bytes())
                            .await
                            .map_err(|e| format!("exec failed: {}", e))?;

                        let mut output = Vec::new();
                        loop {
                            match exec_channel.wait().await {
                                Some(ChannelMsg::Data { data }) => {
                                    output.extend_from_slice(&data);
                                }
                                Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) => break,
                                None => break,
                                _ => {}
                            }
                        }
                        String::from_utf8(output).map_err(|e| format!("utf8: {}", e))
                    };
                    let _ = reply.send(result.await);
                }
            });

            // Terminal read loop
            loop {
                match read_half.wait().await {
                    Some(ChannelMsg::Data { data }) => {
                        if stdout_tx.send(data.to_vec()).is_err() {
                            break;
                        }
                    }
                    Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) => break,
                    None => break,
                    _ => {}
                }
            }

            let _ = app.emit(
                "connection-status",
                serde_json::json!({
                    "session_id": session_id,
                    "status": "disconnected",
                }),
            );
        });

        Ok((
            Self {
                stdin_tx: stdin_tx.clone(),
                handle,
                exec_tx,
                resize_tx,
                session_rx,
            },
            stdout_rx,
        ))
    }
}
