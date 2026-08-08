<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { SearchAddon } from "@xterm/addon-search";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { WebglAddon } from "@xterm/addon-webgl";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { terminalRepository } from "@/repositories/terminalRepository";
import { useSettingsStore } from "@/stores/settings";
import { MATERIAL_DARK } from "@/utils/terminalTheme";
import "@xterm/xterm/css/xterm.css";

const props = defineProps<{ args: string[] }>();
const settings = useSettingsStore();

const container = ref<HTMLDivElement | null>(null);
const sessionId = crypto.randomUUID();

let term: Terminal | null = null;
let fit: FitAddon | null = null;
let search: SearchAddon | null = null;
let unlistenOutput: UnlistenFn | null = null;
let unlistenExit: UnlistenFn | null = null;
let resizeObserver: ResizeObserver | null = null;

// --- Recherche (⌘F), à la manière de Termius ---
const searchOpen = ref(false);
const searchTerm = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const matches = ref({ current: 0, total: 0 });

const SEARCH_OPTIONS = {
  decorations: {
    matchBackground: "#455A64",
    matchOverviewRuler: "#82AAFF",
    activeMatchBackground: "#0277BD",
    activeMatchColorOverviewRuler: "#FFCB6B",
  },
} as const;

/** Attend deux frames pour que le conteneur ait ses dimensions avant `fit`. */
function nextLayout(): Promise<void> {
  return new Promise((resolve) =>
    requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
  );
}

async function openSearch(): Promise<void> {
  searchOpen.value = true;
  await nextTick();
  searchInput.value?.select();
  searchInput.value?.focus();
}

function closeSearch(): void {
  searchOpen.value = false;
  search?.clearDecorations();
  matches.value = { current: 0, total: 0 };
  term?.focus();
}

function findNext(): void {
  if (searchTerm.value) search?.findNext(searchTerm.value, SEARCH_OPTIONS);
}

function findPrevious(): void {
  if (searchTerm.value) search?.findPrevious(searchTerm.value, SEARCH_OPTIONS);
}

watch(searchTerm, (value) => {
  if (!search) return;
  if (!value) {
    search.clearDecorations();
    matches.value = { current: 0, total: 0 };
    return;
  }
  search.findNext(value, SEARCH_OPTIONS);
});

/**
 * ⌘F ouvre la recherche, Échap la ferme. L'écoute est posée sur le conteneur :
 * seul le pane qui a le focus réagit, sans coordination avec le parent.
 */
function onContainerKeydown(event: KeyboardEvent): void {
  const mod = event.metaKey || event.ctrlKey;
  if (mod && event.key.toLowerCase() === "f") {
    event.preventDefault();
    event.stopPropagation();
    void openSearch();
  } else if (event.key === "Escape" && searchOpen.value) {
    event.preventDefault();
    closeSearch();
  }
}

onMounted(async () => {
  if (!container.value) return;

  term = new Terminal({
    fontFamily: settings.terminalFontFamily,
    fontSize: settings.terminalFontSize,
    lineHeight: settings.terminalLineHeight,
    cursorBlink: settings.terminalCursorBlink,
    cursorStyle: "block",
    scrollback: 5000,
    // Le gras reste gras, sans virer à la couleur vive : la palette ci-dessous
    // perdrait sa cohérence si chaque mot en gras changeait de teinte.
    drawBoldTextInBrightColors: false,
    allowProposedApi: true,
    theme: MATERIAL_DARK,
  });

  fit = new FitAddon();
  search = new SearchAddon();
  term.loadAddon(fit);
  term.loadAddon(search);
  search.onDidChangeResults(({ resultIndex, resultCount }) => {
    matches.value = { current: resultIndex + 1, total: resultCount };
  });

  // Les URL de la sortie deviennent cliquables et s'ouvrent dans le navigateur
  // du système — jamais dans la webview, qui n'est pas un navigateur.
  term.loadAddon(
    new WebLinksAddon((event, uri) => {
      event.preventDefault();
      void openUrl(uri);
    }),
  );

  term.open(container.value);

  // Rendu GPU : le rendu DOM devient flou et saccadé au-delà de quelques
  // milliers de lignes. En cas de perte de contexte, xterm revient de lui-même
  // au rendu DOM ; l'addon est simplement libéré.
  try {
    const webgl = new WebglAddon();
    webgl.onContextLoss(() => webgl.dispose());
    term.loadAddon(webgl);
  } catch {
    // Pas d'accélération disponible : le rendu DOM reste parfaitement utilisable.
  }

  await nextLayout();
  fit.fit();

  // Écoute AVANT l'ouverture de session pour ne rien manquer (bannière SSH…).
  unlistenOutput = await terminalRepository.onOutput((payload) => {
    if (payload.sessionId === sessionId && term) {
      term.write(new Uint8Array(payload.data));
    }
  });
  unlistenExit = await terminalRepository.onExit((payload) => {
    if (payload.sessionId === sessionId && term) {
      term.write("\r\n\x1b[38;2;84;84;84m── session terminée ──\x1b[0m\r\n");
    }
  });

  term.onData((data) => {
    terminalRepository.write(sessionId, data);
  });

  await terminalRepository.open(sessionId, props.args, term.cols, term.rows);
  term.focus();

  resizeObserver = new ResizeObserver(() => {
    if (!fit || !term) return;
    fit.fit();
    terminalRepository.resize(sessionId, term.cols, term.rows);
  });
  resizeObserver.observe(container.value);

  // Applique en direct les réglages du terminal depuis les Paramètres.
  watch(
    () =>
      [
        settings.terminalFontSize,
        settings.terminalFontFamily,
        settings.terminalLineHeight,
        settings.terminalCursorBlink,
      ] as const,
    ([size, family, lineHeight, blink]) => {
      if (!term || !fit) return;
      term.options.fontSize = size;
      term.options.fontFamily = family;
      term.options.lineHeight = lineHeight;
      term.options.cursorBlink = blink;
      fit.fit();
      terminalRepository.resize(sessionId, term.cols, term.rows);
    },
  );
});

