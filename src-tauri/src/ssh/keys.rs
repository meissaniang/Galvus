//! Scan des clés SSH présentes dans `~/.ssh`.
//!
//! Détection basée sur les fichiers `*.pub`, dont on extrait type / taille /
//! empreinte / commentaire via `ssh-keygen -lf`. La présence de la clé privée
//! correspondante est vérifiée sur le disque.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::errors::AppError;
use crate::models::SshKey;

/// Retourne le chemin du répertoire `~/.ssh`.
fn ssh_dir() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or(AppError::HomeDirNotFound)?;
    Ok(home.join(".ssh"))
}

/// Liste les clés SSH détectées dans `~/.ssh`, triées par nom.
pub fn list_keys() -> Result<Vec<SshKey>, AppError> {
    let dir = ssh_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut keys = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let is_pub = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("pub"))
            .unwrap_or(false);
        if !is_pub {
            continue;
        }

        if let Some(key) = inspect_key(&path) {
            keys.push(key);
        }
    }

    keys.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(keys)
}

/// Interroge `ssh-keygen -lf` sur une clé publique et construit le modèle.
///
/// Sortie attendue : `256 SHA256:xxxx commentaire (ED25519)`.
fn inspect_key(pub_path: &Path) -> Option<SshKey> {
    let output = Command::new("ssh-keygen")
        .arg("-lf")
        .arg(pub_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let line = String::from_utf8_lossy(&output.stdout);
    let line = line.trim();

    let mut parts = line.splitn(3, ' ');
    let bits = parts.next().and_then(|s| s.parse::<u32>().ok());
    let fingerprint = parts.next().map(str::to_string);
    let rest = parts.next().unwrap_or("");

    // `rest` = "commentaire (TYPE)".
    let (comment, key_type) = match rest.rfind('(') {
        Some(idx) => {
            let comment = rest[..idx].trim();
            let key_type = rest[idx + 1..].trim_end_matches(')').trim();
            (
                (!comment.is_empty()).then(|| comment.to_string()),
                (!key_type.is_empty()).then(|| key_type.to_string()),
            )
        }
        None => (None, None),
    };

    let name = pub_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    // Clé privée = même chemin sans l'extension `.pub`.
    let has_private = pub_path.with_extension("").exists();

    Some(SshKey {
        name,
        path: pub_path.to_string_lossy().to_string(),
        key_type,
        bits,
        fingerprint,
        comment,
        has_private,
    })
}
