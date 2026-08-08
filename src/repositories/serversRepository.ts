import { invoke } from "@tauri-apps/api/core";
import type { Server, ServerInput } from "@/types/ssh";

/**
 * Accès aux serveurs enregistrés (base chiffrée). Unique point d'appel des
 * commandes Tauri `server_*`.
 */
export const serversRepository = {
  list(): Promise<Server[]> {
    return invoke<Server[]>("server_list");
  },

  create(input: ServerInput): Promise<Server> {
    return invoke<Server>("server_create", { input });
  },

  update(id: number, input: ServerInput): Promise<Server> {
    return invoke<Server>("server_update", { id, input });
  },

  /** Renseigne le seul système, sans réécrire le reste de la fiche. */
  setOs(id: number, os: string | null): Promise<void> {
    return invoke<void>("server_set_os", { id, os });
  },

  remove(id: number): Promise<void> {
    return invoke<void>("server_delete", { id });
  },
};
