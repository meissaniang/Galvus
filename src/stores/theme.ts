import { computed, ref } from "vue";
import { defineStore } from "pinia";

/**
 * Modes de thème proposés à l'utilisateur.
 * - `system` : suit la préférence de l'OS et réagit à ses changements en direct.
 */
export type ThemeMode = "light" | "dark" | "system";

/** Thème effectivement appliqué (jamais `system`). */
export type ResolvedTheme = "light" | "dark";

const STORAGE_KEY = "galvus.theme.mode.v2";
/** Classe activant les tokens sombres de PrimeVue (voir `darkModeSelector` dans main.ts). */
const DARK_CLASS = "app-dark";

function readStoredMode(): ThemeMode {
  const value = localStorage.getItem(STORAGE_KEY);
  // Par défaut : thème sombre (esthétique type Termius). Modifiable dans Paramètres.
  return value === "light" || value === "dark" || value === "system" ? value : "dark";
}

function systemPrefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

export const useThemeStore = defineStore("theme", () => {
  const mode = ref<ThemeMode>(readStoredMode());
  /** Suivi de la préférence système, utile quand `mode === "system"`. */
  const systemDark = ref<boolean>(systemPrefersDark());

  const resolved = computed<ResolvedTheme>(() => {
    if (mode.value === "system") {
      return systemDark.value ? "dark" : "light";
    }
    return mode.value;
  });

  const isDark = computed(() => resolved.value === "dark");

  /** Applique/retire la classe sombre sur l'élément racine. */
  function apply(): void {
    document.documentElement.classList.toggle(DARK_CLASS, isDark.value);
  }

  function setMode(next: ThemeMode): void {
    mode.value = next;
    localStorage.setItem(STORAGE_KEY, next);
    apply();
  }

  /** Bascule rapide clair <-> sombre (fige le mode sur une valeur explicite). */
  function toggle(): void {
    setMode(isDark.value ? "light" : "dark");
  }

  /** À appeler une fois au démarrage : applique le thème et écoute l'OS. */
  function init(): void {
    apply();
    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", (event) => {
        systemDark.value = event.matches;
        if (mode.value === "system") {
          apply();
        }
      });
  }

  return { mode, resolved, isDark, setMode, toggle, init };
});
