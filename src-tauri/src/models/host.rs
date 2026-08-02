//! Modèle d'un hôte SSH, tel que résolu depuis `~/.ssh/config` via `ssh -G`.

use serde::{Deserialize, Serialize};

/// Un hôte SSH configuré (une entrée `Host` du fichier de config).
///
/// Les champs sont résolus par OpenSSH (`ssh -G`), ce qui gère nativement
/// les `Include`, `Match` et l'héritage de valeurs par défaut.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Host {
    /// Alias déclaré dans `Host <alias>`.
    pub alias: String,
    /// Nom d'hôte / IP résolu (`HostName`).
    pub hostname: Option<String>,
    /// Utilisateur (`User`).
    pub user: Option<String>,
    /// Port (`Port`, 22 par défaut).
    pub port: Option<u16>,
    /// Première `IdentityFile` résolue.
    pub identity_file: Option<String>,
    /// Bastion (`ProxyJump`), si défini.
    pub proxy_jump: Option<String>,
}

/// Champs éditables d'une entrée `Host` du fichier `~/.ssh/config`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigHostInput {
    /// Nouvel alias (permet de renommer l'entrée).
    pub alias: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<String>,
    pub proxy_jump: Option<String>,
}
