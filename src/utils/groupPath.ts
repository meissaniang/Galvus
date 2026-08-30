/**
 * Chemins de dossiers de l'écran Serveurs.
 *
 * Un dossier est identifié par son chemin complet — `Production/Web` — et la
 * racine par la chaîne vide. Voir `src-tauri/src/database/groups_repository.rs`
 * pour la raison de ce choix : c'est la seule référence qu'un hôte du
 * `~/.ssh/config` sait porter dans son commentaire `# galvus:`.
 */

/** Séparateur de niveaux. Interdit dans un nom de dossier. */
export const SEPARATOR = "/";

/** Découpe un chemin en segments, en ignorant les vides. */
export function segments(path: string): string[] {
  return path
    .split(SEPARATOR)
    .map((segment) => segment.trim())
    .filter(Boolean);
}

/** Recompose un chemin propre à partir de segments ou de sous-chemins. */
export function joinPath(...parts: (string | null | undefined)[]): string {
  return parts
    .filter(Boolean)
    .flatMap((part) => segments(part as string))
    .join(SEPARATOR);
}

/** Chemin nettoyé (segments rognés, vides supprimés). */
export function normalizePath(path: string | null | undefined): string {
  return path ? joinPath(path) : "";
}

/** Dossier parent, ou `""` à la racine. */
export function parentOf(path: string): string {
  const parts = segments(path);
  parts.pop();
  return parts.join(SEPARATOR);
}

/** Dernier segment : le nom affiché du dossier. */
export function nameOf(path: string): string {
  return segments(path).pop() ?? "";
}

/** Le chemin lui-même et tous ses ancêtres, de la racine vers la feuille. */
export function ancestorsOf(path: string): string[] {
  const parts = segments(path);
  return parts.map((_, index) => parts.slice(0, index + 1).join(SEPARATOR));
}

/**
 * Vrai si `path` est `ancestor` ou l'un de ses descendants.
 *
 * Compare segment par segment : `Prodige` ne descend pas de `Prod`, alors
 * qu'une comparaison de préfixe textuel le prétendrait.
 */
export function isWithin(path: string, ancestor: string): boolean {
  if (!ancestor) return true;
  return path === ancestor || path.startsWith(ancestor + SEPARATOR);
}

/** Profondeur d'un chemin ; `0` à la racine. */
export function depthOf(path: string): number {
  return segments(path).length;
}
