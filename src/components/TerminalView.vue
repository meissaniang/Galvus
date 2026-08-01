<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { terminalRepository } from "@/repositories/terminalRepository";
import "@xterm/xterm/css/xterm.css";

const props = defineProps<{ args: string[] }>();

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
    fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace',
    fontSize: 13,
    cursorBlink: true,
    scrollback: 5000,
    theme: {
      background: "#0d1117",
      foreground: "#e6edf3",
      cursor: "#e6edf3",
      selectionBackground: "#284566",
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
  background: #0d1117;
  border-radius: 10px;
  overflow: hidden;
}
</style>
