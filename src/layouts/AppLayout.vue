<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { storeToRefs } from "pinia";
import { useThemeStore, type ThemeMode } from "@/stores/theme";
import { useServersStore } from "@/stores/servers";
import { useMyServersStore } from "@/stores/myServers";
import { useKeysStore } from "@/stores/keys";
import { useTunnelsStore } from "@/stores/tunnels";
import { useConnectionsStore } from "@/stores/connections";

/**
 * Shell applicatif — implémentation fidèle de « GalvusSidebar.dc.html » :
 * feux macOS dans la sidebar (titleBarStyle Overlay), marque + version,
 * navigation avec compteurs, sessions actives, sélecteur de thème 3 segments.
 */
const route = useRoute();
const router = useRouter();
const theme = useThemeStore();

const hostsStore = useServersStore();
const myServers = useMyServersStore();
const keysStore = useKeysStore();
const tunnels = useTunnelsStore();
const connections = useConnectionsStore();

const { hosts } = storeToRefs(hostsStore);
const { servers } = storeToRefs(myServers);
const { keys } = storeToRefs(keysStore);
const { tabs } = storeToRefs(connections);

const serverCount = computed(() => servers.value.length + hosts.value.length);
const keyCount = computed(() => keys.value.length);
const runningTunnels = computed(() => tunnels.runningIds.length);

const isActive = (name: string) => route.name === name;

function openSession(tabId: string): void {
  connections.setActiveTab(tabId);
  router.push({ name: "terminal" });
}

function setTheme(mode: ThemeMode): void {
  theme.setMode(mode);
}

/** ⌘T / Ctrl+T : ouvrir l'espace terminal. */
function onKeydown(event: KeyboardEvent): void {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "t") {
    event.preventDefault();
    router.push({ name: "terminal" });
  }
}

/**
 * Déplacement de la fenêtre (barre de titre Overlay).
 *
 * On n'utilise PAS `data-tauri-drag-region` sur les barres contenant des
 * contrôles : l'attribut natif capte le mousedown et empêche les champs, selects
 * et boutons de réagir. À la place, les zones portent `data-galvus-drag` et ce
 * handler démarre le drag seulement si le clic ne vise pas un élément interactif.
 */
function onDragRegion(event: MouseEvent): void {
  if (event.button !== 0) return;
  const target = event.target as HTMLElement | null;
  if (!target) return;
  if (
    target.closest(
      "button, input, select, textarea, a, label, [role='switch'], [contenteditable]",
    )
  ) {
    return;
  }
  if (!target.closest("[data-galvus-drag]")) return;
  const win = getCurrentWindow();
  if (event.detail === 2) {
    void win.toggleMaximize();
  } else {
    void win.startDragging();
  }
}

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("mousedown", onDragRegion);
  // Charge les compteurs de la sidebar dès l'ouverture de l'app.
  if (hosts.value.length === 0) hostsStore.load();
  if (servers.value.length === 0) myServers.load();
  if (keys.value.length === 0) keysStore.load();
  if (tunnels.tunnels.length === 0) tunnels.load();
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("mousedown", onDragRegion);
});
</script>

