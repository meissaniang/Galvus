// Modules du backend Galvus (voir chaque `mod.rs` pour la responsabilité).
pub mod commands;
pub mod config;
pub mod database;
pub mod errors;
pub mod models;
pub mod security;
pub mod services;
pub mod ssh;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_hosts,
            commands::list_keys
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
