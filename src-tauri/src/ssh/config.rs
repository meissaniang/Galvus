//! Lecture et écriture des hôtes du `~/.ssh/config`.
//!
//! On n'écrit PAS un parseur complet de `ssh_config(5)` : on extrait seulement
//! les alias déclarés par les lignes `Host`, puis on délègue la résolution
//! réelle (HostName, User, Port, IdentityFile, ProxyJump, Include, Match…) à
//! OpenSSH via `ssh -G <alias>`.
//!
//! Les métadonnées de présentation (groupe, couleur, tags, favori) n'existent
//! pas dans le format d'OpenSSH. Elles sont écrites dans un commentaire
//! `# galvus:` juste au-dessus du bloc, qu'OpenSSH ignore et qui suit le fichier
//! lors d'une sauvegarde.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::errors::AppError;
use crate::models::{ConfigHostInput, Host};

/// Préfixe du commentaire portant les métadonnées Galvus.
const META_PREFIX: &str = "# galvus:";

/// Retourne le chemin du fichier `~/.ssh/config`.
fn config_path() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or(AppError::HomeDirNotFound)?;
    Ok(home.join(".ssh").join("config"))
}

/// Métadonnées de présentation attachées à un hôte.
#[derive(Debug, Clone, Default)]
struct Meta {
    group: Option<String>,
    color: Option<String>,
    tags: Vec<String>,
    favorite: bool,
    os: Option<String>,
}

/// Analyse `# galvus: group=Prod; color=#4C8DFF; tags=web,eu; favorite=true; os=ubuntu`.
fn parse_meta(line: &str) -> Meta {
    let mut meta = Meta::default();
    let Some(rest) = line.trim().strip_prefix(META_PREFIX) else {
        return meta;
    };

    for field in rest.split(';') {
        let Some((key, value)) = field.split_once('=') else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        match key.trim() {
            "group" => meta.group = Some(value.to_string()),
            "color" => meta.color = Some(value.to_string()),
            "os" => meta.os = Some(value.to_string()),
            "favorite" => meta.favorite = value == "true",
            "tags" => {
                meta.tags = value
                    .split(',')
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(str::to_string)
                    .collect()
            }
            _ => {}
        }
    }
    meta
}

/// Rend la ligne de métadonnées, ou `None` s'il n'y a rien à conserver.
///
/// Les `;` et `=` sont retirés des valeurs : ce sont les séparateurs du format.
fn render_meta(input: &ConfigHostInput) -> Option<String> {
    render_meta_fields(&Meta {
        group: input.group.clone(),
        color: input.color.clone(),
        tags: input.tags.clone(),
        favorite: input.favorite,
        os: input.os.clone(),
    })
}

/// Rend la ligne de métadonnées depuis un `Meta`, ou `None` s'il n'y a rien à
/// conserver. Les `;` et `=` sont retirés des valeurs : ce sont les
/// séparateurs du format.
fn render_meta_fields(meta: &Meta) -> Option<String> {
    let clean = |s: &str| s.replace([';', '='], "").trim().to_string();
    let mut fields: Vec<String> = Vec::new();

    if let Some(group) = meta.group.as_deref().map(clean).filter(|s| !s.is_empty()) {
        fields.push(format!("group={group}"));
    }
    if let Some(color) = meta.color.as_deref().map(clean).filter(|s| !s.is_empty()) {
        fields.push(format!("color={color}"));
    }
    let tags: Vec<String> = meta
        .tags
        .iter()
        .map(|t| clean(t).replace(',', ""))
        .filter(|t| !t.is_empty())
        .collect();
    if !tags.is_empty() {
        fields.push(format!("tags={}", tags.join(",")));
    }
    if let Some(os) = meta.os.as_deref().map(clean).filter(|s| !s.is_empty()) {
        fields.push(format!("os={os}"));
    }
    if meta.favorite {
        fields.push("favorite=true".to_string());
    }

    (!fields.is_empty()).then(|| format!("{META_PREFIX} {}", fields.join("; ")))
}

