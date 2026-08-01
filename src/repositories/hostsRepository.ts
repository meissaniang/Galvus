import { invoke } from "@tauri-apps/api/core";
import type { Host } from "@/types/ssh";

/**
 * Accès aux hôtes SSH. Unique point d'appel de la commande Tauri `list_hosts`
 * (aucun composant n'appelle `invoke` directement).
 */
export const hostsRepository = {
  list(): Promise<Host[]> {
    return invoke<Host[]>("list_hosts");
  },
};
