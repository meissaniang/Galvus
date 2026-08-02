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

/// Lit le contenu de la clé publique `<name>.pub` dans `~/.ssh`.
pub fn read_public_key(name: &str) -> Result<String, AppError> {
    validate_key_name(name)?;
    let dir = ssh_dir()?;
    let path = dir.join(format!("{name}.pub"));
    Ok(std::fs::read_to_string(path)?.trim().to_string())
}

/// Lit le contenu de la clé PRIVÉE `<name>` dans `~/.ssh`.
///
/// Affichage local uniquement (à la demande de l'utilisateur) : la clé ne
/// quitte jamais la machine et n'est copiée nulle part par l'application.
pub fn read_private_key(name: &str) -> Result<String, AppError> {
    validate_key_name(name)?;
    let dir = ssh_dir()?;
    let path = dir.join(name);
    if !path.is_file() {
        return Err(AppError::Io(format!("clé privée introuvable : {name}")));
    }
    Ok(std::fs::read_to_string(path)?)
}

/// Indique si la clé privée est protégée par une passphrase.
///
/// L'analyse porte sur le fichier lui-même (et non sur `ssh-keygen`, qui refuse
/// de lire une clé aux permissions trop ouvertes et fausserait le résultat) :
/// - format OpenSSH : le champ `ciphername` de l'en-tête vaut `none` si la clé
///   est en clair ;
/// - formats PEM historiques : marqueurs `Proc-Type: 4,ENCRYPTED` / PKCS#8.
fn is_encrypted(private_path: &Path) -> bool {
    use base64::Engine as _;

    let Ok(content) = std::fs::read_to_string(private_path) else {
        return false;
    };

    if content.contains("Proc-Type: 4,ENCRYPTED") || content.contains("BEGIN ENCRYPTED PRIVATE KEY")
    {
        return true;
    }

    if !content.contains("BEGIN OPENSSH PRIVATE KEY") {
        return false;
    }

    let body: String = content
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");
    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(body.trim()) else {
        return false;
    };

    const MAGIC: &[u8] = b"openssh-key-v1\0";
    let Some(rest) = bytes.strip_prefix(MAGIC) else {
        return false;
    };
    if rest.len() < 4 {
        return false;
    }
    let len = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
    match rest.get(4..4 + len) {
        Some(cipher) => cipher != b"none",
        None => false,
    }
}

/// Vrai si la clé privée est lisible par d'autres utilisateurs (permissions
/// trop ouvertes) : `ssh` refuse alors de l'utiliser.
#[cfg(unix)]
fn has_insecure_permissions(private_path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(private_path)
        .map(|m| m.permissions().mode() & 0o077 != 0)
        .unwrap_or(false)
}
#[cfg(not(unix))]
fn has_insecure_permissions(_private_path: &Path) -> bool {
    false
}

/// Garantit que le fichier se termine par un saut de ligne.
///
/// OpenSSH refuse une clé dont la dernière ligne n'est pas terminée et renvoie
/// alors un « invalid format » trompeur (cas fréquent des clés copiées-collées
/// ou exportées par d'autres outils). La réparation est sûre : elle n'ajoute
/// qu'un `\n` en fin de fichier.
fn ensure_trailing_newline(path: &Path) -> Result<(), AppError> {
    let bytes = std::fs::read(path)?;
    if bytes.last() == Some(&b'\n') {
        return Ok(());
    }
    let mut fixed = bytes;
    fixed.push(b'\n');
    std::fs::write(path, fixed)?;
    log::info!("saut de ligne final ajouté à {}", path.display());
    Ok(())
}

/// Restaure les permissions 600 sur la clé privée.
pub fn fix_permissions(name: &str) -> Result<(), AppError> {
    validate_key_name(name)?;
    let dir = ssh_dir()?;
    let path = dir.join(name);
    if !path.is_file() {
        return Err(AppError::Io(format!("clé privée introuvable : {name}")));
    }
    secure_private(&path)
}