/// Liste les hôtes configurés, résolus par OpenSSH et enrichis des métadonnées.
pub fn list_hosts() -> Result<Vec<Host>, AppError> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path)?;
    Ok(parse_entries(&content)
        .into_iter()
        .map(|(alias, meta)| {
            let mut host = resolve_host(&alias);
            host.group = meta.group;
            host.color = meta.color;
            host.tags = meta.tags;
            host.favorite = meta.favorite;
            host.os = meta.os;
            host
        })
        .collect())
}

/// Extrait les couples (alias, métadonnées) du fichier.
///
/// Les motifs génériques (`*`, `?`, négations `!`) sont ignorés : ils ne
/// désignent pas un hôte concret.
fn parse_entries(content: &str) -> Vec<(String, Meta)> {
    let mut entries: Vec<(String, Meta)> = Vec::new();
    let mut pending = Meta::default();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with(META_PREFIX) {
            pending = parse_meta(trimmed);
            continue;
        }
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
            if !entries.iter().any(|(a, _)| a == pattern) {
                entries.push((pattern.to_string(), pending.clone()));
            }
        }
        // Les métadonnées ne valent que pour le bloc qui suit immédiatement.
        pending = Meta::default();
    }

    entries
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
        group: None,
        color: None,
        tags: Vec::new(),
        favorite: false,
        os: None,
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

