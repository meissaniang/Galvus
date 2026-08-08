<script setup lang="ts">
import { onBeforeUnmount, onMounted, reactive, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { useConnectionsStore, type TerminalTab } from "@/stores/connections";
import TerminalView from "@/components/TerminalView.vue";
import OsBadge from "@/components/OsBadge.vue";
import { useMyServersStore } from "@/stores/myServers";
import { useServersStore } from "@/stores/servers";
import { shortcut } from "@/utils/platform";

/**
 * Espace terminal, dans l'esprit de Termius : la sortie occupe toute la
 * surface, sans en-tête par pane. Le chrome se limite à une barre d'onglets
 * plate et à une barre d'état discrète ; les splits se redimensionnent à la
 * souris.
 *
 * Raccourcis — ⌘D (split V) · ⌘⇧D (split H) · ⌘W (fermer le pane) ·
 * ⌘1…9 (aller à l'onglet). ⌘F est géré par le pane qui a le focus.
 */
const connections = useConnectionsStore();
const { tabs, activeTabId, activeTab } = storeToRefs(connections);
const route = useRoute();
const router = useRouter();
const myServers = useMyServersStore();
const hostsStore = useServersStore();

/**
 * Système reconnu dans la bannière : on l'enregistre sur la fiche d'origine,
 * pour que la pastille porte le logo dès la prochaine ouverture de la liste.
 *
 * La clé encode l'origine (`local:12`, `config:web`), les deux n'étant pas
 * stockées au même endroit.
 */
function onOsDetected(serverKey: string | null, os: string): void {
  if (!serverKey) return;
  const [source, id] = [
    serverKey.slice(0, serverKey.indexOf(":")),
    serverKey.slice(serverKey.indexOf(":") + 1),
  ];
  if (source === "local") void myServers.setOs(Number(id), os);
  else if (source === "config") void hostsStore.setOs(id, os);
}

/** Système du serveur d'un onglet, pour la pastille de la barre d'onglets. */
function tabOs(tab: TerminalTab): string | null {
  const key = tab.panes[0]?.serverKey;
  if (!key) return null;
  const id = key.slice(key.indexOf(":") + 1);
  if (key.startsWith("local:")) {
    return myServers.servers.find((s) => s.id === Number(id))?.os ?? null;
  }
  return hostsStore.hosts.find((h) => h.alias === id)?.os ?? null;
}

/** Couleur de pastille dérivée du libellé (même règle que les tuiles). */
function tileColor(label: string): string {
  let hue = 0;
  for (const ch of label) hue = (hue * 31 + ch.charCodeAt(0)) % 360;
  return `hsl(${hue} 65% 52%)`;
}

/** Cible de la session active, affichée dans la barre d'état. */
function activeTarget(tab: TerminalTab | null): string {
  if (!tab) return "";
  const pane = tab.panes.find((p) => p.id === tab.activePaneId) ?? tab.panes[0];
  return pane?.label ?? "";
}

function paneCountText(tab: TerminalTab | null): string {
  if (!tab || tab.panes.length === 1) return "";
  return `${tab.panes.length} panes · ${tab.direction === "row" ? "vertical" : "horizontal"}`;
}

// ---------- Répartition des panes ----------
// Conservée hors du store : c'est une préférence d'affichage, pas un état de
// session. Les tailles sont exprimées en pourcentage et suivent l'onglet.
const sizes = reactive<Record<string, number[]>>({});

watch(
  () => tabs.value.map((t) => `${t.id}:${t.panes.length}`).join("|"),
  () => {
    for (const tab of tabs.value) {
      const current = sizes[tab.id];
      if (!current || current.length !== tab.panes.length) {
        sizes[tab.id] = Array(tab.panes.length).fill(100 / tab.panes.length);
      }
    }
    for (const id of Object.keys(sizes)) {
      if (!tabs.value.some((t) => t.id === id)) delete sizes[id];
    }
  },
  { immediate: true },
);

function paneSize(tab: TerminalTab, index: number): string {
  return `${sizes[tab.id]?.[index] ?? 100 / tab.panes.length}%`;
}

/** Un pane ne peut pas descendre sous ce pourcentage, pour rester lisible. */
const MIN_PANE = 12;

function startResize(event: MouseEvent, tab: TerminalTab, index: number): void {
  event.preventDefault();
  const panesEl = (event.currentTarget as HTMLElement).parentElement;
  if (!panesEl) return;

  const horizontal = tab.direction === "row";
  const total = horizontal ? panesEl.clientWidth : panesEl.clientHeight;
  if (total === 0) return;

  const origin = horizontal ? event.clientX : event.clientY;
  const initial = [...(sizes[tab.id] ?? [])];

  function onMove(move: MouseEvent): void {
    const delta = (((horizontal ? move.clientX : move.clientY) - origin) / total) * 100;
    const before = (initial[index] ?? 0) + delta;
    const after = (initial[index + 1] ?? 0) - delta;
    if (before < MIN_PANE || after < MIN_PANE) return;
    const next = [...initial];
    next[index] = before;
    next[index + 1] = after;
    sizes[tab.id] = next;
  }

  function onUp(): void {
    window.removeEventListener("mousemove", onMove);
    window.removeEventListener("mouseup", onUp);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  }

  window.addEventListener("mousemove", onMove);
  window.addEventListener("mouseup", onUp);
  document.body.style.cursor = horizontal ? "col-resize" : "row-resize";
  document.body.style.userSelect = "none";
}

// ---------- Raccourcis ----------
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
  } else if (/^[1-9]$/.test(key)) {
    const target = tabs.value[Number(key) - 1];
    if (target) {
      event.preventDefault();
      connections.setActiveTab(target.id);
    }
  }
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <section class="workspace">
    <!-- Barre d'onglets -->
    <div class="tabsbar" data-galvus-drag>
      <div class="tabsbar__tabs">
        <div
          v-for="tab in tabs"
          :key="tab.id"
          class="wtab"
          :class="{ 'wtab--on': tab.id === activeTabId }"
          :title="connections.tabTitle(tab)"
          @click="connections.setActiveTab(tab.id)"
        >
          <OsBadge
            v-if="tabOs(tab)"
            class="wtab__logo"
            :os="tabOs(tab)"
            :name="tab.panes[0]?.label ?? ''"
            :size="15"
          />
          <span
            v-else
            class="wtab__dot"
            :style="{ background: tileColor(tab.panes[0]?.label ?? '') }"
          />
          <span class="wtab__label">{{ connections.tabTitle(tab) }}</span>
          <button
            class="wtab__close"
            :title="`Fermer l'onglet (${shortcut('W')})`"
            @click.stop="connections.closeTab(tab.id)"
          >
            <svg width="10" height="10" viewBox="0 0 12 12" fill="none">
              <path
                d="M3 3l6 6M9 3l-6 6"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </div>

        <button
          class="tabsbar__new"
          title="Nouvelle session"
          @click="router.push({ name: 'servers' })"
        >
          <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
            <path
              d="M7 2.6v8.8M2.6 7h8.8"
              stroke="currentColor"
              stroke-width="1.7"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>

      <div class="tabsbar__spacer" />

      <div v-if="activeTab" class="tabsbar__splits">
        <button
          class="splitbtn"
          :class="{
            'splitbtn--on': activeTab.direction === 'row' && activeTab.panes.length > 1,
          }"
          :title="`Split vertical (${shortcut('D')})`"
          @click="connections.splitActive('row')"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect
              x="1.6"
              y="1.6"
              width="10.8"
              height="10.8"
              rx="2"
              stroke="currentColor"
              stroke-width="1.3"
            />
            <path d="M7 1.6v10.8" stroke="currentColor" stroke-width="1.3" />
          </svg>
        </button>
        <button
          class="splitbtn"
          :class="{
            'splitbtn--on':
              activeTab.direction === 'column' && activeTab.panes.length > 1,
          }"
          :title="`Split horizontal (${shortcut('⇧D')})`"
          @click="connections.splitActive('column')"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <rect
              x="1.6"
              y="1.6"
              width="10.8"
              height="10.8"
              rx="2"
              stroke="currentColor"
              stroke-width="1.3"
            />
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
        <template v-for="(pane, index) in tab.panes" :key="pane.id">
          <div
            class="pane"
            :class="{ 'pane--on': tab.panes.length > 1 && pane.id === tab.activePaneId }"
            :style="{ flexBasis: paneSize(tab, index) }"
            @mousedown="connections.setActivePane(tab.id, pane.id)"
          >
            <button
              v-if="tab.panes.length > 1"
              class="pane__close"
              :title="`Fermer ce pane (${shortcut('W')})`"
              @click.stop="connections.closePane(tab.id, pane.id)"
            >
              <svg width="9" height="9" viewBox="0 0 12 12" fill="none">
                <path
                  d="M3 3l6 6M9 3l-6 6"
                  stroke="currentColor"
                  stroke-width="1.7"
                  stroke-linecap="round"
                />
              </svg>
            </button>
            <TerminalView
              :args="pane.args"
              class="pane__view"
              @os-detected="onOsDetected(pane.serverKey, $event)"
              @finished="connections.closePane(tab.id, pane.id)"
            />
          </div>

          <div
            v-if="index < tab.panes.length - 1"
            class="gutter"
            :class="`gutter--${tab.direction}`"
            @mousedown.stop="startResize($event, tab, index)"
          />
        </template>
      </div>

      <div v-if="tabs.length === 0" class="empty">
        <svg width="34" height="34" viewBox="0 0 18 18" fill="none">
          <rect
            x="2.5"
            y="3"
            width="13"
            height="12"
            rx="2.4"
            stroke="currentColor"
            stroke-width="1.2"
          />
          <path
            d="M5.5 7.2l2 1.9-2 1.9M9.6 11.2h3"
            stroke="currentColor"
            stroke-width="1.2"
            stroke-linecap="round"
          />
        </svg>
        <p>Aucune session ouverte.</p>
        <button class="empty__cta" @click="router.push({ name: 'servers' })">
          Ouvrir un serveur
        </button>
      </div>
    </div>

    <!-- Barre d'état -->
    <div v-if="activeTab" class="statusbar">
      <span class="statusbar__dot" />
      <span class="statusbar__strong">{{ activeTarget(activeTab) }}</span>
      <span v-if="paneCountText(activeTab)">{{ paneCountText(activeTab) }}</span>
      <span class="statusbar__spacer" />
      <span>
        {{ shortcut("F") }} rechercher · {{ shortcut("D") }} split ·
        {{ shortcut("W") }} fermer
      </span>
    </div>
  </section>
