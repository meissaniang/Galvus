import { invoke } from "@tauri-apps/api/core";

/**
 * Accès aux dossiers de serveurs (base chiffrée). Unique point d'appel des
 * commandes Tauri `group_*`.
 *
 * Un dossier est désigné par son chemin complet ; voir `utils/groupPath`.
 */
export const groupsRepository = {
  list(): Promise<string[]> {
    return invoke<string[]>("group_list");
  },

  /** Crée le dossier et ses parents manquants. Rend le chemin normalisé. */
  create(path: string): Promise<string> {
    return invoke<string>("group_create", { path });
  },

  /** Renomme le dernier segment. Rend le nouveau chemin. */
  rename(path: string, name: string): Promise<string> {
    return invoke<string>("group_rename", { path, name });
  },

  /** Déplace le dossier sous `parent` (`""` = racine). Rend le nouveau chemin. */
  move(path: string, parent: string): Promise<string> {
    return invoke<string>("group_move", { path, parent });
  },

  /** Supprime le dossier et ses sous-dossiers ; les serveurs remontent au parent. */
  remove(path: string): Promise<void> {
    return invoke<void>("group_delete", { path });
  },
};
