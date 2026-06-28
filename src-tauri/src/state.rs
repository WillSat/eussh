use std::sync::Arc;
use crate::ssh::manager::SshManager;
use crate::storage::config_store::ConfigStore;
use crate::terminal::manager::LocalTerminalManager;

pub struct AppState {
    pub config_store: ConfigStore,
    pub ssh_manager: Arc<SshManager>,
    pub local_terminal_manager: Arc<LocalTerminalManager>,
}
