//! Repository des serveurs : tout l'accès SQL à la table `servers` vit ici.

use rusqlite::{params, Connection, Row};

use crate::errors::AppError;
use crate::models::{Server, ServerInput};

/// Construit un `Server` depuis une ligne SQL.
fn map_row(row: &Row) -> rusqlite::Result<Server> {
    let tags_json: String = row.get(8)?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    Ok(Server {
        id: row.get(0)?,
        name: row.get(1)?,
        hostname: row.get(2)?,
        port: row.get::<_, i64>(3)? as u16,
        username: row.get(4)?,
        identity_file: row.get(5)?,
        color: row.get(6)?,
        favorite: row.get::<_, i64>(7)? != 0,
        tags,
        group: row.get(9)?,
    })
}

const SELECT_COLUMNS: &str =
    "id, name, hostname, port, username, identity_file, color, favorite, tags, group_name";

/// Liste les serveurs (favoris d'abord, puis par nom).
pub fn list(conn: &Connection) -> Result<Vec<Server>, AppError> {
    let sql = format!(
        "SELECT {SELECT_COLUMNS} FROM servers ORDER BY favorite DESC, name COLLATE NOCASE"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    let mut servers = Vec::new();
    for row in rows {
        servers.push(row?);
    }
    Ok(servers)
}

/// Récupère un serveur par identifiant.
pub fn get(conn: &Connection, id: i64) -> Result<Server, AppError> {
    let sql = format!("SELECT {SELECT_COLUMNS} FROM servers WHERE id = ?1");
    let server = conn.query_row(&sql, params![id], map_row)?;
    Ok(server)
}

/// Crée un serveur et retourne l'enregistrement complet.
pub fn create(conn: &Connection, input: &ServerInput) -> Result<Server, AppError> {
    let tags_json = serde_json::to_string(&input.tags).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO servers
            (name, hostname, port, username, identity_file, color, favorite, tags, group_name)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            input.name,
            input.hostname,
            input.port as i64,
            input.username,
            input.identity_file,
            input.color,
            input.favorite as i64,
            tags_json,
            input.group,
        ],
    )?;
    get(conn, conn.last_insert_rowid())
}

/// Met à jour un serveur existant.
pub fn update(conn: &Connection, id: i64, input: &ServerInput) -> Result<Server, AppError> {
    let tags_json = serde_json::to_string(&input.tags).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "UPDATE servers
         SET name = ?1, hostname = ?2, port = ?3, username = ?4, identity_file = ?5,
             color = ?6, favorite = ?7, tags = ?8, group_name = ?9
         WHERE id = ?10",
        params![
            input.name,
            input.hostname,
            input.port as i64,
            input.username,
            input.identity_file,
            input.color,
            input.favorite as i64,
            tags_json,
            input.group,
            id,
        ],
    )?;
    get(conn, id)
}

/// Supprime un serveur.
pub fn delete(conn: &Connection, id: i64) -> Result<(), AppError> {
    conn.execute("DELETE FROM servers WHERE id = ?1", params![id])?;
    Ok(())
}