/// Rend le bloc complet : commentaire de métadonnées puis directives OpenSSH.
fn render_block(input: &ConfigHostInput) -> Vec<String> {
    fn push(lines: &mut Vec<String>, key: &str, value: &str) {
        let value = value.trim();
        if !value.is_empty() {
            lines.push(format!("    {key} {value}"));
        }
    }

    let mut lines = Vec::new();
    if let Some(meta) = render_meta(input) {
        lines.push(meta);
    }
    lines.push(format!("Host {}", input.alias.trim()));

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
fn write_config(path: &Path, lines: &[String]) -> Result<(), AppError> {
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

/// Refuse un alias vide ou composé de plusieurs mots.
fn validate_alias(alias: &str) -> Result<(), AppError> {
    if alias.trim().is_empty() || alias.split_whitespace().count() != 1 {
        return Err(AppError::Command("alias invalide".into()));
    }
    Ok(())
}

/// Ajoute une entrée `Host` à la fin du `~/.ssh/config`.
pub fn create_host(input: &ConfigHostInput) -> Result<(), AppError> {
    validate_alias(&input.alias)?;

    let path = config_path()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let existing = std::fs::read_to_string(&path).unwrap_or_default();

    if parse_entries(&existing)
        .iter()
        .any(|(a, _)| a == input.alias.trim())
    {
        return Err(AppError::Command(format!(
            "l'hôte « {} » existe déjà",
            input.alias.trim()
        )));
    }

    let mut lines: Vec<String> = existing.lines().map(str::to_string).collect();
    if lines.last().is_some_and(|l| !l.trim().is_empty()) {
        lines.push(String::new());
    }
    lines.extend(render_block(input));

    write_config(&path, &lines)
}

/// Met à jour (ou renomme) une entrée `Host` du `~/.ssh/config`.
///
/// Le bloc existant est remplacé intégralement ; le reste du fichier
/// (commentaires, autres hôtes, `Include`) est préservé tel quel.
pub fn update_host(alias: &str, input: &ConfigHostInput) -> Result<(), AppError> {
    validate_alias(&input.alias)?;
    rewrite(alias, Some(input))
}

/// Supprime une entrée `Host` du `~/.ssh/config`.
pub fn delete_host(alias: &str) -> Result<(), AppError> {
    rewrite(alias, None)
}

/// Remplace le bloc d'un alias par `replacement`, ou le supprime si `None`.
/// Renseigne le seul système d'exploitation d'un hôte du fichier.
///
/// Ne touche qu'à la ligne `# galvus:` et laisse le bloc `Host` intact. Passer
/// par `update_host` serait destructeur : les champs de `Host` sont résolus par
/// `ssh -G`, qui matérialise les valeurs par défaut — on réécrirait dans le
/// fichier un `Port 22` et une `IdentityFile` que l'utilisateur n'a jamais
/// écrits.
pub fn set_host_os(alias: &str, os: Option<&str>) -> Result<(), AppError> {
    let path = config_path()?;
    let content = std::fs::read_to_string(&path)?;

    let mut out: Vec<String> = Vec::new();
    let mut found = false;

    for line in content.lines() {
        if !found && is_host_line_for(line, alias) {
            // Métadonnées existantes du bloc, à compléter plutôt qu'à remplacer.
            let mut meta = Meta::default();
            if out
                .last()
                .is_some_and(|l| l.trim().starts_with(META_PREFIX))
            {
                meta = parse_meta(&out.pop().unwrap_or_default());
            }
            meta.os = os.map(str::to_string);

            if let Some(rendered) = render_meta_fields(&meta) {
                out.push(rendered);
            }
            found = true;
        }
        out.push(line.to_string());
    }

    if !found {
        return Err(AppError::Command(format!("hôte « {alias} » introuvable")));
    }

    write_config(&path, &out)
}

fn rewrite(alias: &str, replacement: Option<&ConfigHostInput>) -> Result<(), AppError> {
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
            // Le commentaire de métadonnées appartient au bloc : on le retire.
            if out
                .last()
                .is_some_and(|l| l.trim().starts_with(META_PREFIX))
            {
                out.pop();
            }
            if let Some(input) = replacement {
                out.extend(render_block(input));
            }
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
    use super::{parse_entries, parse_meta, render_meta};
    use crate::models::ConfigHostInput;

    fn aliases(content: &str) -> Vec<String> {
        parse_entries(content).into_iter().map(|(a, _)| a).collect()
    }

    #[test]
    fn extrait_les_alias_simples() {
        let cfg = "Host web\n  HostName 1.2.3.4\nHost db\n  User root\n";
        assert_eq!(aliases(cfg), vec!["web", "db"]);
    }

    #[test]
    fn ignore_les_motifs_generiques_et_commentaires() {
        let cfg = "# commentaire\nHost *\n  ForwardAgent yes\nHost prod !staging\n";
        assert_eq!(aliases(cfg), vec!["prod"]);
    }

    #[test]
    fn gere_plusieurs_alias_sur_une_ligne_sans_doublon() {
        let cfg = "Host a b\nHost b c\n";
        assert_eq!(aliases(cfg), vec!["a", "b", "c"]);
    }

    #[test]
    fn associe_les_metadonnees_au_bloc_suivant() {
        let cfg = "# galvus: group=Prod; color=#4C8DFF; tags=web,eu; favorite=true\n\
                   Host web\n  HostName 1.2.3.4\nHost autre\n";
        let entries = parse_entries(cfg);

        let (alias, meta) = &entries[0];
        assert_eq!(alias, "web");
        assert_eq!(meta.group.as_deref(), Some("Prod"));
        assert_eq!(meta.color.as_deref(), Some("#4C8DFF"));
        assert_eq!(meta.tags, vec!["web", "eu"]);
        assert!(meta.favorite);

        // Elles ne débordent pas sur l'entrée suivante.
        assert!(entries[1].1.group.is_none());
        assert!(!entries[1].1.favorite);
    }

    #[test]
    fn le_rendu_des_metadonnees_est_relisible() {
        let input = ConfigHostInput {
            alias: "web".into(),
            hostname: None,
            user: None,
            port: None,
            identity_file: None,
            proxy_jump: None,
            group: Some("Prod".into()),
            color: Some("#23C48A".into()),
            tags: vec!["nginx".into(), "eu".into()],
            favorite: true,
            os: Some("ubuntu".into()),
        };
        let meta = parse_meta(&render_meta(&input).unwrap());

        assert_eq!(meta.group.as_deref(), Some("Prod"));
        assert_eq!(meta.color.as_deref(), Some("#23C48A"));
        assert_eq!(meta.tags, vec!["nginx", "eu"]);
        assert_eq!(meta.os.as_deref(), Some("ubuntu"));
        assert!(meta.favorite);
    }

    #[test]
    fn pas_de_commentaire_quand_il_n_y_a_rien_a_conserver() {
        let input = ConfigHostInput {
            alias: "web".into(),
            hostname: None,
            user: None,
            port: None,
            identity_file: None,
            proxy_jump: None,
            group: None,
            color: None,
            tags: vec![],
            favorite: false,
            os: None,
        };
        assert!(render_meta(&input).is_none());
    }
}
