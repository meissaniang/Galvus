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

/// Vrai si la ligne ouvre un nouveau bloc (`Host` ou `Match`).
fn is_block_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    let keyword = trimmed.split_whitespace().next().unwrap_or("");
    keyword.eq_ignore_ascii_case("host") || keyword.eq_ignore_ascii_case("match")
}

/// Vrai si la ligne est le `Host` déclarant exactement cet alias.
fn is_host_line_for(line: &str, alias: &str) -> bool {
    let trimmed = line.trim();
    let mut parts = trimmed.split_whitespace();
    match parts.next() {
        Some(k) if k.eq_ignore_ascii_case("host") => parts.any(|p| p == alias),
        _ => false,
    }
}

/// Rend le bloc `Host` correspondant aux champs fournis.
fn render_block(input: &crate::models::ConfigHostInput) -> Vec<String> {
    fn push(lines: &mut Vec<String>, key: &str, value: &str) {
        let value = value.trim();
        if !value.is_empty() {
            lines.push(format!("    {key} {value}"));
        }
    }

    let mut lines = vec![format!("Host {}", input.alias.trim())];
    if let Some(v) = &input.hostname {
        push(&mut lines, "HostName", v);
    }
    if let Some(v) = &input.user {
        push(&mut lines, "User", v);
    }
    if let Some(p) = input.port {
        if p != 22 {
            lines.push(format!("    Port {p}"));
        }
    }
    if let Some(v) = &input.identity_file {
        push(&mut lines, "IdentityFile", v);
    }
    if let Some(v) = &input.proxy_jump {
        push(&mut lines, "ProxyJump", v);
    }
    lines
}

/// Réécrit le fichier de config après sauvegarde en `config.bak`.
fn write_config(path: &std::path::Path, lines: &[String]) -> Result<(), AppError> {
    if path.exists() {
        let _ = std::fs::copy(path, path.with_extension("bak"));
    }
    let mut content = lines.join("\n");
    if !content.ends_with('\n') {
        content.push('\n');
    }
    std::fs::write(path, content)?;
    Ok(())
}

/// Met à jour (ou renomme) une entrée `Host` du `~/.ssh/config`.
///
/// Le bloc existant est remplacé intégralement par les champs fournis ; le reste
/// du fichier (commentaires, autres hôtes, Include) est préservé tel quel.
pub fn update_host(alias: &str, input: &crate::models::ConfigHostInput) -> Result<(), AppError> {
    if input.alias.trim().is_empty() || input.alias.split_whitespace().count() != 1 {
        return Err(AppError::Command("alias invalide".into()));
    }

    let path = config_path()?;
    let content = std::fs::read_to_string(&path)?;

    let mut out: Vec<String> = Vec::new();
    let mut found = false;
    let mut skipping = false;

    for line in content.lines() {
        if skipping {
            // On saute les options du bloc jusqu'au prochain Host/Match.
            if is_block_start(line) {
                skipping = false;
            } else {
                continue;
            }
        }
        if !found && is_host_line_for(line, alias) {
            out.extend(render_block(input));
            found = true;
            skipping = true;
            continue;
        }
        out.push(line.to_string());
    }

    if !found {
        return Err(AppError::Command(format!("hôte « {alias} » introuvable")));
    }

    write_config(&path, &out)
}

/// Supprime une entrée `Host` du `~/.ssh/config`.
pub fn delete_host(alias: &str) -> Result<(), AppError> {
    let path = config_path()?;
    let content = std::fs::read_to_string(&path)?;

    let mut out: Vec<String> = Vec::new();
    let mut found = false;
    let mut skipping = false;

    for line in content.lines() {
        if skipping {
            if is_block_start(line) {
                skipping = false;
            } else {
                continue;
            }
        }
        if !found && is_host_line_for(line, alias) {
            found = true;
            skipping = true;
            continue;
        }
        out.push(line.to_string());
    }

    if !found {
        return Err(AppError::Command(format!("hôte « {alias} » introuvable")));
    }

    write_config(&path, &out)
}

#[cfg(test)]
mod tests {
    use super::parse_host_aliases;

    #[test]
    fn extrait_les_alias_simples() {
        let cfg = "Host web\n  HostName 1.2.3.4\nHost db\n  User root\n";
        assert_eq!(parse_host_aliases(cfg), vec!["web", "db"]);
    }

    #[test]
    fn ignore_les_motifs_generiques_et_commentaires() {
        let cfg = "# commentaire\nHost *\n  ForwardAgent yes\nHost prod !staging\n";
        assert_eq!(parse_host_aliases(cfg), vec!["prod"]);
    }

    #[test]
    fn gere_plusieurs_alias_sur_une_ligne_sans_doublon() {
        let cfg = "Host a b\nHost b c\n";
        assert_eq!(parse_host_aliases(cfg), vec!["a", "b", "c"]);
    }
}
