//! Scan des clés SSH présentes dans `~/.ssh`.
//!
//! Détection basée sur les fichiers `*.pub`, dont on extrait type / taille /
//! empreinte / commentaire via `ssh-keygen -lf`. La présence de la clé privée
//! correspondante est vérifiée sur le disque.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::errors::AppError;
use crate::models::SshKey;

/// Retourne le chemin du répertoire `~/.ssh` (créé si absent).
fn ssh_dir() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or(AppError::HomeDirNotFound)?;
    Ok(home.join(".ssh"))
}

/// Valide un nom de fichier de clé (anti path-traversal : simple nom, pas de séparateur).
fn validate_key_name(name: &str) -> Result<(), AppError> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.ends_with(".pub")
    {
        return Err(AppError::Command(format!("nom de clé invalide : {name}")));
    }
    Ok(())
}

/// Applique les permissions restrictives d'une clé privée (0600 sur Unix).
#[cfg(unix)]
fn secure_private(path: &Path) -> Result<(), AppError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}
#[cfg(not(unix))]
fn secure_private(_path: &Path) -> Result<(), AppError> {
    Ok(())
}

/// Génère une nouvelle paire de clés dans `~/.ssh` via `ssh-keygen`.
pub fn generate_key(
    name: &str,
    key_type: &str,
    comment: &str,
    passphrase: &str,
) -> Result<SshKey, AppError> {
    validate_key_name(name)?;
    if !matches!(key_type, "ed25519" | "rsa" | "ecdsa") {
        return Err(AppError::Command(format!(
            "type de clé non supporté : {key_type}"
        )));
    }

    let dir = ssh_dir()?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(name);
    if path.exists() {
        return Err(AppError::Command(format!("une clé « {name} » existe déjà")));
    }

    let mut cmd = Command::new("ssh-keygen");
    cmd.arg("-t")
        .arg(key_type)
        .arg("-f")
        .arg(&path)
        .arg("-N")
        .arg(passphrase)
        .arg("-C")
        .arg(comment);
    if key_type == "rsa" {
        cmd.arg("-b").arg("4096");
    }

    let output = cmd.output().map_err(|e| AppError::Command(e.to_string()))?;
    if !output.status.success() {
        return Err(AppError::Command(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    secure_private(&path)?;

    inspect_key(&dir.join(format!("{name}.pub")))
        .ok_or_else(|| AppError::Command("clé générée mais illisible".into()))
}

/// Importe une clé privée existante (et sa clé publique) dans `~/.ssh`.
pub fn import_key(source: &str, name: &str) -> Result<SshKey, AppError> {
    validate_key_name(name)?;
    let src = Path::new(source);
    if !src.is_file() {
        return Err(AppError::Io(format!("fichier introuvable : {source}")));
    }

    let dir = ssh_dir()?;
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join(name);
    if dest.exists() {
        return Err(AppError::Command(format!("une clé « {name} » existe déjà")));
    }

    std::fs::copy(src, &dest)?;
    secure_private(&dest)?;

    // Clé publique : copie du `.pub` voisin, sinon dérivation via `ssh-keygen -y`.
    let dest_pub = dir.join(format!("{name}.pub"));
    let src_pub = PathBuf::from(format!("{source}.pub"));
    if src_pub.is_file() {
        std::fs::copy(&src_pub, &dest_pub)?;
    } else if let Ok(output) = Command::new("ssh-keygen")
        .arg("-y")
        .arg("-f")
        .arg(&dest)
        .output()
    {
        if output.status.success() {
            std::fs::write(&dest_pub, &output.stdout)?;
        }
    }

    inspect_key(&dest_pub)
        .ok_or_else(|| AppError::Command("clé importée mais clé publique illisible".into()))
}

/// Supprime une clé (privée + publique) de `~/.ssh`.
pub fn delete_key(name: &str) -> Result<(), AppError> {
    validate_key_name(name)?;
    let dir = ssh_dir()?;
    let private = dir.join(name);
    let public = dir.join(format!("{name}.pub"));
    if private.exists() {
        std::fs::remove_file(&private)?;
    }
    if public.exists() {
        std::fs::remove_file(&public)?;
    }
    Ok(())
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
