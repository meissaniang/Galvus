//! Repository des groupes : tout l'accès SQL à la table `groups` vit ici.
//!
//! Un groupe est un dossier de l'écran Serveurs. Il est identifié par son
//! **chemin complet** (`Production/Web`), et non par un couple nom + parent.
//!
//! Ce choix vient de la double origine des serveurs : ceux du `~/.ssh/config`
//! ne peuvent porter que du texte dans leur commentaire `# galvus:`, jamais un
//! identifiant de ligne. Le chemin est donc la seule référence que les deux
//! sources savent exprimer ; stocker un `parent_id` en plus créerait une
//! seconde représentation de la même hiérarchie, à resynchroniser sans cesse.
//!
//! La table sert à faire exister les dossiers **vides** : un dossier qui
//! contient déjà un serveur se déduirait du serveur, mais on ne pourrait ni en
//! créer un d'avance, ni le garder après en avoir sorti la dernière entrée.

use rusqlite::{params, Connection};

use crate::errors::AppError;

/// Séparateur de niveaux dans un chemin de groupe.
pub const SEPARATOR: char = '/';

/// Nettoie un chemin : segments rognés, vides supprimés.
///
/// Rend `None` pour la racine (chaîne vide), ce qui est un chemin valide :
/// c'est celui des serveurs rangés hors de tout dossier.
pub fn normalize(path: &str) -> Option<String> {
    let cleaned = path
        .split(SEPARATOR)
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(&SEPARATOR.to_string());
    (!cleaned.is_empty()).then_some(cleaned)
}

/// Chemin du dossier parent, ou `None` si le dossier est à la racine.
pub fn parent_of(path: &str) -> Option<String> {
    path.rsplit_once(SEPARATOR)
        .map(|(parent, _)| parent.to_string())
}

/// Vrai si `path` est `ancestor` lui-même ou l'un de ses descendants.
fn is_within(path: &str, ancestor: &str) -> bool {
    path == ancestor
        || (path.starts_with(ancestor) && path[ancestor.len()..].starts_with(SEPARATOR))
}

/// Remplace le préfixe `from` par `to` dans un chemin qui en descend.
fn rebase(path: &str, from: &str, to: &str) -> String {
    format!("{to}{}", &path[from.len()..])
}

/// Liste les chemins de tous les groupes, triés.
pub fn list(conn: &Connection) -> Result<Vec<String>, AppError> {
    let mut stmt = conn.prepare("SELECT path FROM groups ORDER BY path COLLATE NOCASE")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut paths = Vec::new();
    for row in rows {
        paths.push(row?);
    }
    Ok(paths)
}

/// Crée un groupe et tous ses ancêtres manquants. Idempotent.
///
/// Les ancêtres sont créés d'office : un dossier `Production/Web` sans
/// `Production` serait invisible dans l'arborescence.
pub fn create(conn: &Connection, path: &str) -> Result<String, AppError> {
    let path = normalize(path)
        .ok_or_else(|| AppError::Invalid("Le nom du dossier ne peut pas être vide.".into()))?;

    let mut prefix = String::new();
    for segment in path.split(SEPARATOR) {
        if !prefix.is_empty() {
            prefix.push(SEPARATOR);
        }
        prefix.push_str(segment);
        conn.execute(
            "INSERT OR IGNORE INTO groups (path) VALUES (?1)",
            params![prefix],
        )?;
    }
    Ok(path)
}

/// Renomme le dernier segment d'un groupe, et suit ses descendants.
///
/// Le chemin étant la référence des serveurs, un renommage doit réécrire
/// d'un même geste les sous-dossiers et les `group_name` concernés. Les hôtes
/// du `~/.ssh/config` sont hors de cette base : c'est au frontend de les
/// mettre à jour (voir `stores/groups.ts`).
pub fn rename(conn: &Connection, path: &str, new_name: &str) -> Result<String, AppError> {
    let path = normalize(path).ok_or_else(|| AppError::Invalid("Dossier introuvable.".into()))?;
    let name = new_name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid(
            "Le nom du dossier ne peut pas être vide.".into(),
        ));
    }
    if name.contains(SEPARATOR) {
        return Err(AppError::Invalid(format!(
            "Un nom de dossier ne peut pas contenir « {SEPARATOR} »."
        )));
    }

    let target = match parent_of(&path) {
        Some(parent) => format!("{parent}{SEPARATOR}{name}"),
        None => name.to_string(),
    };
    if target == path {
        return Ok(path);
    }
    if list(conn)?.iter().any(|existing| existing == &target) {
        return Err(AppError::Invalid(format!(
            "Un dossier « {name} » existe déjà ici."
        )));
    }

    move_subtree(conn, &path, &target)?;
    Ok(target)
}

