import { ref, watch } from "vue";
import { defineStore } from "pinia";

const STORAGE_KEY = "galvus.settings.v1";

/** Mode d'affichage de la liste des serveurs. */
export type ServersView = "grid" | "list";
/** Critère de tri des serveurs. */
export type ServersSort = "name" | "favorite" | "recent";
/** Accent de marque (DS §01 : Option A émeraude / Option B cyan). */
export type Accent = "emerald" | "cyan";

interface PersistedSettings {
  terminalFontSize: number;
  terminalFontFamily: string;
  terminalLineHeight: number;
  terminalLigatures: boolean;
  terminalCursorBlink: boolean;
  serversView: ServersView;
  serversSort: ServersSort;
  accent: Accent;
}

const DEFAULTS: PersistedSettings = {
  terminalFontSize: 13,
  terminalFontFamily:
    '"JetBrains Mono", "SF Mono", ui-monospace, Menlo, Consolas, monospace',
  terminalLineHeight: 1.6,
  terminalLigatures: true,
  terminalCursorBlink: true,
  serversView: "grid",
  serversSort: "name",
  accent: "emerald",
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

/** Préférences utilisateur persistées (hors thème clair/sombre, géré par le store theme). */
export const useSettingsStore = defineStore("settings", () => {
  const initial = read();
  const terminalFontSize = ref(initial.terminalFontSize);
  const terminalFontFamily = ref(initial.terminalFontFamily);
  const terminalLineHeight = ref(initial.terminalLineHeight);
  const terminalLigatures = ref(initial.terminalLigatures);
  const terminalCursorBlink = ref(initial.terminalCursorBlink);
  const serversView = ref<ServersView>(initial.serversView);
  const serversSort = ref<ServersSort>(initial.serversSort);
  const accent = ref<Accent>(initial.accent);

  /** Horodatage de la dernière sauvegarde (toast « Préférences enregistrées »). */
  const savedAt = ref<number | null>(null);

  function applyAccent(): void {
    document.documentElement.classList.toggle("accent-cyan", accent.value === "cyan");
  }
  applyAccent();

  watch(
    [
      terminalFontSize,
      terminalFontFamily,
      terminalLineHeight,
      terminalLigatures,
      terminalCursorBlink,
      serversView,
      serversSort,
      accent,
    ],
    () => {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          terminalFontSize: terminalFontSize.value,
          terminalFontFamily: terminalFontFamily.value,
          terminalLineHeight: terminalLineHeight.value,
          terminalLigatures: terminalLigatures.value,
          terminalCursorBlink: terminalCursorBlink.value,
          serversView: serversView.value,
          serversSort: serversSort.value,
          accent: accent.value,
        } satisfies PersistedSettings),
      );
      applyAccent();
      savedAt.value = Date.now();
    },
  );

  function reset(): void {
    terminalFontSize.value = DEFAULTS.terminalFontSize;
    terminalFontFamily.value = DEFAULTS.terminalFontFamily;
    terminalLineHeight.value = DEFAULTS.terminalLineHeight;
    terminalLigatures.value = DEFAULTS.terminalLigatures;
    terminalCursorBlink.value = DEFAULTS.terminalCursorBlink;
  }

  return {
    terminalFontSize,
    terminalFontFamily,
    terminalLineHeight,
    terminalLigatures,
    terminalCursorBlink,
    serversView,
    serversSort,
    accent,
    savedAt,
    reset,
  };
});