</template>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--g-term-bg);
}

/* ---------- Barre d'onglets ---------- */
.tabsbar {
  height: 38px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px;
  background: var(--g-sidebar);
  border-bottom: 1px solid var(--g-border);
  flex-shrink: 0;
}

.tabsbar__tabs {
  display: flex;
  align-items: center;
  gap: 2px;
  min-width: 0;
  overflow-x: auto;
  scrollbar-width: none;
}

.tabsbar__tabs::-webkit-scrollbar {
  display: none;
}

/* Onglets plats façon Termius : pas de bordure, pas d'effet « dossier ». */
.wtab {
  display: flex;
  align-items: center;
  gap: 7px;
  height: 26px;
  padding: 0 8px 0 10px;
  border-radius: 7px;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--g-t2);
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  transition:
    background 0.12s ease,
    color 0.12s ease;
}

.wtab:hover {
  background: var(--g-s2);
  color: var(--g-t1);
}

.wtab--on,
.wtab--on:hover {
  background: var(--g-s3);
  color: var(--g-t1);
  font-weight: 600;
}

.wtab__dot {
  width: 7px;
  height: 7px;
  border-radius: 999px;
  flex-shrink: 0;
  opacity: 0.55;
}

.wtab--on .wtab__dot {
  opacity: 1;
}

