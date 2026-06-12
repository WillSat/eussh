use russh::client::{self, Handler};
use russh::keys::PrivateKeyWithHashAlg;
use russh::keys::ssh_key::{HashAlg, PublicKey};
use russh::{Channel, ChannelMsg};
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Emitter;
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::timeout;

const EXEC_TIMEOUT: Duration = Duration::from_secs(30);
const READ_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

use crate::models::connection::{AuthMethod, ConnectionProfile};
use crate::ssh::host_key::HostKeyVerificationManager;

pub(crate) struct ClientHandler {
    pub app_handle: tauri::AppHandle,
    pub host: String,
    pub session_id: String,
    pub host_key_verification: Arc<HostKeyVerificationManager>,
}

impl Handler for ClientHandler {
    type Error = russh::Error;

    #[allow(refining_impl_trait)]
    fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<bool, Self::Error>> + Send + '_>> {
        let fingerprint = server_public_key.fingerprint(HashAlg::Sha256).to_string();
        let host = self.host.clone();
        let session_id = self.session_id.clone();
        let verif = self.host_key_verification.clone();
        let app = self.app_handle.clone();

        Box::pin(async move {
            // Already trusted — accept immediately
            if verif.is_known(&host, &fingerprint) {
                let short_fp = if fingerprint.len() > 30 {
                    &fingerprint[..30]
                } else {
                    &fingerprint
                };
                let _ = app.emit("debug-event", serde_json::json!({
                    "session_id": session_id,
                    "level": "info",
                    "source": "SSH::HostKey",
                    "message": format!("Host key verified from known_hosts: {}", short_fp),
                }));
                return Ok(true);
            }

            let is_key_changed = verif.get_known_fingerprint(&host).is_some();

            if is_key_changed {
                let _ = app.emit("debug-event", serde_json::json!({
                    "session_id": session_id,
                    "level": "warn",
                    "source": "SSH::HostKey",
                    "message": format!("Host key CHANGED for {} — prompting user", host),
                }));
            }

            // Create pending verification and prompt the user
            let (request_id, rx) = verif.create_pending(host.clone(), fingerprint.clone()).await;

            let _ = app.emit("host-key-verify", serde_json::json!({
                "requestId": request_id,
                "host": host,
                "fingerprint": fingerprint,
                "sessionId": session_id,
                "isKeyChanged": is_key_changed,
            }));

            // Wait for user response; treat cancellation as rejection
            match timeout(Duration::from_secs(60), rx).await {
                Ok(Ok(true)) => {
                    let _ = app.emit("debug-event", serde_json::json!({
                        "session_id": session_id,
                        "level": "info",
                        "source": "SSH::HostKey",
                        "message": "User accepted host key",
                    }));
                    Ok(true)
                }
                Ok(Ok(false)) | Ok(Err(_)) => {
                    let _ = app.emit("debug-event", serde_json::json!({
                        "session_id": session_id,
                        "level": "warn",
                        "source": "SSH::HostKey",
                        "message": "User rejected host key",
                    }));
                    Ok(false)
                }
                Err(_) => {
                    verif.cancel(&request_id).await;
                    let _ = app.emit("debug-event", serde_json::json!({
                        "session_id": session_id,
                        "level": "warn",
                        "source": "SSH::HostKey",
                        "message": "Host key verification timed out",
                    }));
                    Ok(false)
                }
            }
        })
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
        host_key_verification: Arc<HostKeyVerificationManager>,
    ) -> Result<(Self, mpsc::UnboundedReceiver<Vec<u8>>), String> {
        let profile_clone = profile.clone();

        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (stdout_tx, stdout_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (exec_tx, mut exec_rx) = mpsc::unbounded_channel::<(String, tokio::sync::oneshot::Sender<Result<String, String>>)>();
        let (resize_tx, mut resize_rx) = mpsc::unbounded_channel::<(u16, u16)>();

        let (session_tx, session_rx) = tokio::sync::oneshot::channel();

        let app = app_handle.clone();

        let handle = tokio::spawn(async move {
            let t_start = Instant::now();
            let result = async {
                let addr = format!("{}:{}", profile_clone.host, profile_clone.port);
                let mut config = client::Config::default();
                // Use a standard OpenSSH client ID to avoid DPI/filtering blocking
                // the default "SSH-2.0-russh_0.61" string (relevant behind restrictive firewalls).
                config.client_id = russh::SshId::Standard(std::borrow::Cow::Borrowed(
                    "SSH-2.0-OpenSSH_9.9",
                ));
                let config = Arc::new(config);

                // ── DNS ──────────────────────────────────────────
                let t_dns = Instant::now();
                let resolved = tokio::net::lookup_host(&addr)
                    .await
                    .map_err(|e| format!("DNS lookup failed: {}", e))?
                    .next()
                    .ok_or_else(|| format!("No address found for {}", addr))?;
                let _ = app.emit("debug-event", serde_json::json!({
                    "session_id": session_id,
                    "level": "info",
                    "source": "SSH::DNS",
                    "message": format!("Resolved {} → {}", addr, resolved),
                    "elapsed_ms": t_dns.elapsed().as_millis(),
                }));

                // ── TCP connect ──────────────────────────────────
                let t_tcp = Instant::now();
                let stream = match timeout(Duration::from_secs(10), tokio::net::TcpStream::connect(resolved)).await {
                    Ok(Ok(s)) => s,
                    Ok(Err(e)) => return Err(format!("TCP connect to {} failed: {}", resolved, e)),
                    Err(_) => return Err(format!("TCP connect to {} timed out after 10s", resolved)),
                };
                let _ = app.emit("debug-event", serde_json::json!({
                    "session_id": session_id,
                    "level": "info",
                    "source": "SSH::TCP",
                    "message": format!("TCP connected to {}", resolved),
                    "elapsed_ms": t_tcp.elapsed().as_millis(),
                }));

                // ── SSH handshake ────────────────────────────────
                let t_ssh = Instant::now();
                let handler = ClientHandler {
                    app_handle: app.clone(),
                    host: format!("{}:{}", profile_clone.host, profile_clone.port),
                    session_id: session_id.clone(),
                    host_key_verification: host_key_verification.clone(),
                };
                let mut session = match timeout(Duration::from_secs(15), client::connect_stream(config.clone(), stream, handler)).await {
                    Ok(Ok(s)) => s,
                    Ok(Err(e)) => return Err(format!("SSH handshake failed: {}", e)),
                    Err(_) => return Err(format!("SSH handshake timed out after 15s")),
                };
                let _ = app.emit("debug-event", serde_json::json!({
                    "session_id": session_id,
                    "level": "info",
                    "source": "SSH::Handshake",
                    "message": "SSH handshake complete",
                    "elapsed_ms": t_ssh.elapsed().as_millis(),
                }));

                // ── Authentication ──────────────────────────────
                let t_auth = Instant::now();
                let auth_method_name: &str;
                match &profile_clone.auth_method {
                    AuthMethod::Password { password } => {
                        auth_method_name = "password";
                        session
                            .authenticate_password(&profile_clone.username, password)
                            .await
                            .map_err(|e| format!("Password auth failed: {}", e))?;
                    }
                    AuthMethod::PrivateKey {
                        private_key_path,
                        passphrase,
                    } => {
                        auth_method_name = "publickey";
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
                let _ = app.emit("debug-event", serde_json::json!({
                    "session_id": session_id,
                    "level": "info",
                    "source": "SSH::Auth",
                    "message": format!("Authenticated as {} ({})", profile_clone.username, auth_method_name),
                    "elapsed_ms": t_auth.elapsed().as_millis(),
                }));

                // ── Channel open ────────────────────────────────
                let t_chan = Instant::now();
                let channel: Channel<client::Msg> = session
                    .channel_open_session()
                    .await
                    .map_err(|e| format!("Channel open failed: {}", e))?;
                let _ = app.emit("debug-event", serde_json::json!({
                    "session_id": session_id,
                    "level": "info",
                    "source": "SSH::Channel",
                    "message": "Channel opened",
                    "elapsed_ms": t_chan.elapsed().as_millis(),
                }));

                // ── PTY ─────────────────────────────────────────
                let t_pty = Instant::now();
                channel
                    .request_pty(true, "xterm-256color", 80, 24, 0, 0, &[])
                    .await
                    .map_err(|e| format!("PTY request failed: {}", e))?;
                let _ = app.emit("debug-event", serde_json::json!({
                    "session_id": session_id,
                    "level": "info",
                    "source": "SSH::PTY",
                    "message": "PTY allocated (80x24 xterm-256color)",
                    "elapsed_ms": t_pty.elapsed().as_millis(),
                }));

                // ── Shell ───────────────────────────────────────
                let t_shell = Instant::now();
                channel
                    .request_shell(true)
                    .await
                    .map_err(|e| format!("Shell start failed: {}", e))?;
                let _ = app.emit("debug-event", serde_json::json!({
                    "session_id": session_id,
                    "level": "info",
                    "source": "SSH::Shell",
                    "message": "Shell started",
                    "elapsed_ms": t_shell.elapsed().as_millis(),
                }));

                let total_ms = t_start.elapsed().as_millis();
                let _ = app.emit("debug-event", serde_json::json!({
                    "session_id": session_id,
                    "level": "info",
                    "source": "SSH",
                    "message": format!("Session fully established in {}ms", total_ms),
                    "elapsed_ms": total_ms,
                }));

                Ok::<_, String>((session, channel))
            };

            let (session, channel) = match result.await {
                Ok(v) => v,
                Err(e) => {
                    let total_ms = t_start.elapsed().as_millis();
                    let _ = app.emit("debug-event", serde_json::json!({
                        "session_id": session_id,
                        "level": "error",
                        "source": "SSH",
                        "message": format!("Connection failed after {}ms: {}", total_ms, e),
                        "elapsed_ms": total_ms,
                    }));
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

            // Spawn exec command handler — concurrent with semaphore (max 5)
            // Each incoming command spawns a tokio task; the semaphore bounds
            // concurrent SSH channel usage per session.
            let exec_session = session.clone();
            let exec_semaphore = Arc::new(Semaphore::new(5));
            tokio::spawn(async move {
                while let Some((cmd, reply)) = exec_rx.recv().await {
                    let sess = exec_session.clone();
                    let sem = exec_semaphore.clone();
                    tokio::spawn(async move {
                        let result = timeout(EXEC_TIMEOUT, async {
                            let _permit = sem.acquire().await;
                            let mut exec_channel = {
                                let sess = sess.lock().await;
                                sess.channel_open_session()
                                    .await
                                    .map_err(|e| format!("exec channel failed: {}", e))?
                            }; // session lock released — other execs can now proceed
                            exec_channel
                                .exec(true, cmd.as_bytes())
                                .await
                                .map_err(|e| format!("exec failed: {}", e))?;

                            let mut output = Vec::new();
                            let mut exit_status: Option<u32> = None;
                            loop {
                                match exec_channel.wait().await {
                                    Some(ChannelMsg::Data { data }) => {
                                        output.extend_from_slice(&data);
                                    }
                                    Some(ChannelMsg::ExitStatus { exit_status: status }) => {
                                        exit_status = Some(status);
                                    }
                                    Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) => break,
                                    None => break,
                                    _ => {}
                                }
                            }
                            let text = String::from_utf8(output).map_err(|e| format!("utf8: {}", e))?;
                            match exit_status {
                                Some(0) | None => Ok(text),
                                Some(code) => Err(format!("Command exited with code {}", code)),
                            }
                        }).await;
                        let result = match result {
                            Ok(result) => result,
                            Err(_) => Err(format!("exec timed out after {}s", EXEC_TIMEOUT.as_secs())),
                        };
                        let _ = reply.send(result);
                    });
                }
            });

            // Terminal read loop – timeout ensures we don't hang forever
            // if the SSH connection drops without clean Eof/Close.
            loop {
                match timeout(READ_IDLE_TIMEOUT, read_half.wait()).await {
                    Ok(Some(ChannelMsg::Data { data })) => {
                        if stdout_tx.send(data.to_vec()).is_err() {
                            break;
                        }
                    }
                    Ok(Some(ChannelMsg::Eof)) | Ok(Some(ChannelMsg::Close)) => break,
                    Ok(None) => break,
                    Err(_) => {
                        // Idle timeout — connection likely dead
                        let _ = app.emit("debug-event", serde_json::json!({
                            "session_id": session_id,
                            "level": "warn",
                            "source": "SSH::ReadLoop",
                            "message": format!("No terminal data for {}s — closing session", READ_IDLE_TIMEOUT.as_secs()),
                        }));
                        break;
                    }
                    Ok(Some(_)) => {}
                }
            }

            let total_life = t_start.elapsed().as_millis();
            let _ = app.emit("debug-event", serde_json::json!({
                "session_id": session_id,
                "level": "info",
                "source": "SSH",
                "message": format!("Session disconnected (lifetime: {}ms)", total_life),
            }));
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
