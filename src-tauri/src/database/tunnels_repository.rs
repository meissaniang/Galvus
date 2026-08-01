//! Repository des tunnels : tout l'accès SQL à la table `tunnels` vit ici.

use rusqlite::{params, Connection, Row};

use crate::errors::AppError;
use crate::models::{Tunnel, TunnelInput};

fn map_row(row: &Row) -> rusqlite::Result<Tunnel> {
    Ok(Tunnel {
        id: row.get(0)?,
        name: row.get(1)?,
        kind: row.get(2)?,
        ssh_target: row.get(3)?,
        listen_port: row.get::<_, i64>(4)? as u16,
        target_host: row.get(5)?,
        target_port: row.get::<_, Option<i64>>(6)?.map(|p| p as u16),
    })
}

const COLUMNS: &str = "id, name, kind, ssh_target, listen_port, target_host, target_port";

pub fn list(conn: &Connection) -> Result<Vec<Tunnel>, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM tunnels ORDER BY name COLLATE NOCASE");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    let mut tunnels = Vec::new();
    for row in rows {
        tunnels.push(row?);
    }
    Ok(tunnels)
}

pub fn get(conn: &Connection, id: i64) -> Result<Tunnel, AppError> {
    let sql = format!("SELECT {COLUMNS} FROM tunnels WHERE id = ?1");
    Ok(conn.query_row(&sql, params![id], map_row)?)
}

pub fn create(conn: &Connection, input: &TunnelInput) -> Result<Tunnel, AppError> {
    conn.execute(
        "INSERT INTO tunnels (name, kind, ssh_target, listen_port, target_host, target_port)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            input.name,
            input.kind,
            input.ssh_target,
            input.listen_port as i64,
            input.target_host,
            input.target_port.map(|p| p as i64),
        ],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn delete(conn: &Connection, id: i64) -> Result<(), AppError> {
    conn.execute("DELETE FROM tunnels WHERE id = ?1", params![id])?;
    Ok(())
}