/* La pastille de la barre d'onglets se passe du fond : le logo suffit. */
.wtab__logo {
  border: 0 !important;
  background: transparent !important;
}

.wtab__label {
  max-width: 150px;
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
  width: 24px;
  height: 24px;
  margin-left: 4px;
  border-radius: 7px;
  background: transparent;
  border: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--g-t3);
  cursor: pointer;
  flex-shrink: 0;
}

.tabsbar__new:hover {
  color: var(--g-t1);
  background: var(--g-s2);
}

.tabsbar__spacer {
  flex: 1;
}

.tabsbar__splits {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.splitbtn {
  width: 26px;
  height: 24px;
  border-radius: 7px;
  background: transparent;
  border: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--g-t3);
  cursor: pointer;
  transition:
    color 0.12s ease,
    background 0.12s ease;
}

.splitbtn:hover {
  color: var(--g-t1);
  background: var(--g-s2);
}

.splitbtn--on {
  color: var(--g-accent);
  background: var(--g-accent-soft);
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
}

.panes--row {
  flex-direction: row;
}

.panes--column {
  flex-direction: column;
}

.pane {
  position: relative;
  flex-grow: 0;
  flex-shrink: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  background: var(--g-term-bg);
}

/*
 * Le pane actif se signale par un liseré interne, jamais par une bordure : une
 * bordure décalerait la grille de caractères à chaque changement de focus.
 */
.pane--on::after {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
  box-shadow: inset 0 0 0 1px var(--g-accent-ring);
}

.pane__view {
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.pane__close {
  position: absolute;
  top: 6px;
  right: 8px;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  /* Posé sur le fond du terminal, pas sur une surface du thème. */
  color: #757575;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.12s ease;
}

.pane:hover .pane__close {
  opacity: 1;
}

.pane__close:hover {
  color: var(--g-danger);
  background: rgb(255 255 255 / 7%);
}

/* ---------- Séparateurs redimensionnables ---------- */
.gutter {
  position: relative;
  flex: 0 0 1px;
  background: var(--g-border);
  z-index: 3;
}

/* La zone saisissable déborde du trait, qui reste fin visuellement. */
.gutter::after {
  content: "";
  position: absolute;
  inset: -3px;
}

.gutter:hover {
  background: var(--g-accent);
}

.gutter--row {
  cursor: col-resize;
}

.gutter--column {
  cursor: row-resize;
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
  height: 26px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 14px;
  background: var(--g-sidebar);
  border-top: 1px solid var(--g-border);
  font-family: var(--g-font-mono);
  font-size: 10.5px;
  color: var(--g-t3);
}

.statusbar__dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--g-success);
  flex-shrink: 0;
}

.statusbar__strong {
  color: var(--g-t2);
}

.statusbar__spacer {
  flex: 1;
}
</style>
