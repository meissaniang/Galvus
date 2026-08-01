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

    /// Crée le schéma si nécessaire.
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
        Ok(())
    }
}
