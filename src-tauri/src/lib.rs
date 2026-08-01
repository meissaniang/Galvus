// Modules du backend Galvus (voir chaque `mod.rs` pour la responsabilité).
pub mod commands;
pub mod config;
pub mod database;
pub mod errors;
pub mod models;
pub mod security;
pub mod services;
pub mod ssh;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(services::terminal::TerminalManager::default())
        .setup(|app| {
            // Coffre natif -> clé de chiffrement -> ouverture de la base chiffrée.
            let key = security::vault::get_or_create_db_key()?;
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let database =
                database::Database::initialize(&data_dir.join("galvus.db"), &key)?;
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_hosts,
            commands::list_keys,
            commands::server_list,
            commands::server_create,
            commands::server_update,
            commands::server_delete,
            commands::terminal_open,
            commands::terminal_write,
            commands::terminal_resize,
            commands::terminal_close
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
