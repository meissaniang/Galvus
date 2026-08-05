import { invoke } from "@tauri-apps/api/core";
import type { ConfigHostInput, Host } from "@/types/ssh";

/**
 * Accès aux hôtes SSH du `~/.ssh/config`. Unique point d'appel des commandes
 * Tauri correspondantes (aucun composant n'appelle `invoke` directement).
 */
export const hostsRepository = {
  list(): Promise<Host[]> {
    return invoke<Host[]>("list_hosts");
  },

  /** Ajoute une entrée au fichier de config. */
  create(input: ConfigHostInput): Promise<void> {
    return invoke<void>("config_host_create", { input });
  },

  /** Met à jour (ou renomme) une entrée du fichier de config. */
  update(alias: string, input: ConfigHostInput): Promise<void> {
    return invoke<void>("config_host_update", { alias, input });
  },

  /** Supprime une entrée du fichier de config. */
  remove(alias: string): Promise<void> {
    return invoke<void>("config_host_delete", { alias });
  },
};
