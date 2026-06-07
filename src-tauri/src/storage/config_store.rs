use std::path::PathBuf;
use crate::models::config::AppConfig;
use crate::storage::encrypt;

pub struct ConfigStore {
    config_path: PathBuf,
}

impl ConfigStore {
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("eussh");
        std::fs::create_dir_all(&config_dir).ok();
        let config_path = config_dir.join("config.enc.json");
        Self { config_path }
    }

    pub fn load(&self) -> Result<AppConfig, String> {
        if !self.config_path.exists() {
            return Ok(AppConfig::default());
        }
        let encrypted = std::fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Read error: {}", e))?;
        let plaintext = encrypt::decrypt_config(&encrypted)?;
        serde_json::from_str(&plaintext).map_err(|e| format!("Parse error: {}", e))
    }

    pub fn save(&self, config: &AppConfig) -> Result<(), String> {
        let plaintext = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Serialize error: {}", e))?;
        let encrypted = encrypt::encrypt_config(&plaintext)?;
        std::fs::write(&self.config_path, &encrypted)
            .map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

}
