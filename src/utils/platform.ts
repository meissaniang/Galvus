/**
 * Aides liées à la plateforme hôte.
 *
 * Les raccourcis sont gérés avec `metaKey || ctrlKey` dans le code ; seuls les
 * libellés affichés changent (⌘ sur macOS, Ctrl ailleurs).
 */

/** Vrai si l'application tourne sur macOS. */
export const isMac = /Mac|iPhone|iPad/.test(navigator.platform ?? navigator.userAgent);

/** Préfixe de la touche de modification à afficher. */
export const modKey = isMac ? "⌘" : "Ctrl+";

/** Compose un libellé de raccourci : `shortcut("T")` → « ⌘T » ou « Ctrl+T ». */
export function shortcut(keys: string): string {
  return `${modKey}${keys}`;
}
