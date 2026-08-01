import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { SshKey } from "@/types/ssh";

/** Paramètres de génération d'une clé. */
export interface GenerateKeyInput {
  name: string;
  keyType: "ed25519" | "rsa" | "ecdsa";
  comment: string;
  passphrase: string;
}

/**
 * Accès aux clés SSH. Unique point d'appel des commandes Tauri `key_*` /
 * `list_keys` et du sélecteur de fichier natif.
 */
export const keysRepository = {
  list(): Promise<SshKey[]> {
    return invoke<SshKey[]>("list_keys");
  },

  generate(input: GenerateKeyInput): Promise<SshKey> {
    return invoke<SshKey>("key_generate", {
      name: input.name,
      keyType: input.keyType,
      comment: input.comment,
      passphrase: input.passphrase,
    });
  },

  import(source: string, name: string): Promise<SshKey> {
    return invoke<SshKey>("key_import", { source, name });
  },

  remove(name: string): Promise<void> {
    return invoke<void>("key_delete", { name });
  },

  /** Contenu de la clé publique (pour copie dans le presse-papiers). */
  readPublic(name: string): Promise<string> {
    return invoke<string>("key_read_public", { name });
  },

  /** Contenu de la clé privée (affichage local uniquement). */
  readPrivate(name: string): Promise<string> {
    return invoke<string>("key_read_private", { name });
  },

  /** Ouvre le sélecteur de fichier natif pour choisir une clé privée à importer. */
  async pickKeyFile(): Promise<string | null> {
    const selected = await open({
      multiple: false,
      directory: false,
      title: "Choisir une clé privée à importer",
    });
    return typeof selected === "string" ? selected : null;
  },
};
