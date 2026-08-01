<script setup lang="ts">
import { onBeforeUnmount, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { useConnectionsStore, type TerminalTab } from "@/stores/connections";
import TerminalView from "@/components/TerminalView.vue";

/**
 * Espace terminal — fidèle à « ScreenTerminal.dc.html » : barre d'onglets sur
 * fond sidebar (onglet actif = surface-0 + liseré accent 2px), splits V/H,
 * panes avec en-tête (pane actif entouré accent), barre d'état mono, raccourcis
 * ⌘D (split V) · ⌘⇧D (split H) · ⌘W (fermer le pane).
 */
const connections = useConnectionsStore();
const { tabs, activeTabId, activeTab } = storeToRefs(connections);
const route = useRoute();
const router = useRouter();

/** Couleur de mini-pastille dérivée du libellé (même règle que les tuiles). */
function tileColor(label: string): string {
  let hue = 0;
  for (const ch of label) hue = (hue * 31 + ch.charCodeAt(0)) % 360;
  return `hsl(${hue} 65% 48%)`;
}

function abbr(label: string): string {
  return label.replace(/[^a-zA-Z0-9]/g, "").slice(0, 2).toUpperCase();
}

function statusText(tab: TerminalTab | null): string {
  if (!tab) return "";
  const n = tab.panes.length;
  if (n === 1) return "1 pane";
  return `${n} panes · split ${tab.direction === "row" ? "vertical" : "horizontal"}`;
}

function closeActivePane(): void {
  const tab = activeTab.value;
  if (tab) connections.closePane(tab.id, tab.activePaneId);
}

function onKeydown(event: KeyboardEvent): void {
  if (route.name !== "terminal") return;
  const mod = event.metaKey || event.ctrlKey;
  if (!mod) return;
  const key = event.key.toLowerCase();
  if (key === "d" && !event.shiftKey) {
    event.preventDefault();
    connections.splitActive("row");
  } else if (key === "d" && event.shiftKey) {
    event.preventDefault();
    connections.splitActive("column");
  } else if (key === "w") {
    event.preventDefault();
    closeActivePane();
  }
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <section class="workspace">
    <!-- Barre d'onglets -->
    <div class="tabsbar" data-tauri-drag-region>
      <div
        v-for="tab in tabs"
        :key="tab.id"
        class="wtab"
        :class="{ 'wtab--on': tab.id === activeTabId }"
        @click="connections.setActiveTab(tab.id)"
      >
        <span v-if="tab.id === activeTabId" class="wtab__accent" />
        <span class="wtab__tile" :style="{ background: tileColor(tab.panes[0]?.label ?? '') }">
          {{ abbr(tab.panes[0]?.label ?? "") }}
        </span>
        <span class="wtab__label">{{ connections.tabTitle(tab) }}</span>
        <button class="wtab__close" title="Fermer l'onglet (⌘W)" @click.stop="connections.closeTab(tab.id)">
          <svg width="10" height="10" viewBox="0 0 12 12" fill="none">
            <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
          </svg>
        </button>
      </div>

      <button class="tabsbar__new" title="Nouvelle session" @click="router.push({ name: 'servers' })">
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
          <path d="M7 2.6v8.8M2.6 7h8.8" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" />
        </svg>
      </button>

      <div class="tabsbar__spacer" />

      <div v-if="activeTab" class="tabsbar__splits">
        <button
          class="splitbtn"
          :class="{ 'splitbtn--on': activeTab.direction === 'row' && activeTab.panes.length > 1 }"
          title="Split vertical (⌘D)"
          @click="connections.splitActive('row')"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="1.6" y="1.6" width="10.8" height="10.8" rx="2" stroke="currentColor" stroke-width="1.3" />
            <path d="M7 1.6v10.8" stroke="currentColor" stroke-width="1.3" />
          </svg>
        </button>
        <button
          class="splitbtn"
          :class="{ 'splitbtn--on': activeTab.direction === 'column' && activeTab.panes.length > 1 }"
          title="Split horizontal (⌘⇧D)"
          @click="connections.splitActive('column')"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect x="1.6" y="1.6" width="10.8" height="10.8" rx="2" stroke="currentColor" stroke-width="1.3" />
            <path d="M1.6 7h10.8" stroke="currentColor" stroke-width="1.3" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Panes -->
    <div class="body">
      <div
        v-for="tab in tabs"
        v-show="tab.id === activeTabId"
        :key="tab.id"
        class="panes"
        :class="`panes--${tab.direction}`"
      >
        <div
          v-for="(pane, index) in tab.panes"
          :key="pane.id"
          class="pane"
          :class="{ 'pane--on': tab.panes.length > 1 && pane.id === tab.activePaneId }"
          @mousedown="connections.setActivePane(tab.id, pane.id)"
        >
          <div class="pane__head">
            <span class="pane__dot" />
            <span class="pane__title">
              {{ pane.label }} — pane {{ index + 1
              }}{{ tab.panes.length > 1 && pane.id === tab.activePaneId ? " · actif" : "" }}
            </span>
            <span class="pane__spacer" />
            <button
              v-if="tab.panes.length > 1"
              class="pane__close"
              title="Fermer ce pane (⌘W)"
              @click.stop="connections.closePane(tab.id, pane.id)"
            >
              <svg width="9" height="9" viewBox="0 0 12 12" fill="none">
                <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
              </svg>
            </button>
          </div>
          <TerminalView :args="pane.args" class="pane__view" />
        </div>
      </div>

      <div v-if="tabs.length === 0" class="empty">
        <svg width="34" height="34" viewBox="0 0 18 18" fill="none">
          <rect x="2.5" y="3" width="13" height="12" rx="2.4" stroke="currentColor" stroke-width="1.2" />
          <path d="M5.5 7.2l2 1.9-2 1.9M9.6 11.2h3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        </svg>
        <p>Aucune session ouverte.</p>
        <button class="empty__cta" @click="router.push({ name: 'servers' })">
          Ouvrir un serveur
        </button>
      </div>
    </div>

    <!-- Barre d'état -->
    <div v-if="activeTab" class="statusbar">
      <span class="statusbar__strong">{{ statusText(activeTab) }}</span>
      <span>scrollback 5 000 lignes</span>
      <span class="statusbar__spacer" />
      <span>⌘D split · ⌘⇧D split H · ⌘W fermer</span>
    </div>
  </section>
</template>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--g-s0);
}