<template>
  <div class="app-shell">
    <aside class="sb">
      <!-- Zone des feux macOS (fenêtre en titleBarStyle Overlay) + drag. -->
      <div class="sb__traffic" data-tauri-drag-region></div>

      <div class="sb__brand" data-galvus-drag>
        <div class="sb__logo">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <rect x="2.5" y="2.5" width="11" height="11" rx="3" stroke="var(--g-accent-fg)" stroke-width="1.6" />
            <circle cx="8" cy="8" r="2.1" fill="var(--g-accent-fg)" />
          </svg>
        </div>
        <div class="sb__brand-text">
          <span class="sb__name">Galvus</span>
          <span class="sb__version">v0.1.0 · offline</span>
        </div>
      </div>

      <div class="sb__label">Navigation</div>

      <nav class="sb__nav">
        <router-link :to="{ name: 'servers' }" class="sb__item" :class="{ 'sb__item--on': isActive('servers') }">
          <span class="sb__bar" />
          <svg width="17" height="17" viewBox="0 0 18 18" fill="none">
            <rect x="2.5" y="2.8" width="13" height="5" rx="1.8" stroke="currentColor" stroke-width="1.5" />
            <rect x="2.5" y="10.2" width="13" height="5" rx="1.8" stroke="currentColor" stroke-width="1.5" />
            <circle cx="5.4" cy="5.3" r="1" fill="currentColor" />
            <circle cx="5.4" cy="12.7" r="1" fill="currentColor" />
          </svg>
          <span class="sb__text">Serveurs</span>
          <span class="sb__count">{{ serverCount }}</span>
        </router-link>

        <router-link :to="{ name: 'keys' }" class="sb__item" :class="{ 'sb__item--on': isActive('keys') }">
          <span class="sb__bar" />
          <svg width="17" height="17" viewBox="0 0 18 18" fill="none">
            <circle cx="6.2" cy="6.2" r="3.4" stroke="currentColor" stroke-width="1.5" />
            <path d="M8.7 8.7L14.5 14.5M12 13l1.6-1.6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
          <span class="sb__text">Clés SSH</span>
          <span class="sb__count">{{ keyCount }}</span>
        </router-link>

        <router-link :to="{ name: 'tunnels' }" class="sb__item" :class="{ 'sb__item--on': isActive('tunnels') }">
          <span class="sb__bar" />
          <svg width="17" height="17" viewBox="0 0 18 18" fill="none">
            <circle cx="4.4" cy="9" r="2.3" stroke="currentColor" stroke-width="1.5" />
            <circle cx="13.6" cy="9" r="2.3" stroke="currentColor" stroke-width="1.5" />
            <path d="M6.7 9h4.6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
          <span class="sb__text">Tunnels</span>
          <span v-if="runningTunnels > 0" class="sb__meta"><span class="sb__dot" />{{ runningTunnels }}</span>
        </router-link>

        <router-link :to="{ name: 'terminal' }" class="sb__item" :class="{ 'sb__item--on': isActive('terminal') }">
          <span class="sb__bar" />
          <svg width="17" height="17" viewBox="0 0 18 18" fill="none">
            <rect x="2.5" y="3" width="13" height="12" rx="2.4" stroke="currentColor" stroke-width="1.5" />
            <path d="M5.5 7.2l2 1.9-2 1.9M9.6 11.2h3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
          <span class="sb__text">Terminal</span>
          <span class="sb__kbd">⌘T</span>
        </router-link>

        <router-link :to="{ name: 'settings' }" class="sb__item" :class="{ 'sb__item--on': isActive('settings') }">
          <span class="sb__bar" />
          <svg width="17" height="17" viewBox="0 0 18 18" fill="none">
            <circle cx="9" cy="9" r="2.4" stroke="currentColor" stroke-width="1.5" />
            <circle cx="9" cy="9" r="6" stroke="currentColor" stroke-width="1.5" stroke-dasharray="2.6 2.6" />
          </svg>
          <span class="sb__text">Paramètres</span>
        </router-link>
      </nav>

      <template v-if="tabs.length > 0">
        <div class="sb__sep" />
        <div class="sb__label">Sessions actives</div>
        <div class="sb__sessions">
          <button
            v-for="tab in tabs.slice(0, 5)"
            :key="tab.id"
            class="sb__session"
            @click="openSession(tab.id)"
          >
            <span class="sb__dot sb__dot--halo" />
            <span class="sb__session-name">{{ connections.tabTitle(tab) }}</span>
          </button>
        </div>
      </template>

      <div class="sb__spacer" />

      <div class="sb__footer">
        <div class="sb__theme">
          <button
            class="sb__seg"
            :class="{ 'sb__seg--on': theme.mode === 'system' }"
            @click="setTheme('system')"
          >
            Auto
          </button>
          <button
            class="sb__seg"
            :class="{ 'sb__seg--on': theme.mode === 'light' }"
            @click="setTheme('light')"
          >
            <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
              <circle cx="7" cy="7" r="3" stroke="currentColor" stroke-width="1.5" />
              <path d="M7 1v1.4M7 11.6V13M1 7h1.4M11.6 7H13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
            Clair
          </button>
          <button
            class="sb__seg"
            :class="{ 'sb__seg--on': theme.mode === 'dark' }"
            @click="setTheme('dark')"
          >
            <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
              <path d="M11.5 8.6A5 5 0 015.4 2.5a5 5 0 106.1 6.1z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
            </svg>
            Sombre
          </button>
        </div>
      </div>
    </aside>

    <main class="app-content">
      <!-- L'espace terminal est gardé en vie pour préserver les sessions PTY. -->
      <router-view v-slot="{ Component }">
        <keep-alive :include="['TerminalWorkspace']">
          <component :is="Component" />
        </keep-alive>
      </router-view>
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  color: var(--g-t1);
  background: var(--g-s0);
}