onBeforeUnmount(async () => {
  resizeObserver?.disconnect();
  unlistenOutput?.();
  unlistenExit?.();
  await terminalRepository.close(sessionId);
  term?.dispose();
});

defineExpose({ openSearch, focus: () => term?.focus() });
</script>

<template>
  <div class="terminal-view" @keydown="onContainerKeydown">
    <div ref="container" class="terminal-view__screen" />

    <Transition name="search">
      <div v-if="searchOpen" class="search">
        <svg class="search__icon" width="12" height="12" viewBox="0 0 14 14" fill="none">
          <circle cx="6.2" cy="6.2" r="4.2" stroke="currentColor" stroke-width="1.4" />
          <path
            d="M9.4 9.4L12.4 12.4"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linecap="round"
          />
        </svg>
        <input
          ref="searchInput"
          v-model="searchTerm"
          class="search__input"
          type="text"
          placeholder="Rechercher dans le terminal"
          spellcheck="false"
          @keydown.enter.exact.prevent="findNext"
          @keydown.enter.shift.prevent="findPrevious"
          @keydown.esc.prevent="closeSearch"
        />
        <span v-if="searchTerm" class="search__count">
          {{ matches.total === 0 ? "aucun" : `${matches.current}/${matches.total}` }}
        </span>
        <button class="search__nav" title="Précédent (⇧⏎)" @click="findPrevious">
          <svg width="10" height="10" viewBox="0 0 12 12" fill="none">
            <path
              d="M2.5 7.5L6 4l3.5 3.5"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <button class="search__nav" title="Suivant (⏎)" @click="findNext">
          <svg width="10" height="10" viewBox="0 0 12 12" fill="none">
            <path
              d="M2.5 4.5L6 8l3.5-3.5"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </button>
        <button class="search__nav" title="Fermer (Échap)" @click="closeSearch">
          <svg width="10" height="10" viewBox="0 0 12 12" fill="none">
            <path
              d="M3 3l6 6M9 3l-6 6"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.terminal-view {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 0;
  background: var(--g-term-bg);
  overflow: hidden;
}

/* Termius laisse respirer la sortie : marge confortable, jamais collée au bord. */
.terminal-view__screen {
  width: 100%;
  height: 100%;
  padding: 10px 6px 8px 14px;
}

.terminal-view :deep(.xterm) {
  height: 100%;
}

.terminal-view :deep(.xterm-viewport) {
  background: transparent !important;
  scrollbar-width: thin;
  scrollbar-color: #424242 transparent;
}

.terminal-view :deep(.xterm-viewport)::-webkit-scrollbar {
  width: 9px;
}

.terminal-view :deep(.xterm-viewport)::-webkit-scrollbar-track {
  background: transparent;
}

.terminal-view :deep(.xterm-viewport)::-webkit-scrollbar-thumb {
  background: #424242;
  border: 2px solid var(--g-term-bg);
  border-radius: 999px;
}

.terminal-view :deep(.xterm-viewport)::-webkit-scrollbar-thumb:hover {
  background: #545454;
}

/* Les liens ne doivent se signaler qu'au survol, pour ne pas barioler la sortie. */
.terminal-view :deep(.xterm-link-layer) a:hover {
  text-decoration: underline;
  cursor: pointer;
}

/* ---------- Recherche ---------- */
.search {
  position: absolute;
  top: 10px;
  right: 14px;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 6px 5px 9px;
  border-radius: 9px;
  background: rgba(42, 42, 42, 0.96);
  border: 1px solid #3a3a3a;
  box-shadow: 0 8px 24px rgb(0 0 0 / 38%);
  backdrop-filter: blur(8px);
}

.search__icon {
  color: #9e9e9e;
  flex-shrink: 0;
}

.search__input {
  width: 190px;
  border: 0;
  background: transparent;
  color: #eeffff;
  font-family: var(--g-font-mono);
  font-size: 11.5px;
  outline: none;
}

.search__input::placeholder {
  color: #757575;
}

.search__count {
  font-family: var(--g-font-mono);
  font-size: 10.5px;
  color: #757575;
  white-space: nowrap;
  flex-shrink: 0;
}

.search__nav {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: #b0b0b0;
  cursor: pointer;
  flex-shrink: 0;
}

.search__nav:hover {
  color: #ffffff;
  background: #3a3a3a;
}

.search-enter-active,
.search-leave-active {
  transition:
    opacity 0.12s ease,
    transform 0.12s ease;
}

.search-enter-from,
.search-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