/// Déplace un groupe (et son contenu) sous un autre dossier.
///
/// `new_parent` vide signifie la racine. Refuse de déplacer un dossier dans
/// l'un de ses propres descendants, qui le détacherait de l'arborescence.
pub fn move_to(conn: &Connection, path: &str, new_parent: &str) -> Result<String, AppError> {
    let path = normalize(path).ok_or_else(|| AppError::Invalid("Dossier introuvable.".into()))?;
    let parent = normalize(new_parent);

    let name = path.rsplit(SEPARATOR).next().unwrap_or(&path).to_string();
    let target = match &parent {
        Some(parent) => format!("{parent}{SEPARATOR}{name}"),
        None => name.clone(),
    };
    if target == path {
        return Ok(path);
    }
    if let Some(parent) = &parent {
        if is_within(parent, &path) {
            return Err(AppError::Invalid(
                "Un dossier ne peut pas être déplacé dans l'un de ses sous-dossiers.".into(),
            ));
        }
    }
    if list(conn)?.iter().any(|existing| existing == &target) {
        return Err(AppError::Invalid(format!(
            "Un dossier « {name} » existe déjà à destination."
        )));
    }

    move_subtree(conn, &path, &target)?;
    Ok(target)
}

/// Réécrit `from` en `to` pour le dossier, ses descendants et les serveurs.
fn move_subtree(conn: &Connection, from: &str, to: &str) -> Result<(), AppError> {
    // Réécriture calculée en Rust plutôt qu'en SQL : un `LIKE 'préfixe%'`
    // demanderait d'échapper `%` et `_`, que rien n'interdit dans un nom de
    // dossier. Les volumes en jeu sont de l'ordre de la dizaine.
    let groups: Vec<String> = list(conn)?
        .into_iter()
        .filter(|existing| is_within(existing, from))
        .collect();
    for existing in groups {
        conn.execute(
            "UPDATE groups SET path = ?1 WHERE path = ?2",
            params![rebase(&existing, from, to), existing],
        )?;
    }

    for (id, group) in servers_in(conn, from)? {
        conn.execute(
            "UPDATE servers SET group_name = ?1 WHERE id = ?2",
            params![rebase(&group, from, to), id],
        )?;
    }
    Ok(())
}

/// Supprime un groupe et ses sous-dossiers ; leurs serveurs remontent au parent.
///
/// Rien n'est détruit d'autre que des dossiers : supprimer un rangement ne doit
/// jamais emporter les serveurs qu'il contenait.
pub fn delete(conn: &Connection, path: &str) -> Result<(), AppError> {
    let path = normalize(path).ok_or_else(|| AppError::Invalid("Dossier introuvable.".into()))?;
    let parent = parent_of(&path);

    for (id, _) in servers_in(conn, &path)? {
        conn.execute(
            "UPDATE servers SET group_name = ?1 WHERE id = ?2",
            params![parent, id],
        )?;
    }
    for existing in list(conn)?
        .into_iter()
        .filter(|existing| is_within(existing, &path))
    {
        conn.execute("DELETE FROM groups WHERE path = ?1", params![existing])?;
    }
    Ok(())
}

