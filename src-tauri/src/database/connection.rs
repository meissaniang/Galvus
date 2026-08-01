//! Connexion à la base SQLite chiffrée (SQLCipher) et migrations.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::errors::AppError;

/// Base de données de l'application, partagée comme état Tauri.
///
/// La connexion est protégée par un `Mutex` : rusqlite `Connection` n'est pas
/// `Sync`, et l'accès concurrent depuis les commandes doit être sérialisé.
pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    /// Ouvre la base à `db_path`, la déverrouille avec `key` (passphrase
    /// SQLCipher) puis applique les migrations.
    pub fn initialize(db_path: &Path, key: &str) -> Result<Self, AppError> {
        let conn = Connection::open(db_path)?;

        // Déverrouillage SQLCipher. `key` est une chaîne hexadécimale [0-9a-f]
        // issue du coffre natif : aucune injection possible.
        conn.execute_batch(&format!("PRAGMA key = '{key}';"))?;

        // Vérifie que la clé est correcte (base lisible).
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
            .map_err(|e| AppError::Database(format!("clé invalide ou base illisible : {e}")))?;

        Self::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Crée le schéma et applique les migrations additives.
    fn migrate(conn: &Connection) -> Result<(), AppError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS servers (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                name          TEXT    NOT NULL,
                hostname      TEXT    NOT NULL,
                port          INTEGER NOT NULL DEFAULT 22,
                username      TEXT,
                identity_file TEXT,
                color         TEXT,
                favorite      INTEGER NOT NULL DEFAULT 0,
                created_at    TEXT    NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        // Colonnes ajoutées après coup (bases existantes de l'étape 5).
        ensure_column(conn, "servers", "tags", "TEXT NOT NULL DEFAULT '[]'")?;
        ensure_column(conn, "servers", "group_name", "TEXT")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tunnels (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT    NOT NULL,
                kind        TEXT    NOT NULL,
                ssh_target  TEXT    NOT NULL,
                listen_port INTEGER NOT NULL,
                target_host TEXT,
                target_port INTEGER,
                created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        Ok(())
    }
}

/// Ajoute une colonne si elle n'existe pas déjà (migration idempotente).
fn ensure_column(
    conn: &Connection,
    table: &str,
    column: &str,
    declaration: &str,
) -> Result<(), AppError> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let exists = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(Result::ok)
        .any(|name| name == column);
    if !exists {
        conn.execute_batch(&format!(
            "ALTER TABLE {table} ADD COLUMN {column} {declaration};"
        ))?;
    }
    Ok(())
}
