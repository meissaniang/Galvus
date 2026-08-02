//! Intégration de l'agent SSH et du trousseau natif.
//!
//! Objectif : ne saisir la passphrase qu'une seule fois. La clé est ajoutée à
//! `ssh-agent` et, sur macOS, la passphrase est mémorisée dans le Trousseau
//! (`ssh-add --apple-use-keychain`), conformément à la règle « secrets dans le
//! coffre natif uniquement ».
//!
//! La passphrase est transmise via un pseudo-terminal : elle ne transite ni par
//! un fichier temporaire, ni par une variable d'environnement, ni par la ligne
//! de commande (visible dans `ps`).

use std::collections::HashSet;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use portable_pty::{CommandBuilder, PtySize};

use crate::errors::AppError;

/// Empreintes (`SHA256:…`) des clés actuellement chargées dans l'agent.
pub fn loaded_fingerprints() -> HashSet<String> {
    let mut set = HashSet::new();
    let Ok(output) = Command::new("ssh-add").arg("-l").output() else {
        return set;
    };
    if !output.status.success() {
        return set; // agent absent ou aucune identité
    }
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(fp) = line.split_whitespace().nth(1) {
            set.insert(fp.to_string());
        }
    }
    set
}

/// Ajoute la clé à l'agent en lui fournissant la passphrase via un PTY.
///
/// Sur macOS, `--apple-use-keychain` enregistre aussi la passphrase dans le
/// Trousseau : elle sera rechargée automatiquement aux prochaines sessions.
pub fn add_key(private_path: &Path, passphrase: &str) -> Result<(), AppError> {
    if !private_path.is_file() {
        return Err(AppError::Io(format!(
            "clé introuvable : {}",
            private_path.display()
        )));
    }

    let pair = portable_pty::native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| AppError::Command(e.to_string()))?;

    let mut cmd = CommandBuilder::new("ssh-add");
    #[cfg(target_os = "macos")]
    cmd.arg("--apple-use-keychain");
    cmd.arg(private_path);

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| AppError::Command(e.to_string()))?;
    drop(pair.slave);

    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| AppError::Command(e.to_string()))?;
    let mut writer = pair
        .master
        .take_writer()
        .map_err(|e| AppError::Command(e.to_string()))?;

    // Collecte de la sortie dans un thread : le PTY doit être lu en continu
    // pour ne pas bloquer le processus enfant.
    let transcript = Arc::new(Mutex::new(String::new()));
    let sink = Arc::clone(&transcript);
    std::thread::spawn(move || {
        let mut buf = [0u8; 1024];
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            if let Ok(mut guard) = sink.lock() {
                guard.push_str(&String::from_utf8_lossy(&buf[..n]));
            }
        }
    });

    // Répond dès que l'invite de passphrase apparaît (rien à envoyer si la clé
    // n'est pas chiffrée : ssh-add se termine seul).
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut answered = false;
    while Instant::now() < deadline {
        if let Some(status) = child
            .try_wait()
            .map_err(|e| AppError::Command(e.to_string()))?
        {
            return finish(status.success(), &transcript);
        }
        let seen = transcript
            .lock()
            .map(|g| g.to_lowercase().contains("passphrase"))
            .unwrap_or(false);
        if seen && !answered {
            use std::io::Write as _;
            writer
                .write_all(format!("{passphrase}\n").as_bytes())
                .map_err(|e| AppError::Io(e.to_string()))?;
            writer.flush().map_err(|e| AppError::Io(e.to_string()))?;
            answered = true;
        }
        std::thread::sleep(Duration::from_millis(80));
    }

    let _ = child.kill();
    Err(AppError::Command(
        "délai dépassé en communiquant avec ssh-add".into(),
    ))
}

/// Traduit la sortie de `ssh-add` en résultat exploitable.
fn finish(success: bool, transcript: &Arc<Mutex<String>>) -> Result<(), AppError> {
    if success {
        return Ok(());
    }
    let text = transcript
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default()
        .to_lowercase();
    let message = if text.contains("bad passphrase") || text.contains("incorrect passphrase") {
        "passphrase incorrecte"
    } else if text.contains("could not open a connection") {
        "aucun agent SSH disponible (SSH_AUTH_SOCK absent)"
    } else {
        "échec de l'ajout à l'agent"
    };
    Err(AppError::Command(message.into()))
}

/// Retire la clé de l'agent.
pub fn remove_key(private_path: &Path) -> Result<(), AppError> {
    let output = Command::new("ssh-add")
        .arg("-d")
        .arg(private_path)
        .output()
        .map_err(|e| AppError::Command(e.to_string()))?;
    if !output.status.success() {
        return Err(AppError::Command(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    Ok(())
}

/// Ajoute au `~/.ssh/config` le bloc qui fait charger les clés dans l'agent et
/// relire le Trousseau automatiquement, si ce n'est pas déjà configuré.
pub fn ensure_config() -> Result<bool, AppError> {
    let home = dirs::home_dir().ok_or(AppError::HomeDirNotFound)?;
    let dir = home.join(".ssh");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("config");

    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if existing.to_lowercase().contains("addkeystoagent") {
        return Ok(false);
    }

    if path.exists() {
        let _ = std::fs::copy(&path, path.with_extension("bak"));
    }

    // Le bloc doit précéder les autres `Host` : OpenSSH retient la première
    // valeur rencontrée pour chaque option.
    let mut block = String::from(
        "# Ajouté par Galvus : mémorise les passphrases dans l'agent / le Trousseau.\nHost *\n    AddKeysToAgent yes\n",
    );
    if cfg!(target_os = "macos") {
        block.push_str("    UseKeychain yes\n");
    }
    block.push('\n');
    block.push_str(&existing);

    std::fs::write(&path, block)?;
    log::info!("~/.ssh/config : bloc agent/Trousseau ajouté");
    Ok(true)
}
