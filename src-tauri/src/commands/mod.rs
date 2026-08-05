//! Commandes Tauri exposées au frontend (#[tauri::command]). Couche mince : délègue aux services.

use crate::database::{servers_repository, tunnels_repository, Database};
use crate::errors::AppError;
use crate::models::{ConfigHostInput, Host, Server, ServerInput, SshKey, Tunnel, TunnelInput};
use crate::services::terminal::TerminalManager;
use crate::services::tunnels::TunnelManager;

/// Liste les hôtes du `~/.ssh/config` (résolus via `ssh -G`).
#[tauri::command]
pub fn list_hosts() -> Result<Vec<Host>, AppError> {
    crate::ssh::config::list_hosts()
}

/// Écrit un fichier de sauvegarde à l'emplacement choisi par l'utilisateur.
///
/// Le chemin provient du sélecteur de fichier natif : c'est l'utilisateur qui
/// désigne la destination.
#[tauri::command]
pub fn backup_write(path: String, content: String) -> Result<(), AppError> {
    std::fs::write(path, content)?;
    Ok(())
}

/// Lit un fichier de sauvegarde.
#[tauri::command]
pub fn backup_read(path: String) -> Result<String, AppError> {
    Ok(std::fs::read_to_string(path)?)
}

/// Ajoute une entrée au `~/.ssh/config`.
#[tauri::command]
pub fn config_host_create(input: ConfigHostInput) -> Result<(), AppError> {
    crate::ssh::config::create_host(&input)
}

/// Met à jour (ou renomme) une entrée du `~/.ssh/config`.
#[tauri::command]
pub fn config_host_update(alias: String, input: ConfigHostInput) -> Result<(), AppError> {
    crate::ssh::config::update_host(&alias, &input)
}

/// Supprime une entrée du `~/.ssh/config`.
#[tauri::command]
pub fn config_host_delete(alias: String) -> Result<(), AppError> {
    crate::ssh::config::delete_host(&alias)
}

/// Liste les clés SSH détectées dans `~/.ssh`.
#[tauri::command]
pub fn list_keys() -> Result<Vec<SshKey>, AppError> {
    crate::ssh::keys::list_keys()
}

/// Génère une nouvelle paire de clés dans `~/.ssh`.
#[tauri::command]
pub fn key_generate(
    name: String,
    key_type: String,
    comment: String,
    passphrase: String,
) -> Result<SshKey, AppError> {
    crate::ssh::keys::generate_key(&name, &key_type, &comment, &passphrase)
}

/// Importe une clé privée existante dans `~/.ssh`.
#[tauri::command]
pub fn key_import(source: String, name: String) -> Result<SshKey, AppError> {
    crate::ssh::keys::import_key(&source, &name)
}

/// Supprime une clé de `~/.ssh`.
#[tauri::command]
pub fn key_delete(name: String) -> Result<(), AppError> {
    crate::ssh::keys::delete_key(&name)
}

/// Lit le contenu de la clé publique (pour copie dans le presse-papiers).
#[tauri::command]
pub fn key_read_public(name: String) -> Result<String, AppError> {
    crate::ssh::keys::read_public_key(&name)
}

/// Lit le contenu de la clé privée (affichage local à la demande de l'utilisateur).
#[tauri::command]
pub fn key_read_private(name: String) -> Result<String, AppError> {
    crate::ssh::keys::read_private_key(&name)
}

/// Enregistre le contenu modifié de la clé privée (sauvegarde + permissions 600).
#[tauri::command]
pub fn key_write_private(name: String, content: String) -> Result<(), AppError> {
    crate::ssh::keys::write_private_key(&name, &content)
}

/// Enregistre le contenu modifié de la clé publique.
#[tauri::command]
pub fn key_write_public(name: String, content: String) -> Result<(), AppError> {
    crate::ssh::keys::write_public_key(&name, &content)
}

/// Charge la clé dans l'agent SSH (et le Trousseau macOS) : la passphrase n'est
/// alors plus demandée aux connexions suivantes.
#[tauri::command]
pub fn key_add_to_agent(
    name: String,
    passphrase: String,
    configure_ssh: bool,
) -> Result<(), AppError> {
    let home = dirs::home_dir().ok_or(AppError::HomeDirNotFound)?;
    let path = home.join(".ssh").join(&name);
    crate::ssh::agent::add_key(&path, &passphrase)?;
    if configure_ssh {
        crate::ssh::agent::ensure_config()?;
    }
    Ok(())
}

/// Retire la clé de l'agent SSH.
#[tauri::command]
pub fn key_remove_from_agent(name: String) -> Result<(), AppError> {
    let home = dirs::home_dir().ok_or(AppError::HomeDirNotFound)?;
    crate::ssh::agent::remove_key(&home.join(".ssh").join(&name))
}

/// Restaure les permissions 600 sur une clé privée.
#[tauri::command]
pub fn key_fix_permissions(name: String) -> Result<(), AppError> {
    crate::ssh::keys::fix_permissions(&name)
}

/// Ajoute, change ou retire la passphrase d'une clé privée.
#[tauri::command]
pub fn key_change_passphrase(
    name: String,
    old_passphrase: String,
    new_passphrase: String,
) -> Result<SshKey, AppError> {
    crate::ssh::keys::change_passphrase(&name, &old_passphrase, &new_passphrase)
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

/// Liste les tunnels enregistrés.
#[tauri::command]
pub fn tunnel_list(db: tauri::State<'_, Database>) -> Result<Vec<Tunnel>, AppError> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    tunnels_repository::list(&conn)
}

/// Crée un tunnel.
#[tauri::command]
pub fn tunnel_create(
    db: tauri::State<'_, Database>,
    input: TunnelInput,
) -> Result<Tunnel, AppError> {
    let conn = db.conn.lock().expect("db mutex poisoned");
    tunnels_repository::create(&conn, &input)
}

/// Supprime un tunnel (l'arrête d'abord s'il tourne).
#[tauri::command]
pub fn tunnel_delete(
    db: tauri::State<'_, Database>,
    manager: tauri::State<'_, TunnelManager>,
    id: i64,
) -> Result<(), AppError> {
    manager.stop(id)?;
    let conn = db.conn.lock().expect("db mutex poisoned");
    tunnels_repository::delete(&conn, id)
}

/// Démarre un tunnel.
#[tauri::command]
pub fn tunnel_start(
    db: tauri::State<'_, Database>,
    manager: tauri::State<'_, TunnelManager>,
    id: i64,
) -> Result<(), AppError> {
    let tunnel = {
        let conn = db.conn.lock().expect("db mutex poisoned");
        tunnels_repository::get(&conn, id)?
    };
    manager.start(&tunnel)
}

/// Arrête un tunnel.
#[tauri::command]
pub fn tunnel_stop(manager: tauri::State<'_, TunnelManager>, id: i64) -> Result<(), AppError> {
    manager.stop(id)
}

/// Identifiants des tunnels actuellement actifs.
#[tauri::command]
pub fn tunnel_running(manager: tauri::State<'_, TunnelManager>) -> Result<Vec<i64>, AppError> {
    Ok(manager.running_ids())
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
