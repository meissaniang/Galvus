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

  /** Enregistre le contenu modifié de la clé privée. */
  writePrivate(name: string, content: string): Promise<void> {
    return invoke<void>("key_write_private", { name, content });
  },

  /** Enregistre le contenu modifié de la clé publique. */
  writePublic(name: string, content: string): Promise<void> {
    return invoke<void>("key_write_public", { name, content });
  },

  /** Charge la clé dans l'agent SSH (+ Trousseau macOS). */
  addToAgent(name: string, passphrase: string, configureSsh: boolean): Promise<void> {
    return invoke<void>("key_add_to_agent", { name, passphrase, configureSsh });
  },

  /** Retire la clé de l'agent SSH. */
  removeFromAgent(name: string): Promise<void> {
    return invoke<void>("key_remove_from_agent", { name });
  },

  /** Restaure les permissions 600 sur la clé privée. */
  fixPermissions(name: string): Promise<void> {
    return invoke<void>("key_fix_permissions", { name });
  },

  /** Ajoute, change ou retire la passphrase (nouvelle vide = retrait). */
  changePassphrase(
    name: string,
    oldPassphrase: string,
    newPassphrase: string,
  ): Promise<SshKey> {
    return invoke<SshKey>("key_change_passphrase", {
      name,
      oldPassphrase,
      newPassphrase,
    });
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
