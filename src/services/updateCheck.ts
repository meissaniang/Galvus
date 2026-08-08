import { getVersion } from "@tauri-apps/api/app";

/**
 * Vérification de disponibilité d'une nouvelle version.
 *
 * C'est la **seule** requête réseau de Galvus, et elle est désactivable dans
 * les Paramètres. Elle interroge l'API publique de GitHub, n'envoie aucune
 * donnée personnelle et ne télécharge rien : la mise à jour reste à la main de
 * l'utilisateur.
 */

const RELEASES_API = "https://api.github.com/repos/meissaniang/Galvus/releases/latest";
const RELEASES_PAGE = "https://github.com/meissaniang/Galvus/releases/latest";

const LAST_CHECK_KEY = "galvus.update.lastCheck";
const DISMISSED_KEY = "galvus.update.dismissed";

/** Une seule vérification par jour suffit. */
const CHECK_INTERVAL_MS = 24 * 60 * 60 * 1000;

export interface UpdateInfo {
  version: string;
  url: string;
}

/** Compare deux versions `x.y.z` ; positif si `a` est plus récente que `b`. */
export function compareVersions(a: string, b: string): number {
  const parse = (v: string) =>
    v
      .replace(/^v/, "")
      .split(".")
      .map((n) => Number.parseInt(n, 10) || 0);
  const left = parse(a);
  const right = parse(b);
  for (let i = 0; i < Math.max(left.length, right.length); i++) {
    const diff = (left[i] ?? 0) - (right[i] ?? 0);
    if (diff !== 0) return diff;
  }
  return 0;
}

/** Vrai si l'intervalle est écoulé depuis la dernière vérification. */
function isDue(): boolean {
  const last = Number.parseInt(localStorage.getItem(LAST_CHECK_KEY) ?? "0", 10);
  return Number.isNaN(last) || Date.now() - last > CHECK_INTERVAL_MS;
}

/** Masque le bandeau pour cette version précise. */
export function dismiss(version: string): void {
  localStorage.setItem(DISMISSED_KEY, version);
}

/**
 * Retourne la version disponible si elle est plus récente que celle installée
 * et n'a pas déjà été écartée, sinon `null`.
 *
 * Toute erreur — réseau coupé, API indisponible — est silencieuse : une
 * vérification de version ne doit jamais gêner l'utilisation.
 */
export async function checkForUpdate(force = false): Promise<UpdateInfo | null> {
  if (!force && !isDue()) return null;

  try {
    const response = await fetch(RELEASES_API, {
      headers: { Accept: "application/vnd.github+json" },
    });
    if (!response.ok) return null;

    const release = (await response.json()) as { tag_name?: string; html_url?: string };
    const latest = release.tag_name?.replace(/^v/, "");
    if (!latest) return null;

    localStorage.setItem(LAST_CHECK_KEY, String(Date.now()));

    const current = await getVersion();
    if (compareVersions(latest, current) <= 0) return null;
    if (!force && localStorage.getItem(DISMISSED_KEY) === latest) return null;

    return { version: latest, url: release.html_url ?? RELEASES_PAGE };
  } catch {
    return null;
  }
}
