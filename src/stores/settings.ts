import { ref, watch } from "vue";
import { defineStore } from "pinia";

const STORAGE_KEY = "galvus.settings.v1";

/** Mode d'affichage de la liste des serveurs. */
export type ServersView = "grid" | "list";
/** Critère de tri des serveurs. */
export type ServersSort = "name" | "favorite" | "recent";

interface PersistedSettings {
  terminalFontSize: number;
  terminalFontFamily: string;
  serversView: ServersView;
  serversSort: ServersSort;
}

const DEFAULTS: PersistedSettings = {
  terminalFontSize: 13,
  terminalFontFamily:
    '"JetBrains Mono", "SF Mono", ui-monospace, Menlo, Consolas, monospace',
  serversView: "grid",
  serversSort: "name",
};

function read(): PersistedSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    return { ...DEFAULTS, ...(JSON.parse(raw) as Partial<PersistedSettings>) };
  } catch {
    return { ...DEFAULTS };
  }
}

/** Préférences utilisateur persistées (hors thème, géré par le store theme). */
export const useSettingsStore = defineStore("settings", () => {
  const initial = read();
  const terminalFontSize = ref(initial.terminalFontSize);
  const terminalFontFamily = ref(initial.terminalFontFamily);
  const serversView = ref<ServersView>(initial.serversView);
  const serversSort = ref<ServersSort>(initial.serversSort);

  watch([terminalFontSize, terminalFontFamily, serversView, serversSort], () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        terminalFontSize: terminalFontSize.value,
        terminalFontFamily: terminalFontFamily.value,
        serversView: serversView.value,
        serversSort: serversSort.value,
      }),
    );
  });

  function reset(): void {
    terminalFontSize.value = DEFAULTS.terminalFontSize;
    terminalFontFamily.value = DEFAULTS.terminalFontFamily;
  }

  return { terminalFontSize, terminalFontFamily, serversView, serversSort, reset };
});
