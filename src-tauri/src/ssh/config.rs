//! Lecture des hôtes du `~/.ssh/config`.
//!
//! On n'écrit PAS un parseur complet de `ssh_config(5)` : on extrait seulement
//! les alias déclarés par les lignes `Host`, puis on délègue la résolution
//! réelle (HostName, User, Port, IdentityFile, ProxyJump, Include, Match…) à
//! OpenSSH via `ssh -G <alias>`.

use std::path::PathBuf;
use std::process::Command;

use crate::errors::AppError;
use crate::models::Host;

/// Retourne le chemin du fichier `~/.ssh/config`.
fn config_path() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or(AppError::HomeDirNotFound)?;
    Ok(home.join(".ssh").join("config"))
}

/// Liste les hôtes configurés, résolus par OpenSSH.
pub fn list_hosts() -> Result<Vec<Host>, AppError> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path)?;
    let aliases = parse_host_aliases(&content);

    Ok(aliases.iter().map(|alias| resolve_host(alias)).collect())
}

/// Extrait les alias des lignes `Host`, en ignorant les motifs génériques
/// (`*`, `?`, négations `!`) qui ne désignent pas un hôte concret.
fn parse_host_aliases(content: &str) -> Vec<String> {
    let mut aliases: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let Some(keyword) = parts.next() else {
            continue;
        };
        if !keyword.eq_ignore_ascii_case("host") {
            continue;
        }

        for pattern in parts {
            if pattern.contains('*') || pattern.contains('?') || pattern.contains('!') {
                continue;
            }
            if !aliases.iter().any(|a| a == pattern) {
                aliases.push(pattern.to_string());
            }
        }
    }

    aliases
}

/// Résout un alias via `ssh -G`. En cas d'échec, retourne un hôte minimal.
fn resolve_host(alias: &str) -> Host {
    let mut host = Host {
        alias: alias.to_string(),
        hostname: None,
        user: None,
        port: None,
        identity_file: None,
        proxy_jump: None,
    };

    let Ok(output) = Command::new("ssh").arg("-G").arg(alias).output() else {
        return host;
    };
    if !output.status.success() {
        return host;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let mut it = line.splitn(2, ' ');
        let key = it.next().unwrap_or("");
        let value = it.next().unwrap_or("").trim();
        if value.is_empty() {
            continue;
        }

        match key {
            "hostname" => host.hostname = Some(value.to_string()),
            "user" => host.user = Some(value.to_string()),
            "port" => host.port = value.parse().ok(),
            // `ssh -G` peut lister plusieurs identityfile : on garde la première.
            "identityfile" if host.identity_file.is_none() => {
                host.identity_file = Some(value.to_string());
            }
            "proxyjump" if value != "none" => host.proxy_jump = Some(value.to_string()),
            _ => {}
        }
    }

    host
}
