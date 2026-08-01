//! Commandes Tauri exposées au frontend (#[tauri::command]). Couche mince : délègue aux services.

use crate::errors::AppError;
use crate::models::{Host, SshKey};
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

/// Ouvre une session terminal SSH interactive (binaire `ssh` système dans un PTY).
#[tauri::command]
pub fn terminal_open(
    app: tauri::AppHandle,
    state: tauri::State<'_, TerminalManager>,
    session_id: String,
    host: String,
    cols: u16,
    rows: u16,
) -> Result<(), AppError> {
    state.open(app, session_id, "ssh".to_string(), vec![host], cols, rows)
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
