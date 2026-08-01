import { invoke } from "@tauri-apps/api/core";
import type { SshKey } from "@/types/ssh";

/**
 * Accès aux clés SSH. Unique point d'appel de la commande Tauri `list_keys`.
 */
export const keysRepository = {
  list(): Promise<SshKey[]> {
    return invoke<SshKey[]>("list_keys");
  },
};
