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
use tauri_plugin_log::{Target, TargetKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Logs : niveaux + rotation (fichier 5 Mo max) + sortie standard.
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .max_file_size(5_000_000)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("galvus".into()),
                    }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .manage(services::terminal::TerminalManager::default())
        .manage(services::tunnels::TunnelManager::default())
        .setup(|app| {
            // Mécanisme de mise à jour signé (desktop uniquement).
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            // Coffre natif -> clé de chiffrement -> ouverture de la base chiffrée.
            let key = security::vault::get_or_create_db_key()?;
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let database = database::Database::initialize(&data_dir.join("galvus.db"), &key)?;
            app.manage(database);
            log::info!("Galvus démarré, base initialisée dans {data_dir:?}");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_hosts,
            commands::list_keys,
            commands::server_list,
            commands::server_create,
            commands::server_update,
            commands::server_delete,
            commands::tunnel_list,
            commands::tunnel_create,
            commands::tunnel_delete,
            commands::tunnel_start,
            commands::tunnel_stop,
            commands::tunnel_running,
            commands::terminal_open,
            commands::terminal_write,
            commands::terminal_resize,
            commands::terminal_close
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
