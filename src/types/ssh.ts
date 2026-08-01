/**
 * Types partagés côté frontend, alignés sur les modèles Rust (serde camelCase).
 * Voir src-tauri/src/models/.
 */

/** Un hôte SSH résolu depuis ~/.ssh/config via `ssh -G`. */
export interface Host {
  alias: string;
  hostname: string | null;
  user: string | null;
  port: number | null;
  identityFile: string | null;
  proxyJump: string | null;
}

/** Une clé SSH détectée dans ~/.ssh. */
export interface SshKey {
  name: string;
  path: string;
  keyType: string | null;
  bits: number | null;
  fingerprint: string | null;
  comment: string | null;
  hasPrivate: boolean;
}
