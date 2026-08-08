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
  /** Métadonnées Galvus, stockées en commentaire dans le fichier. */
  group: string | null;
  color: string | null;
  tags: string[];
  favorite: boolean;
  /** Système détecté ou choisi (voir `OsId`). */
  os: string | null;
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
  os: string | null;
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
  os: string | null;
}

/** Champs éditables d'une entrée Host du ~/.ssh/config. */
export interface ConfigHostInput {
  alias: string;
  hostname: string | null;
  user: string | null;
  port: number | null;
  identityFile: string | null;
  proxyJump: string | null;
  group: string | null;
  color: string | null;
  tags: string[];
  favorite: boolean;
  os: string | null;
}

/** Origine d'un serveur affiché : base chiffrée ou fichier de configuration. */
export type ServerSource = "local" | "config";

/**
 * Vue unifiée d'un serveur, quelle que soit son origine. Permet d'afficher et
 * de manipuler de la même façon les entrées de la base et celles du
 * `~/.ssh/config`.
 */
export interface ServerItem {
  key: string;
  source: ServerSource;
  /** Identifiant en base (source « local ») ou alias (source « config »). */
  id: number | null;
  alias: string;
  name: string;
  hostname: string;
  port: number;
  username: string | null;
  identityFile: string | null;
  color: string | null;
  favorite: boolean;
  tags: string[];
  group: string | null;
  os: string | null;
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
  /** Vrai si la clé est chargée dans l'agent SSH (passphrase déjà déverrouillée). */
  inAgent: boolean;
}