/* ---------- Barre d'onglets ---------- */
.tabsbar {
  height: 40px;
  display: flex;
  align-items: flex-end;
  padding: 0 10px;
  background: var(--g-sidebar);
  border-bottom: 1px solid var(--g-border);
  flex-shrink: 0;
  overflow-x: auto;
}

.wtab {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
  height: 32px;
  padding: 0 12px;
  border-radius: 9px 9px 0 0;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--g-t2);
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.12s ease, color 0.12s ease;
}

.wtab:hover {
  color: var(--g-t1);
}

.wtab--on {
  height: 34px;
  background: var(--g-s0);
  border: 1px solid var(--g-border);
  border-bottom: none;
  color: var(--g-t1);
  font-weight: 600;
}

.wtab__accent {
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  height: 2px;
  border-radius: 2px;
  background: var(--g-accent);
}

.wtab__tile {
  width: 14px;
  height: 14px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 7px;
  font-weight: 700;
  color: #fff;
  flex-shrink: 0;
}

.wtab__label {
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.wtab__close {
  display: flex;
  border: 0;
  background: transparent;
  color: var(--g-t3);
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
  opacity: 0;
  transition: opacity 0.12s ease;
}

.wtab:hover .wtab__close,
.wtab--on .wtab__close {
  opacity: 1;
}

.wtab__close:hover {
  color: var(--g-danger);
}

.tabsbar__new {
  width: 26px;
  height: 26px;
  margin: 0 0 4px 6px;
  border-radius: 7px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--g-t2);
  cursor: pointer;
  flex-shrink: 0;
}

.tabsbar__new:hover {
  color: var(--g-t1);
  background: var(--g-s3);
}

.tabsbar__spacer {
  flex: 1;
}

.tabsbar__splits {
  display: flex;
  gap: 5px;
  margin-bottom: 4px;
}

.splitbtn {
  width: 28px;
  height: 26px;
  border-radius: 7px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--g-t2);
  cursor: pointer;
  transition: color 0.12s ease, border-color 0.12s ease;
}

.splitbtn:hover {
  color: var(--g-t1);
}

.splitbtn--on {
  border-color: var(--g-accent-ring);
  color: var(--g-accent);
}

/* ---------- Panes ---------- */
.body {
  flex: 1;
  min-height: 0;
  position: relative;
}

.panes {
  position: absolute;
  inset: 0;
  display: flex;
  gap: 1px;
  background: var(--g-border);
}

.panes--row {
  flex-direction: row;
}

.panes--column {
  flex-direction: column;
}

.pane {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: var(--g-term-bg);
  border: 1px solid transparent;
}

.pane--on {
  border-color: var(--g-accent);
}

.pane__head {
  height: 26px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 11px;
  background: rgba(255, 255, 255, 0.04);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.pane__dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--g-success);
}

.pane__title {
  font-family: var(--g-font-mono);
  font-size: 10.5px;
  color: #9aa7b8;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pane__spacer {
  flex: 1;
}

.pane__close {
  display: flex;
  border: 0;
  background: transparent;
  color: #6b7a8d;
  cursor: pointer;
  padding: 3px;
  border-radius: 4px;
}

.pane__close:hover {
  color: var(--g-danger);
}

.pane__view {
  flex: 1;
  min-height: 0;
}

.pane__view :deep(.terminal-view),
.pane :deep(.terminal-view) {
  border-radius: 0;
}

/* ---------- État vide ---------- */
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 100%;
  color: var(--g-t3);
  font-size: 13px;
}

.empty__cta {
  border: 0;
  background: none;
  color: var(--g-accent);
  font-family: inherit;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  text-decoration: underline;
}

/* ---------- Barre d'état ---------- */
.statusbar {
  height: 28px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 0 14px;
  background: var(--g-sidebar);
  border-top: 1px solid var(--g-border);
  font-family: var(--g-font-mono);
  font-size: 10.5px;
  color: var(--g-t3);
}

.statusbar__strong {
  color: var(--g-t2);
}

.statusbar__spacer {
  flex: 1;
}
</style>
