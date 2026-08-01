//! Modèle d'un serveur créé par l'utilisateur (persisté dans la base chiffrée).
//!
//! Distinct de [`crate::models::Host`], qui provient de `~/.ssh/config` en
//! lecture seule.

use serde::{Deserialize, Serialize};

/// Un serveur enregistré par l'utilisateur.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: i64,
    pub name: String,
    pub hostname: String,
    pub port: u16,
    pub username: Option<String>,
    pub identity_file: Option<String>,
    pub color: Option<String>,
    pub favorite: bool,
    pub tags: Vec<String>,
    pub group: Option<String>,
}

/// Données de création/édition d'un serveur (sans identifiant).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInput {
    pub name: String,
    pub hostname: String,
    pub port: u16,
    pub username: Option<String>,
    pub identity_file: Option<String>,
    pub color: Option<String>,
    pub favorite: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    pub group: Option<String>,
}
