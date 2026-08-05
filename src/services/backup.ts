import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { ConfigHostInput, ServerInput, TunnelInput } from "@/types/ssh";

/**
 * Sauvegarde et restauration de la configuration Galvus.
 *
 * Le fichier rassemble ce qui, autrement, vit à trois endroits distincts : la
 * base chiffrée (serveurs, tunnels), le `~/.ssh/config` et les préférences
 * d'affichage.
 *
 * Il ne contient **aucun secret** : ni clé privée, ni passphrase. Les clés
 * restent dans `~/.ssh` et les passphrases dans le coffre du système.
 */

/** Version du format, pour refuser proprement un fichier incompatible. */
const FORMAT_VERSION = 1;

export interface BackupFile {
  galvus: number;
  exportedAt: string;
  servers: ServerInput[];
  configHosts: ConfigHostInput[];
  tunnels: TunnelInput[];
  settings: Record<string, string>;
}

export interface ImportSummary {
  servers: number;
  configHosts: number;
  tunnels: number;
  settingsRestored: boolean;
  skipped: number;
}

/** Clés de préférences conservées dans le stockage local. */
const SETTINGS_KEYS = ["galvus.settings.v1", "galvus.theme.mode.v2"];

function collectSettings(): Record<string, string> {
  const out: Record<string, string> = {};
  for (const key of SETTINGS_KEYS) {
    const value = localStorage.getItem(key);
    if (value !== null) out[key] = value;
  }
  return out;
}

/**
 * Propose un emplacement puis écrit la sauvegarde.
 * Retourne le chemin choisi, ou `null` si l'utilisateur annule.
 */
export async function exportBackup(data: Omit<BackupFile, "galvus" | "exportedAt">) {
  const date = new Date().toISOString().slice(0, 10);
  const path = await save({
    title: "Exporter la configuration Galvus",
    defaultPath: `galvus-${date}.json`,
    filters: [{ name: "Sauvegarde Galvus", extensions: ["json"] }],
  });
  if (!path) return null;

  const file: BackupFile = {
    galvus: FORMAT_VERSION,
    exportedAt: new Date().toISOString(),
    ...data,
    settings: collectSettings(),
  };
  await invoke("backup_write", { path, content: JSON.stringify(file, null, 2) });
  return path;
}

/**
 * Demande un fichier et le valide.
 * Retourne son contenu, ou `null` si l'utilisateur annule.
 */
export async function pickBackup(): Promise<BackupFile | null> {
  const selected = await open({
    title: "Importer une configuration Galvus",
    multiple: false,
    directory: false,
    filters: [{ name: "Sauvegarde Galvus", extensions: ["json"] }],
  });
  if (typeof selected !== "string") return null;

  const raw = await invoke<string>("backup_read", { path: selected });
  let file: BackupFile;
  try {
    file = JSON.parse(raw) as BackupFile;
  } catch {
    throw new Error("fichier illisible : ce n'est pas un JSON valide");
  }
  if (file.galvus !== FORMAT_VERSION) {
    throw new Error(
      `format de sauvegarde non pris en charge (version ${file.galvus ?? "inconnue"})`,
    );
  }
  return file;
}

/** Réapplique les préférences d'affichage contenues dans une sauvegarde. */
export function restoreSettings(settings: Record<string, string> | undefined): boolean {
  if (!settings) return false;
  let restored = false;
  for (const key of SETTINGS_KEYS) {
    const value = settings[key];
    if (typeof value === "string") {
      localStorage.setItem(key, value);
      restored = true;
    }
  }
  return restored;
}
