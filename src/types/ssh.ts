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
  tags: string[];
  group: string | null;
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
  tags: string[];
  group: string | null;
}

/** Champs éditables d'une entrée Host du ~/.ssh/config. */
export interface ConfigHostInput {
  alias: string;
  hostname: string | null;
  user: string | null;
  port: number | null;
  identityFile: string | null;
  proxyJump: string | null;
}

/** Type de tunnel SSH. */
export type TunnelKind = "local" | "remote" | "dynamic";

/** Un tunnel SSH (redirection de port). */
export interface Tunnel {
  id: number;
  name: string;
  kind: TunnelKind;
  sshTarget: string;
  listenPort: number;
  targetHost: string | null;
  targetPort: number | null;
}

/** Données de création d'un tunnel (sans identifiant). */
export interface TunnelInput {
  name: string;
  kind: TunnelKind;
  sshTarget: string;
  listenPort: number;
  targetHost: string | null;
  targetPort: number | null;
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
  /** Vrai si la clé privée est protégée par une passphrase. */
  encrypted: boolean;
  /** Vrai si les permissions de la clé privée sont trop ouvertes (ssh la refuse). */
  insecurePermissions: boolean;
}
