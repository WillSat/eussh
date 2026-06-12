use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use tokio::sync::{Mutex, oneshot};

/// Manages the known_hosts file and pending host-key verification requests.
///
/// The known_hosts file is stored alongside the encrypted config at
/// `{config_dir}/eussh/known_hosts` using one entry per line:
///
/// ```text
/// <host:port> <fingerprint>
/// ```
///
/// Where `<fingerprint>` is the output of `PublicKey::fingerprint(HashAlg::Sha256)`,
/// e.g. `SHA256:abc123...`.
pub struct HostKeyVerificationManager {
    known_hosts_path: PathBuf,
    pending: Mutex<HashMap<String, PendingVerification>>,
}

struct PendingVerification {
    host: String,
    fingerprint: String,
    sender: oneshot::Sender<bool>,
}

impl HostKeyVerificationManager {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("eussh");
        fs::create_dir_all(&config_dir).ok();
        Self {
            known_hosts_path: config_dir.join("known_hosts"),
            pending: Mutex::new(HashMap::new()),
        }
    }

    // ── known_hosts helpers ──────────────────────────────────────────

    /// Returns `true` when a line `<host:port> <fingerprint>` already exists.
    pub fn is_known(&self, host: &str, fingerprint: &str) -> bool {
        let file = match fs::File::open(&self.known_hosts_path) {
            Ok(f) => f,
            Err(_) => return false,
        };
        for line in BufReader::new(file).lines().flatten() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((h, fp)) = line.split_once(' ') {
                if h == host && fp.trim() == fingerprint {
                    return true;
                }
            }
        }
        false
    }

    /// Returns the fingerprint stored for this host, if any.
    /// Used to detect key changes (host known but fingerprint differs).
    pub fn get_known_fingerprint(&self, host: &str) -> Option<String> {
        let file = fs::File::open(&self.known_hosts_path).ok()?;
        for line in BufReader::new(file).lines().flatten() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((h, fp)) = line.split_once(' ') {
                if h == host {
                    return Some(fp.trim().to_string());
                }
            }
        }
        None
    }

    fn save_to_known_hosts(&self, host: &str, fingerprint: &str) -> Result<(), String> {
        // Read existing entries, replacing any line for the same host
        let mut entries: Vec<String> = Vec::new();
        let mut replaced = false;

        if let Ok(file) = fs::File::open(&self.known_hosts_path) {
            for line in BufReader::new(file).lines().flatten() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    entries.push(line);
                    continue;
                }
                if let Some((h, _fp)) = trimmed.split_once(' ') {
                    if h == host {
                        entries.push(format!("{} {}", host, fingerprint));
                        replaced = true;
                        continue;
                    }
                }
                entries.push(line);
            }
        }
        if !replaced {
            entries.push(format!("{} {}", host, fingerprint));
        }

        let mut out = String::new();
        for e in &entries {
            out.push_str(e);
            out.push('\n');
        }
        fs::write(&self.known_hosts_path, &out)
            .map_err(|e| format!("Failed to write known_hosts: {}", e))
    }

    // ── pending verification ─────────────────────────────────────────

    /// Creates a pending verification entry and returns the request ID + receiver.
    pub async fn create_pending(
        &self,
        host: String,
        fingerprint: String,
    ) -> (String, oneshot::Receiver<bool>) {
        let request_id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(
            request_id.clone(),
            PendingVerification {
                host,
                fingerprint,
                sender: tx,
            },
        );
        (request_id, rx)
    }

    /// Called by the frontend to respond to a verification prompt.
    /// If `accepted` and `remember` are both true, the key is persisted to known_hosts.
    pub async fn confirm(
        &self,
        request_id: &str,
        accepted: bool,
        remember: bool,
    ) -> Result<(), String> {
        let mut pending = self.pending.lock().await;
        let p = pending
            .remove(request_id)
            .ok_or_else(|| "Verification request not found".to_string())?;

        if accepted && remember {
            self.save_to_known_hosts(&p.host, &p.fingerprint)?;
        }
        // If the receiver is gone (user closed window, etc.) the send errors
        // silently — that's fine.
        let _ = p.sender.send(accepted);
        Ok(())
    }

    pub async fn cancel(&self, request_id: &str) {
        let mut pending = self.pending.lock().await;
        if let Some(p) = pending.remove(request_id) {
            let _ = p.sender.send(false);
        }
    }
}
