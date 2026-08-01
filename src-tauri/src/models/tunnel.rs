//! Modèle d'un tunnel SSH (redirection de port), persisté dans la base chiffrée.

use serde::{Deserialize, Serialize};

/// Un tunnel SSH.
///
/// - `local`   : `-L listen_port:target_host:target_port ssh_target`
/// - `remote`  : `-R listen_port:target_host:target_port ssh_target`
/// - `dynamic` : `-D listen_port ssh_target` (proxy SOCKS)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tunnel {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub ssh_target: String,
    pub listen_port: u16,
    pub target_host: Option<String>,
    pub target_port: Option<u16>,
}

/// Données de création/édition d'un tunnel (sans identifiant).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelInput {
    pub name: String,
    pub kind: String,
    pub ssh_target: String,
    pub listen_port: u16,
    pub target_host: Option<String>,
    pub target_port: Option<u16>,
}