/* ---------- Sidebar 236px ---------- */
.sb {
  display: flex;
  flex-direction: column;
  width: 236px;
  min-width: 236px;
  flex-shrink: 0;
  background: var(--g-sidebar);
  border-right: 1px solid var(--g-border);
}

.sb__traffic {
  height: 40px;
  flex-shrink: 0;
}

.sb__brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 2px 14px 14px;
}

.sb__logo {
  width: 28px;
  height: 28px;
  border-radius: 9px;
  background: var(--g-accent);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: var(--g-sh1);
}

.sb__brand-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.sb__name {
  font-size: 14.5px;
  font-weight: 600;
  color: var(--g-t1);
  letter-spacing: -0.01em;
}

.sb__version {
  font-family: var(--g-font-mono);
  font-size: 10px;
  color: var(--g-t3);
}

.sb__label {
  padding: 0 10px 6px;
  margin-left: 4px;
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--g-t3);
}

.sb__nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0 10px;
}

.sb__item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  height: 36px;
  padding: 0 10px;
  border-radius: 10px;
  font-size: 13.5px;
  font-weight: 500;
  color: var(--g-t2);
  text-decoration: none;
  transition: background 0.12s ease, color 0.12s ease;
}

.sb__item:hover {
  background: var(--g-s2);
  color: var(--g-t1);
}

.sb__item--on {
  background: var(--g-s2);
  color: var(--g-t1);
  box-shadow: var(--g-sh1);
}

.sb__bar {
  position: absolute;
  left: -10px;
  top: 9px;
  width: 3px;
  height: 18px;
  border-radius: 0 3px 3px 0;
  background: var(--g-accent);
  opacity: 0;
  transform: scaleY(0.4);
  transition: opacity 0.16s cubic-bezier(0.2, 0.8, 0.3, 1),
    transform 0.16s cubic-bezier(0.2, 0.8, 0.3, 1);
}

.sb__item--on .sb__bar {
  opacity: 1;
  transform: scaleY(1);
}

.sb__item svg {
  flex-shrink: 0;
}

.sb__text {
  flex: 1;
}

.sb__count {
  font-family: var(--g-font-mono);
  font-size: 10.5px;
  color: var(--g-t3);
  background: var(--g-s2);
  padding: 1px 6px;
  border-radius: 6px;
}

.sb__item--on .sb__count {
  background: var(--g-s3);
}

.sb__meta {
  display: flex;
  align-items: center;
  gap: 5px;
  font-family: var(--g-font-mono);
  font-size: 10.5px;
  color: var(--g-t2);
}

.sb__kbd {
  font-family: var(--g-font-mono);
  font-size: 10.5px;
  color: var(--g-t3);
}

.sb__dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--g-success);
}

.sb__dot--halo {
  box-shadow: 0 0 0 3px var(--g-accent-soft);
}

.sb__sep {
  margin: 18px 14px 6px;
  height: 1px;
  background: var(--g-border);
}

.sb__sessions {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0 10px;
}

.sb__session {
  display: flex;
  align-items: center;
  gap: 9px;
  height: 32px;
  padding: 0 10px;
  border: 0;
  border-radius: 9px;
  background: transparent;
  font-size: 12.5px;
  color: var(--g-t2);
  cursor: pointer;
  transition: background 0.12s ease;
}

.sb__session:hover {
  background: var(--g-s2);
}

.sb__session-name {
  flex: 1;
  text-align: left;
  font-family: var(--g-font-mono);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sb__spacer {
  flex: 1;
}

.sb__footer {
  padding: 10px 14px 14px;
}

.sb__theme {
  display: flex;
  padding: 3px;
  gap: 3px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  border-radius: 10px;
}

.sb__seg {
  flex: 1;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  font-family: inherit;
  font-size: 11.5px;
  font-weight: 500;
  color: var(--g-t2);
  cursor: pointer;
  transition: background 0.12s ease, color 0.12s ease;
}

.sb__seg--on {
  background: var(--g-s0);
  color: var(--g-t1);
  box-shadow: var(--g-sh1);
}

/* ---------- Contenu ---------- */
.app-content {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
