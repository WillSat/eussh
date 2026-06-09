#![windows_subsystem = "windows"]

mod state;
mod commands;
mod ssh;
mod storage;
mod models;

use state::AppState;
use commands::{config, connection, file};
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_state = AppState {
                config_store: storage::config_store::ConfigStore::new(),
                ssh_manager: ssh::manager::SshManager::new(app.handle().clone()),
            };
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::get_config,
            config::save_config,
            config::save_connection,
            config::delete_connection,
            connection::connect,
            connection::disconnect,
            connection::terminal_write,
            connection::terminal_resize,
            connection::exec_command,
            connection::ping,
            connection::clipboard_read,
            connection::clipboard_write,
            file::file_list,
            file::file_mkdir,
            file::file_remove,
            file::file_rename,
            file::file_copy,
            file::file_exists,
            file::file_read,
            file::file_write,
            file::file_download_dir,
            file::file_upload_path,
            file::file_chmod,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
