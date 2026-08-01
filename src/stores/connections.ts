import { computed, ref } from "vue";
import { defineStore } from "pinia";

/** Un onglet de terminal : libellé + arguments `ssh`. */
export interface TerminalTab {
  id: string;
  label: string;
  args: string[];
}

/**
 * Onglets de terminal ouverts. Source de vérité de l'espace terminal : chaque
 * onglet correspond à une session PTY vivante. Base pour les splits (à venir).
 */
export const useConnectionsStore = defineStore("connections", () => {
  const tabs = ref<TerminalTab[]>([]);
  const activeId = ref<string | null>(null);

  const activeTab = computed(() =>
    tabs.value.find((t) => t.id === activeId.value) ?? null,
  );
  const count = computed(() => tabs.value.length);

  /** Ouvre un nouvel onglet et le rend actif. Retourne son identifiant. */
  function open(label: string, args: string[]): string {
    const id = crypto.randomUUID();
    tabs.value.push({ id, label, args });
    activeId.value = id;
    return id;
  }

  /** Ferme un onglet et sélectionne un voisin comme actif. */
  function close(id: string): void {
    const index = tabs.value.findIndex((t) => t.id === id);
    if (index === -1) return;
    tabs.value.splice(index, 1);
    if (activeId.value === id) {
      const neighbour = tabs.value[index] ?? tabs.value[index - 1] ?? null;
      activeId.value = neighbour?.id ?? null;
    }
  }

  function setActive(id: string): void {
    if (tabs.value.some((t) => t.id === id)) {
      activeId.value = id;
    }
  }

  return { tabs, activeId, activeTab, count, open, close, setActive };
});
