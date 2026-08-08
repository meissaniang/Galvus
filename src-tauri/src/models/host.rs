//! Modèle d'un hôte SSH, tel que résolu depuis `~/.ssh/config` via `ssh -G`.

use serde::{Deserialize, Serialize};

/// Un hôte SSH configuré (une entrée `Host` du fichier de config).
///
/// Les champs de connexion sont résolus par OpenSSH (`ssh -G`), ce qui gère
/// nativement les `Include`, `Match` et l'héritage de valeurs par défaut.
///
/// Les champs de présentation — groupe, couleur, tags, favori — n'existent pas
/// dans `ssh_config(5)` : ils sont conservés dans un commentaire `# galvus:`
/// placé juste au-dessus du bloc, qu'OpenSSH ignore. Ils suivent ainsi le
/// fichier lors d'une sauvegarde ou d'une copie sur une autre machine.
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

    // --- Métadonnées Galvus (commentaire `# galvus:`) ---
    pub group: Option<String>,
    pub color: Option<String>,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub os: Option<String>,
}

/// Champs éditables d'une entrée `Host` du fichier `~/.ssh/config`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigHostInput {
    /// Alias — permet aussi de renommer l'entrée.
    pub alias: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<String>,
    pub proxy_jump: Option<String>,

    pub group: Option<String>,
    pub color: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub os: Option<String>,
}
