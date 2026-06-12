use std::path::PathBuf;
use std::sync::Mutex;
use crate::models::config::AppConfig;
use crate::storage::encrypt;

pub struct ConfigStore {
    config_path: PathBuf,
    lock: Mutex<()>,
}

impl ConfigStore {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("eussh");
        std::fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("config.enc.json");
        Self { config_path, lock: Mutex::new(()) }
    }

    pub fn load(&self) -> Result<AppConfig, String> {
        let _guard = self.lock.lock().map_err(|_| "Config lock poisoned".to_string())?;
        self.load_unlocked()
    }

    pub fn save(&self, config: &AppConfig) -> Result<(), String> {
        let _guard = self.lock.lock().map_err(|_| "Config lock poisoned".to_string())?;
        self.save_unlocked(config)
    }

    pub fn update<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&mut AppConfig) -> Result<T, String>,
    {
        let _guard = self.lock.lock().map_err(|_| "Config lock poisoned".to_string())?;
        let mut config = self.load_unlocked()?;
        let result = f(&mut config)?;
        self.save_unlocked(&config)?;
        Ok(result)
    }

    fn load_unlocked(&self) -> Result<AppConfig, String> {
        if !self.config_path.exists() {
            return Ok(AppConfig::default());
        }
        let encrypted = std::fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Read error: {}", e))?;
        let plaintext = encrypt::decrypt_config(&encrypted)?;
        serde_json::from_str(&plaintext).map_err(|e| format!("Parse error: {}", e))
    }

    fn save_unlocked(&self, config: &AppConfig) -> Result<(), String> {
        let plaintext = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Serialize error: {}", e))?;
        let encrypted = encrypt::encrypt_config(&plaintext)?;

        let tmp_path = self.config_path.with_extension("enc.json.tmp");
        std::fs::write(&tmp_path, &encrypted)
            .map_err(|e| format!("Write error: {}", e))?;
        // On Windows, rename() requires the target not to exist — remove first.
        // On Unix, rename() is atomic and overwrites; remove_file is a no-harm no-op.
        let _ = std::fs::remove_file(&self.config_path);
        std::fs::rename(&tmp_path, &self.config_path)
            .map_err(|e| format!("Replace error: {}", e))?;
        Ok(())
    }
}
