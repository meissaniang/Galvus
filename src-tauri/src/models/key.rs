//! Modèle d'une clé SSH détectée dans `~/.ssh`.

use serde::Serialize;

/// Une clé SSH (paire publique/privée) présente dans `~/.ssh`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SshKey {
    /// Nom de base du fichier (sans `.pub`).
    pub name: String,
    /// Chemin absolu de la clé publique.
    pub path: String,
    /// Type de clé (ED25519, RSA, ECDSA…).
    pub key_type: Option<String>,
    /// Taille en bits.
    pub bits: Option<u32>,
    /// Empreinte (`SHA256:…`).
    pub fingerprint: Option<String>,
    /// Commentaire embarqué dans la clé.
    pub comment: Option<String>,
    /// Vrai si la clé privée correspondante existe sur le disque.
    pub has_private: bool,
    /// Vrai si la clé privée est protégée par une passphrase.
    pub encrypted: bool,
    /// Vrai si la clé privée a des permissions trop ouvertes (ssh la refusera).
    pub insecure_permissions: bool,
}
