<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { watch } from "vue";
import { terminalRepository } from "@/repositories/terminalRepository";
import { useSettingsStore } from "@/stores/settings";
import "@xterm/xterm/css/xterm.css";

const props = defineProps<{ args: string[] }>();
const settings = useSettingsStore();

const container = ref<HTMLDivElement | null>(null);
const sessionId = crypto.randomUUID();

let term: Terminal | null = null;
let fit: FitAddon | null = null;
let unlistenOutput: UnlistenFn | null = null;
let unlistenExit: UnlistenFn | null = null;
let resizeObserver: ResizeObserver | null = null;

/** Attend deux frames pour que le conteneur ait ses dimensions avant `fit`. */
function nextLayout(): Promise<void> {
  return new Promise((resolve) =>
    requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
  );
}

onMounted(async () => {
  if (!container.value) return;

  term = new Terminal({
    fontFamily: settings.terminalFontFamily,
    fontSize: settings.terminalFontSize,
    lineHeight: settings.terminalLineHeight,
    cursorBlink: settings.terminalCursorBlink,
    scrollback: 5000,
    theme: {
      background: "#0b1017",
      foreground: "#d6e1ec",
      cursor: "#23c48a",
      cursorAccent: "#0b1017",
      selectionBackground: "rgba(35,196,138,.28)",
    },
  });
  fit = new FitAddon();
  term.loadAddon(fit);
  term.open(container.value);

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
      term.write("\r\n\x1b[90m— session terminée —\x1b[0m\r\n");
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
</script>

<template>
  <div ref="container" class="terminal-view" />
</template>

<style scoped>
.terminal-view {
  width: 100%;
  height: 100%;
  padding: 0.5rem;
  background: #0b1017;
  border-radius: 10px;
  overflow: hidden;
}
</style>
