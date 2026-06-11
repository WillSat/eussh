use std::sync::Arc;
use crate::ssh::manager::SshManager;
use crate::storage::config_store::ConfigStore;

pub struct AppState {
    pub config_store: ConfigStore,
    pub ssh_manager: Arc<SshManager>,
}