/// Écrit le contenu d'une clé privée (sauvegarde `.bak`, permissions 600).
pub fn write_private_key(name: &str, content: &str) -> Result<(), AppError> {
    validate_key_name(name)?;
    let dir = ssh_dir()?;
    let path = dir.join(name);
    if !path.is_file() {
        return Err(AppError::Io(format!("clé privée introuvable : {name}")));
    }
    let _ = std::fs::copy(&path, dir.join(format!("{name}.bak")));

    let mut body = content.trim_end().to_string();
    body.push('\n');
    std::fs::write(&path, body)?;
    secure_private(&path)?;
    Ok(())
}

/// Écrit le contenu d'une clé publique (sauvegarde `.bak`).
pub fn write_public_key(name: &str, content: &str) -> Result<(), AppError> {
    validate_key_name(name)?;
    let dir = ssh_dir()?;
    let path = dir.join(format!("{name}.pub"));
    if path.is_file() {
        let _ = std::fs::copy(&path, dir.join(format!("{name}.pub.bak")));
    }

    let mut body = content.trim_end().to_string();
    body.push('\n');
    std::fs::write(&path, body)?;
    Ok(())
}

/// Ajoute, change ou retire la passphrase d'une clé privée via `ssh-keygen -p`.
///
/// `new_passphrase` vide retire la protection. Une sauvegarde `.bak` est faite
/// avant l'opération.
pub fn change_passphrase(
    name: &str,
    old_passphrase: &str,
    new_passphrase: &str,
) -> Result<SshKey, AppError> {
    validate_key_name(name)?;
    let dir = ssh_dir()?;
    let path = dir.join(name);
    if !path.is_file() {
        return Err(AppError::Io(format!("clé privée introuvable : {name}")));
    }
    let _ = std::fs::copy(&path, dir.join(format!("{name}.bak")));
    // Répare le cas fréquent du fichier sans saut de ligne final, qu'OpenSSH
    // rejette avec un « invalid format » peu explicite.
    ensure_trailing_newline(&path)?;

    let output = Command::new("ssh-keygen")
        .arg("-p")
        .arg("-f")
        .arg(&path)
        .arg("-P")
        .arg(old_passphrase)
        .arg("-N")
        .arg(new_passphrase)
        .output()
        .map_err(|e| AppError::Command(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let message = stderr.trim();
        let lower = message.to_lowercase();
        let friendly = if lower.contains("incorrect passphrase") || lower.contains("load failed") {
            "passphrase actuelle incorrecte".to_string()
        } else if lower.contains("invalid format") {
            "format de clé non reconnu par OpenSSH (fichier corrompu ou tronqué)".to_string()
        } else if lower.contains("unprotected private key") {
            "permissions trop ouvertes : corrigez-les (600) avant de continuer".to_string()
        } else {
            message.to_string()
        };
        return Err(AppError::Command(friendly));
    }
    secure_private(&path)?;

    inspect_key(&dir.join(format!("{name}.pub")))
        .ok_or_else(|| AppError::Command("clé modifiée mais illisible".into()))
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

    // Une seule interrogation de l'agent pour l'ensemble des clés.
    let loaded = crate::ssh::agent::loaded_fingerprints();
    for key in &mut keys {
        if let Some(fp) = &key.fingerprint {
            key.in_agent = loaded.contains(fp);
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
    let private_path = pub_path.with_extension("");
    let has_private = private_path.exists();
    let encrypted = has_private && is_encrypted(&private_path);
    let insecure_permissions = has_private && has_insecure_permissions(&private_path);

    Some(SshKey {
        name,
        path: pub_path.to_string_lossy().to_string(),
        key_type,
        bits,
        fingerprint,
        comment,
        has_private,
        encrypted,
        insecure_permissions,
        // Renseigné par `list_keys`, qui interroge l'agent une seule fois.
        in_agent: false,
    })
}
