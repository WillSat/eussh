use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub(crate) struct LocalPtySession {
    pub stdin_tx: mpsc::UnboundedSender<Vec<u8>>,
    pub resize_tx: mpsc::UnboundedSender<(u16, u16)>,
    pub read_handle: JoinHandle<()>,
    pub write_handle: JoinHandle<()>,
    pub resize_handle: JoinHandle<()>,
}

impl LocalPtySession {
    pub fn spawn(
        app_handle: tauri::AppHandle,
        session_id: String,
        cols: u16,
        rows: u16,
    ) -> Result<Self, String> {
        let pty_system = native_pty_system();
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        let pair = pty_system
            .openpty(size)
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        #[cfg(target_os = "windows")]
        let shell = {
            if let Ok(c) = std::env::var("COMSPEC") {
                if !c.is_empty() { c } else { "cmd.exe".into() }
            } else {
                "cmd.exe".into()
            }
        };
        #[cfg(not(target_os = "windows"))]
        let shell = std::env::var("SHELL")
            .unwrap_or_else(|_| "/bin/sh".into());

        let cmd = CommandBuilder::new(shell);
        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone PTY reader: {}", e))?;

        let mut writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to take PTY writer: {}", e))?;

        let master: Arc<Mutex<Box<dyn MasterPty + Send>>> = Arc::new(Mutex::new(pair.master));

        let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (stdout_tx, mut stdout_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (resize_tx, mut resize_rx) = mpsc::unbounded_channel::<(u16, u16)>();

        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        let _ = stdout_tx.send(vec![]);
                        break;
                    }
                    Ok(n) => {
                        if stdout_tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let write_handle = tokio::spawn(async move {
            while let Some(data) = stdin_rx.recv().await {
                let _ = writer.write_all(&data);
                let _ = writer.flush();
            }
        });

        let sid_r = session_id.clone();
        let read_relay = tokio::spawn(async move {
            while let Some(data) = stdout_rx.recv().await {
                if data.is_empty() {
                    break;
                }
                let _ = app_handle.emit(
                    "local-terminal-data",
                    serde_json::json!({ "session_id": sid_r, "data": data }),
                );
            }
            let _ = app_handle.emit(
                "connection-status",
                serde_json::json!({ "session_id": sid_r, "status": "disconnected" }),
            );
        });

        let resize_handle = tokio::spawn(async move {
            while let Some((cols, rows)) = resize_rx.recv().await {
                let m = master.lock().unwrap();
                let _ = m.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                });
            }
        });

        Ok(LocalPtySession {
            stdin_tx,
            resize_tx,
            read_handle: read_relay,
            write_handle,
            resize_handle,
        })
    }
}
