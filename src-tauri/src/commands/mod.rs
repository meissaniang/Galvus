//! Commandes Tauri exposées au frontend (#[tauri::command]). Couche mince : délègue aux services.

use crate::errors::AppError;
use crate::models::{Host, SshKey};

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
