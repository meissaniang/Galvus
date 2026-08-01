import { ref, watch } from "vue";
import { defineStore } from "pinia";

const STORAGE_KEY = "galvus.settings.v1";

interface PersistedSettings {
  terminalFontSize: number;
  terminalFontFamily: string;
}

const DEFAULTS: PersistedSettings = {
  terminalFontSize: 13,
  terminalFontFamily:
    'ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace',
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

  watch([terminalFontSize, terminalFontFamily], () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        terminalFontSize: terminalFontSize.value,
        terminalFontFamily: terminalFontFamily.value,
      }),
    );
  });

  function reset(): void {
    terminalFontSize.value = DEFAULTS.terminalFontSize;
    terminalFontFamily.value = DEFAULTS.terminalFontFamily;
  }

  return { terminalFontSize, terminalFontFamily, reset };
});
