import { computed, ref } from "vue";
import { defineStore } from "pinia";

/** Un pane = une session PTY vivante (arguments `ssh`). */
export interface Pane {
  id: string;
  label: string;
  args: string[];
}

/** Sens de découpe des panes au sein d'un onglet. */
export type SplitDirection = "row" | "column";

/** Un onglet contient un ou plusieurs panes (split simple, un seul niveau). */
export interface TerminalTab {
  id: string;
  direction: SplitDirection;
  panes: Pane[];
  activePaneId: string;
}

function uuid(): string {
  return crypto.randomUUID();
}

/**
 * Onglets et splits de terminal. Source de vérité de l'espace terminal : chaque
 * pane correspond à une session PTY vivante.
 */
export const useConnectionsStore = defineStore("connections", () => {
  const tabs = ref<TerminalTab[]>([]);
  const activeTabId = ref<string | null>(null);

  const activeTab = computed(
    () => tabs.value.find((t) => t.id === activeTabId.value) ?? null,
  );
  const count = computed(() => tabs.value.length);

  /** Libellé d'onglet : nom du 1er pane (+N si splits). */
  function tabTitle(tab: TerminalTab): string {
    const base = tab.panes[0]?.label ?? "Session";
    return tab.panes.length > 1 ? `${base} +${tab.panes.length - 1}` : base;
  }

  /** Ouvre un nouvel onglet (1 pane) et le rend actif. */
  function open(label: string, args: string[]): string {
    const paneId = uuid();
    const tab: TerminalTab = {
      id: uuid(),
      direction: "row",
      panes: [{ id: paneId, label, args }],
      activePaneId: paneId,
    };
    tabs.value.push(tab);
    activeTabId.value = tab.id;
    return tab.id;
  }

  /** Découpe le pane actif de l'onglet actif (nouvelle session, même cible). */
  function splitActive(direction: SplitDirection): void {
    const tab = activeTab.value;
    if (!tab) return;
    const source = tab.panes.find((p) => p.id === tab.activePaneId) ?? tab.panes[0];
    if (!source) return;
    const pane: Pane = { id: uuid(), label: source.label, args: [...source.args] };
    tab.direction = direction;
    tab.panes.push(pane);
    tab.activePaneId = pane.id;
  }

  /** Ferme un pane ; si c'était le dernier de l'onglet, ferme l'onglet. */
  function closePane(tabId: string, paneId: string): void {
    const tab = tabs.value.find((t) => t.id === tabId);
    if (!tab) return;
    const index = tab.panes.findIndex((p) => p.id === paneId);
    if (index === -1) return;
    tab.panes.splice(index, 1);
    if (tab.panes.length === 0) {
      closeTab(tabId);
      return;
    }
    if (tab.activePaneId === paneId) {
      tab.activePaneId = (tab.panes[index] ?? tab.panes[index - 1]).id;
    }
  }

  /** Ferme un onglet entier et sélectionne un voisin. */
  function closeTab(tabId: string): void {
    const index = tabs.value.findIndex((t) => t.id === tabId);
    if (index === -1) return;
    tabs.value.splice(index, 1);
    if (activeTabId.value === tabId) {
      activeTabId.value = (tabs.value[index] ?? tabs.value[index - 1])?.id ?? null;
    }
  }

  function setActiveTab(id: string): void {
    if (tabs.value.some((t) => t.id === id)) activeTabId.value = id;
  }

  function setActivePane(tabId: string, paneId: string): void {
    const tab = tabs.value.find((t) => t.id === tabId);
    if (tab && tab.panes.some((p) => p.id === paneId)) tab.activePaneId = paneId;
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    count,
    tabTitle,
    open,
    splitActive,
    closePane,
    closeTab,
    setActiveTab,
    setActivePane,
  };
});
