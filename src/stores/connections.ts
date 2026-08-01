import { reactive } from "vue";
import { defineStore } from "pinia";

/** Cible d'une connexion terminal : libellé d'affichage + arguments `ssh`. */
export interface ConnectionTarget {
  label: string;
  args: string[];
}

/**
 * Registre des connexions en cours d'ouverture. On y dépose une cible avant de
 * naviguer vers /terminal/:id, ce qui évite de faire transiter les arguments
 * `ssh` par l'URL. Base extensible pour les onglets multiples (Livrable 2).
 */
export const useConnectionsStore = defineStore("connections", () => {
  const targets = reactive(new Map<string, ConnectionTarget>());

  function start(label: string, args: string[]): string {
    const id = crypto.randomUUID();
    targets.set(id, { label, args });
    return id;
  }

  function get(id: string): ConnectionTarget | undefined {
    return targets.get(id);
  }

  function end(id: string): void {
    targets.delete(id);
  }

  return { targets, start, get, end };
});
