use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use rand::RngCore;
use ring::pbkdf2;

const SALT: &[u8] = b"eussh-config-salt-v1";
const PBKDF2_ITERATIONS: u32 = 100_000;
const NONCE_SIZE: usize = 12;

fn get_machine_id() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("reg")
            .args([
                "query",
                r"HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography",
                "/v",
                "MachineGuid",
            ])
            .output()
            .map_err(|e| format!("Failed to get machine GUID: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("MachineGuid") {
                if let Some(guid) = line.split("REG_SZ").nth(1) {
                    return Ok(guid.trim().to_string());
                }
            }
        }
        Err("Could not find MachineGuid".into())
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("ioreg")
            .args(["-d2", "-c", "IOPlatformExpertDevice"])
            .output()
            .map_err(|e| format!("Failed to get platform UUID: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("IOPlatformUUID") {
                if let Some(uuid) = line.split('"').nth(3) {
                    return Ok(uuid.to_string());
                }
            }
        }
        Err("Could not find platform UUID".into())
    }

    #[cfg(target_os = "linux")]
    {
        for path in &["/etc/machine-id", "/var/lib/dbus/machine-id"] {
            if let Ok(id) = std::fs::read_to_string(path) {
                return Ok(id.trim().to_string());
            }
        }
        // Fallback: hostname
        if let Ok(hostname) = std::process::Command::new("hostname").output() {
            return Ok(String::from_utf8_lossy(&hostname.stdout).trim().to_string());
        }
        Err("Could not determine machine ID".into())
    }
}

fn derive_key() -> Result<[u8; 32], String> {
    let machine_id = get_machine_id()?;
    let mut key = [0u8; 32];

    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(PBKDF2_ITERATIONS).unwrap(),
        SALT,
        machine_id.as_bytes(),
        &mut key,
    );

    Ok(key)
}

pub fn encrypt_config(plaintext: &str) -> Result<String, String> {
    let key = derive_key()?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Invalid key: {}", e))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(base64::engine::general_purpose::STANDARD.encode(&combined))
}

pub fn decrypt_config(encrypted: &str) -> Result<String, String> {
    let key = derive_key()?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Invalid key: {}", e))?;

    let combined = base64::engine::general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    if combined.len() < NONCE_SIZE {
        return Err("Invalid encrypted data".into());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))
}
