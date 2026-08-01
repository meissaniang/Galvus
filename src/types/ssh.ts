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

/** Un serveur enregistré par l'utilisateur (persisté dans la base chiffrée). */
export interface Server {
  id: number;
  name: string;
  hostname: string;
  port: number;
  username: string | null;
  identityFile: string | null;
  color: string | null;
  favorite: boolean;
}

/** Données de création/édition d'un serveur (sans identifiant). */
export interface ServerInput {
  name: string;
  hostname: string;
  port: number;
  username: string | null;
  identityFile: string | null;
  color: string | null;
  favorite: boolean;
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