/// Serveurs rangés dans `path` ou l'un de ses descendants : `(id, groupe)`.
fn servers_in(conn: &Connection, path: &str) -> Result<Vec<(i64, String)>, AppError> {
    let mut stmt =
        conn.prepare("SELECT id, group_name FROM servers WHERE group_name IS NOT NULL")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut found = Vec::new();
    for row in rows {
        let (id, group) = row?;
        if is_within(&group, path) {
            found.push((id, group));
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE groups (id INTEGER PRIMARY KEY AUTOINCREMENT, path TEXT NOT NULL UNIQUE);
             CREATE TABLE servers (id INTEGER PRIMARY KEY AUTOINCREMENT, group_name TEXT);",
        )
        .unwrap();
        conn
    }

    fn add_server(conn: &Connection, group: Option<&str>) -> i64 {
        conn.execute(
            "INSERT INTO servers (group_name) VALUES (?1)",
            params![group],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    fn group_of(conn: &Connection, id: i64) -> Option<String> {
        conn.query_row(
            "SELECT group_name FROM servers WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn normalise_les_chemins() {
        assert_eq!(normalize(" Prod / Web "), Some("Prod/Web".into()));
        assert_eq!(normalize("Prod//Web"), Some("Prod/Web".into()));
        assert_eq!(normalize("  /  "), None);
    }

    #[test]
    fn cree_les_ancetres_manquants() {
        let conn = setup();
        create(&conn, "Production/Europe/Web").unwrap();
        assert_eq!(
            list(&conn).unwrap(),
            vec!["Production", "Production/Europe", "Production/Europe/Web"]
        );
    }

    #[test]
    fn creation_idempotente() {
        let conn = setup();
        create(&conn, "Prod").unwrap();
        create(&conn, "Prod").unwrap();
        assert_eq!(list(&conn).unwrap(), vec!["Prod"]);
    }

    #[test]
    fn renomme_les_descendants_et_les_serveurs() {
        let conn = setup();
        create(&conn, "Prod/Web").unwrap();
        let server = add_server(&conn, Some("Prod/Web"));
        let voisin = add_server(&conn, Some("Prodige"));

        assert_eq!(rename(&conn, "Prod", "Production").unwrap(), "Production");
        assert_eq!(list(&conn).unwrap(), vec!["Production", "Production/Web"]);
        assert_eq!(group_of(&conn, server).as_deref(), Some("Production/Web"));
        // Un préfixe textuel commun ne fait pas un descendant.
        assert_eq!(group_of(&conn, voisin).as_deref(), Some("Prodige"));
    }

    #[test]
    fn refuse_un_nom_deja_pris() {
        let conn = setup();
        create(&conn, "Prod").unwrap();
        create(&conn, "Staging").unwrap();
        assert!(rename(&conn, "Prod", "Staging").is_err());
    }

    #[test]
    fn refuse_un_nom_avec_separateur() {
        let conn = setup();
        create(&conn, "Prod").unwrap();
        assert!(rename(&conn, "Prod", "a/b").is_err());
    }

    #[test]
    fn deplace_un_dossier_sous_un_autre() {
        let conn = setup();
        create(&conn, "Prod/Web").unwrap();
        create(&conn, "Archives").unwrap();
        let server = add_server(&conn, Some("Prod/Web"));

        assert_eq!(move_to(&conn, "Prod", "Archives").unwrap(), "Archives/Prod");
        assert_eq!(
            group_of(&conn, server).as_deref(),
            Some("Archives/Prod/Web")
        );
    }

    #[test]
    fn refuse_de_se_deplacer_dans_son_descendant() {
        let conn = setup();
        create(&conn, "Prod/Web").unwrap();
        assert!(move_to(&conn, "Prod", "Prod/Web").is_err());
    }

    #[test]
    fn suppression_remonte_les_serveurs_au_parent() {
        let conn = setup();
        create(&conn, "Prod/Web/Front").unwrap();
        let profond = add_server(&conn, Some("Prod/Web/Front"));
        let direct = add_server(&conn, Some("Prod/Web"));

        delete(&conn, "Prod/Web").unwrap();
        assert_eq!(list(&conn).unwrap(), vec!["Prod"]);
        assert_eq!(group_of(&conn, profond).as_deref(), Some("Prod"));
        assert_eq!(group_of(&conn, direct).as_deref(), Some("Prod"));
    }

    #[test]
    fn suppression_a_la_racine_degroupe_les_serveurs() {
        let conn = setup();
        create(&conn, "Prod").unwrap();
        let server = add_server(&conn, Some("Prod"));

        delete(&conn, "Prod").unwrap();
        assert!(list(&conn).unwrap().is_empty());
        assert_eq!(group_of(&conn, server), None);
    }
}
