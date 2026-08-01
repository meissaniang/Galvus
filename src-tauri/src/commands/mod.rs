//! Commandes Tauri exposées au frontend (#[tauri::command]). Couche mince : délègue aux services.

use crate::database::{servers_repository, Database};
use crate::errors::AppError;
use crate::models::{Host, Server, ServerInput, SshKey};
use crate::services::terminal::TerminalManager;

/// Liste les hôtes du `~/.ssh/config` (résolus via `ssh -G`).
#[tauri::command]
pub fn list_hosts() -> Result<Vec<Host>, AppError> {
    crate::ssh::config::list_hosts()
}

/// Liste les clés SSH détectées dans `~/.ssh`.
#[tauri::command]
pub fn list_keys() -> Result<Vec<SshKey>, AppError> {
    crate::ssh::keys::list_keys()
}

/// Liste les serveurs enregistrés par l'utilisateur.
#[tauri::command]
pub fn server_list(db: tauri::State<'_, Database>) -> Result<Vec<Server>, AppError> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    servers_repository::list(&conn)
}

/// Crée un serveur.
#[tauri::command]
pub fn server_create(
    db: tauri::State<'_, Database>,
    input: ServerInput,
) -> Result<Server, AppError> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    servers_repository::create(&conn, &input)
}

/// Met à jour un serveur.
#[tauri::command]
pub fn server_update(
    db: tauri::State<'_, Database>,
    id: i64,
    input: ServerInput,
) -> Result<Server, AppError> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    servers_repository::update(&conn, id, &input)
}

/// Supprime un serveur.
#[tauri::command]
pub fn server_delete(db: tauri::State<'_, Database>, id: i64) -> Result<(), AppError> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    servers_repository::delete(&conn, id)
}

/// Ouvre une session terminal SSH interactive (binaire `ssh` système dans un PTY).
///
/// `args` sont les arguments passés à `ssh` (ex. `["vps-meissa-1"]` pour un hôte
/// du config, ou `["-p", "2222", "-i", "/path", "user@host"]` pour un serveur).
#[tauri::command]
pub fn terminal_open(
    app: tauri::AppHandle,
    state: tauri::State<'_, TerminalManager>,
    session_id: String,
    args: Vec<String>,
    cols: u16,
    rows: u16,
) -> Result<(), AppError> {
    state.open(app, session_id, "ssh".to_string(), args, cols, rows)
}

/// Envoie des données (frappes clavier) à une session.
#[tauri::command]
pub fn terminal_write(
    state: tauri::State<'_, TerminalManager>,
    session_id: String,
    data: String,
) -> Result<(), AppError> {
    state.write(&session_id, data.as_bytes())
}

/// Redimensionne le PTY d'une session.
#[tauri::command]
pub fn terminal_resize(
    state: tauri::State<'_, TerminalManager>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), AppError> {
    state.resize(&session_id, cols, rows)
}

/// Ferme une session et tue le processus `ssh`.
#[tauri::command]
pub fn terminal_close(
    state: tauri::State<'_, TerminalManager>,
    session_id: String,
) -> Result<(), AppError> {
    state.close(&session_id)
}
