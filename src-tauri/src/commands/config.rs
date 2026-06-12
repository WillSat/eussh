use tauri::State;
use crate::state::AppState;
use crate::models::connection::ConnectionProfile;
use crate::models::config::AppConfig;
use uuid::Uuid;

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    state.config_store.load()
}

#[tauri::command]
pub async fn save_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    state.config_store.update(|existing| {
        if !config.theme.is_empty() {
            existing.theme = config.theme;
        }
        if !config.language.is_empty() {
            existing.language = config.language;
        }
        existing.settings = config.settings;
        Ok(())
    })
}

#[tauri::command]
pub async fn save_connection(
    state: State<'_, AppState>,
    mut profile: ConnectionProfile,
) -> Result<ConnectionProfile, String> {
    state.config_store.update(|config| {
        if profile.id.is_empty() {
            profile.id = Uuid::new_v4().to_string();
        }

        if let Some(existing) = config.connections.iter_mut().find(|c| c.id == profile.id) {
            *existing = profile.clone();
        } else {
            config.connections.push(profile.clone());
        }

        Ok(profile)
    })
}

#[tauri::command]
pub async fn delete_connection(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state.config_store.update(|config| {
        config.connections.retain(|c| c.id != id);
        Ok(())
    })
}
