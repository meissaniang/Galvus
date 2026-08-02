<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from "vue";
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";
import { useServersStore } from "@/stores/servers";
import { useMyServersStore } from "@/stores/myServers";
import { useConnectionsStore } from "@/stores/connections";
import { useSettingsStore } from "@/stores/settings";
import HostCard from "@/components/HostCard.vue";
import ServerCard from "@/components/ServerCard.vue";
import ServerFormDialog from "@/components/ServerFormDialog.vue";
import ConfigHostDialog from "@/components/ConfigHostDialog.vue";
import type { ConfigHostInput, Host, Server, ServerInput } from "@/types/ssh";
import { shortcut } from "@/utils/platform";

/**
 * Écran Serveurs — implémentation fidèle de « ScreenServers.dc.html » :
 * topbar (recherche ⌘K, tri, bascule grille/liste, Nouveau serveur),
 * groupes repliables avec compteurs, tuiles avec actions au survol,
 * carte « Ajouter à… », section ~/.ssh/config en lecture seule.
 */
const hostsStore = useServersStore();
const myServers = useMyServersStore();
const connections = useConnectionsStore();
const settings = useSettingsStore();
const router = useRouter();

const { hosts, loading: hostsLoading, error: hostsError } = storeToRefs(hostsStore);
const { servers, error: serversError } = storeToRefs(myServers);
const { serversView, serversSort } = storeToRefs(settings);

const search = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const collapsed = reactive(new Set<string>());
const configCollapsed = ref(false);

function match(values: (string | null)[]): boolean {
  const q = search.value.trim().toLowerCase();
  if (!q) return true;
  return values
    .filter((v): v is string => Boolean(v))
    .some((v) => v.toLowerCase().includes(q));
}

const sortedServers = computed<Server[]>(() => {
  const list = servers.value.filter((s) =>
    match([s.name, s.hostname, s.username, ...s.tags]),
  );
  const sorted = [...list];
  if (serversSort.value === "favorite") {
    sorted.sort(
      (a, b) => Number(b.favorite) - Number(a.favorite) || a.name.localeCompare(b.name),
    );
  } else if (serversSort.value === "recent") {
    sorted.sort((a, b) => b.id - a.id);
  } else {
    sorted.sort((a, b) => a.name.localeCompare(b.name));
  }
  return sorted;
});

/** Groupes ordonnés : nommés d'abord (alpha), non groupés en dernier. */
const groupedServers = computed<[string, Server[]][]>(() => {
  const groups = new Map<string, Server[]>();
  for (const server of sortedServers.value) {
    const key = server.group ?? "";
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(server);
  }
  return [...groups.entries()].sort(([a], [b]) => {
    if (a === "") return 1;
    if (b === "") return -1;
    return a.localeCompare(b);
  });
});

const groupNames = computed(() =>
  [
    ...new Set(servers.value.map((s) => s.group).filter((g): g is string => Boolean(g))),
  ].sort(),
);

/** Hôtes du config : filtrés puis triés selon le même critère. */
const filteredHosts = computed<Host[]>(() => {
  const list = hosts.value.filter((h) => match([h.alias, h.hostname, h.user]));
  const sorted = [...list];
  if (serversSort.value === "recent") {
    sorted.reverse();
  } else {
    sorted.sort((a, b) => a.alias.localeCompare(b.alias));
  }
  return sorted;
});

/** Libellés des sessions ouvertes → point vert « connecté » sur les cartes. */
const connectedLabels = computed(
  () => new Set(connections.tabs.flatMap((t) => t.panes.map((p) => p.label))),
);

function toggleGroup(name: string): void {
  if (collapsed.has(name)) collapsed.delete(name);
  else collapsed.add(name);
}

// --- Connexions ---
function connectHost(host: Host): void {
  connections.open(host.alias, [host.alias]);
  router.push({ name: "terminal" });
}
function connectServer(server: Server): void {
  const args: string[] = [];
  if (server.port !== 22) args.push("-p", String(server.port));
  if (server.identityFile) args.push("-i", server.identityFile);
  args.push(server.username ? `${server.username}@${server.hostname}` : server.hostname);
  connections.open(server.name, args);
  router.push({ name: "terminal" });
}

// --- CRUD ---
const dialogOpen = ref(false);
const editingServer = ref<Server | null>(null);
const dialogGroup = ref<string | null>(null);

function openNew(group: string | null = null): void {
  editingServer.value = null;
  dialogGroup.value = group;
  dialogOpen.value = true;
}
function openEdit(server: Server): void {
  editingServer.value = server;
  dialogGroup.value = null;
  dialogOpen.value = true;
}
async function onSave(input: ServerInput): Promise<void> {
  if (editingServer.value) {
    await myServers.update(editingServer.value.id, input);
  } else {
    await myServers.create(input);
  }
  dialogOpen.value = false;
}
async function onRemove(server: Server): Promise<void> {
  if (window.confirm(`Supprimer « ${server.name} » ?`)) {
    await myServers.remove(server.id);
  }
}
function toInput(s: Server): ServerInput {
  return {
    name: s.name,
    hostname: s.hostname,
    port: s.port,
    username: s.username,
    identityFile: s.identityFile,
    color: s.color,
    favorite: s.favorite,
    tags: s.tags,
    group: s.group,
  };
}
async function toggleFavorite(server: Server): Promise<void> {
  await myServers.update(server.id, { ...toInput(server), favorite: !server.favorite });
}

// --- Édition des hôtes ~/.ssh/config ---
const hostDialogOpen = ref(false);
const editingHost = ref<Host | null>(null);

function openEditHost(host: Host): void {
  editingHost.value = host;
  hostDialogOpen.value = true;
}

async function onSaveHost(input: ConfigHostInput): Promise<void> {
  if (!editingHost.value) return;
  try {
    await hostsStore.update(editingHost.value.alias, input);
    hostDialogOpen.value = false;
  } catch {
    /* erreur affichée via hostsError */
  }
}

async function onRemoveHost(host: Host): Promise<void> {
  if (window.confirm(`Supprimer « ${host.alias} » de ~/.ssh/config ?`)) {
    await hostsStore.remove(host.alias);
  }
}

/** ⌘K / Ctrl+K : focus sur la recherche. */
function onKeydown(event: KeyboardEvent): void {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    searchInput.value?.focus();
    searchInput.value?.select();
  }
}

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
  if (hosts.value.length === 0) hostsStore.load();
  if (servers.value.length === 0) myServers.load();
});
onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));

function addressOf(server: Server): string {
  const target = server.username
    ? `${server.username}@${server.hostname}`
    : server.hostname;
  return `${target}:${server.port}`;
}
</script>

<template>
  <section class="screen">
    <!-- Topbar 56px -->
    <div class="topbar" data-galvus-drag>
      <div class="search">
        <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
          <circle cx="7" cy="7" r="4.4" stroke="currentColor" stroke-width="1.5" />
          <path
            d="M10.4 10.4L14 14"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          />
        </svg>
        <input
          ref="searchInput"
          v-model="search"
          type="text"
          placeholder="Rechercher un serveur, un tag, un host…"
          spellcheck="false"
        />
        <span class="search__kbd">{{ shortcut("K") }}</span>
      </div>

      <label class="sort">
        <span class="sort__label">Trier</span>
        <select v-model="serversSort">
          <option value="name">Nom</option>
          <option value="favorite">Favoris</option>
          <option value="recent">Récents</option>
        </select>
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path
            d="M3 4.6l3 3 3-3"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
          />
        </svg>
      </label>

      <div class="viewtoggle">
        <button
          class="viewtoggle__btn"
          :class="{ 'viewtoggle__btn--on': serversView === 'grid' }"
          title="Vue grille"
          @click="serversView = 'grid'"
        >
          <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
            <rect x="1.5" y="1.5" width="4.4" height="4.4" rx="1.2" fill="currentColor" />
            <rect x="8.1" y="1.5" width="4.4" height="4.4" rx="1.2" fill="currentColor" />
            <rect x="1.5" y="8.1" width="4.4" height="4.4" rx="1.2" fill="currentColor" />
            <rect x="8.1" y="8.1" width="4.4" height="4.4" rx="1.2" fill="currentColor" />
          </svg>
        </button>
        <button
          class="viewtoggle__btn"
          :class="{ 'viewtoggle__btn--on': serversView === 'list' }"
          title="Vue liste"
          @click="serversView = 'list'"
        >
          <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
            <rect x="1.5" y="2.2" width="11" height="2.2" rx="1.1" fill="currentColor" />
            <rect x="1.5" y="6" width="11" height="2.2" rx="1.1" fill="currentColor" />
            <rect x="1.5" y="9.8" width="11" height="2.2" rx="1.1" fill="currentColor" />
          </svg>
        </button>
      </div>

      <button class="newbtn" @click="openNew()">
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
          <path
            d="M7 2.4v9.2M2.4 7h9.2"
            stroke="currentColor"
            stroke-width="1.9"
            stroke-linecap="round"
          />
        </svg>
        Nouveau serveur
      </button>
    </div>

    <!-- Contenu défilant -->
    <div class="content">
      <p v-if="serversError" class="state state--error">{{ serversError }}</p>

      <!-- Groupes de serveurs -->
      <section
        v-for="[group, list] in groupedServers"
        :key="group || '__none'"
        class="group"
      >
        <header class="group__head">
          <button
            class="group__chevron"
            :class="{ 'group__chevron--closed': collapsed.has(group) }"
            @click="toggleGroup(group)"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path
                d="M3 4.6l3 3 3-3"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
              />
            </svg>
          </button>
          <span class="group__name">{{ group || "Mes serveurs" }}</span>
          <span class="group__count">{{ list.length }}</span>
          <span class="group__line" />
        </header>

        <template v-if="!collapsed.has(group)">
          <!-- Vue grille -->
          <div v-if="serversView === 'grid'" class="grid">
            <ServerCard
              v-for="server in list"
              :key="server.id"
              :server="server"
              :connected="connectedLabels.has(server.name)"
              @connect="connectServer"
              @edit="openEdit"
              @remove="onRemove"
              @toggle-favorite="toggleFavorite"
            />
            <button class="addcard" @click="openNew(group || null)">
              <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
                <path
                  d="M7 2.4v9.2M2.4 7h9.2"
                  stroke="currentColor"
                  stroke-width="1.7"
                  stroke-linecap="round"
                />
              </svg>
              {{ group ? `Ajouter à ${group}` : "Ajouter un serveur" }}
            </button>
          </div>

          <!-- Vue liste -->
          <div v-else class="rows">
            <div
              v-for="server in list"
              :key="server.id"
              class="row"
              @dblclick="connectServer(server)"
            >
              <span
                class="row__ava"
                :style="{ background: server.color ?? 'var(--g-s3)' }"
                >{{ server.name.slice(0, 2).toUpperCase() }}</span
              >
              <span class="row__name">
                {{ server.name }}
                <span v-if="connectedLabels.has(server.name)" class="row__dot" />
              </span>
              <span class="row__addr">{{ addressOf(server) }}</span>
              <span class="row__tags">
                <span v-for="tag in server.tags" :key="tag" class="row__tag">{{
                  tag
                }}</span>
              </span>
              <span class="row__actions">
                <button class="row__connect" @click.stop="connectServer(server)">
                  Connecter
                </button>
                <button class="row__icon" title="Éditer" @click.stop="openEdit(server)">
                  <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
                    <path
                      d="M9.4 2.4l2.2 2.2-6.4 6.4-2.8.6.6-2.8z"
                      stroke="currentColor"
                      stroke-width="1.4"
                      stroke-linejoin="round"
                    />
                  </svg>
                </button>
                <button
                  class="row__icon row__icon--danger"
                  title="Supprimer"
                  @click.stop="onRemove(server)"
                >
                  <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
                    <path
                      d="M2.6 4.4h8.8M5.4 4.4V3.2h3.2v1.2M4 4.4l.6 6.6h4.8L10 4.4"
                      stroke="currentColor"
                      stroke-width="1.3"
                      stroke-linecap="round"
                    />
                  </svg>
                </button>
              </span>
            </div>
          </div>
        </template>
      </section>

      <!-- Section ~/.ssh/config -->
      <section v-if="filteredHosts.length > 0 || hostsError" class="group">
        <header class="group__head">
          <button
            class="group__chevron"
            :class="{ 'group__chevron--closed': configCollapsed }"
            @click="configCollapsed = !configCollapsed"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path
                d="M3 4.6l3 3 3-3"
                stroke="currentColor"
                stroke-width="1.6"
                stroke-linecap="round"
              />
            </svg>
          </button>
          <span class="group__mono">~/.ssh/config</span>
          <span class="group__lock">
            <svg width="9" height="9" viewBox="0 0 10 10" fill="none">
              <rect
                x="1.6"
                y="4.4"
                width="6.8"
                height="4.4"
                rx="1.2"
                stroke="currentColor"
                stroke-width="1.2"
              />
              <path
                d="M3.4 4.4V3.2a1.6 1.6 0 013.2 0v1.2"
                stroke="currentColor"
                stroke-width="1.2"
              />
            </svg>
            lecture seule
          </span>
          <span class="group__line" />
          <span class="group__hint">{{ filteredHosts.length }} hôtes importés</span>
        </header>

        <p v-if="hostsError" class="state state--error">{{ hostsError }}</p>
        <div v-else-if="!configCollapsed" class="grid grid--config">
          <HostCard
            v-for="host in filteredHosts"
            :key="host.alias"
            :host="host"
            :connected="connectedLabels.has(host.alias)"
            @click="connectHost(host)"
            @edit="openEditHost"
            @remove="onRemoveHost"
          />
        </div>
      </section>

      <!-- État vide -->
      <div
        v-if="sortedServers.length === 0 && filteredHosts.length === 0 && !hostsLoading"
        class="empty"
      >
        <svg width="34" height="34" viewBox="0 0 18 18" fill="none">
          <rect
            x="2.5"
            y="2.8"
            width="13"
            height="5"
            rx="1.8"
            stroke="currentColor"
            stroke-width="1.2"
          />
          <rect
            x="2.5"
            y="10.2"
            width="13"
            height="5"
            rx="1.8"
            stroke="currentColor"
            stroke-width="1.2"
          />
        </svg>
        <p v-if="search">Aucun résultat pour « {{ search }} ».</p>
        <p v-else>Aucun serveur. Clique sur « Nouveau serveur » pour commencer.</p>
      </div>
    </div>

    <ServerFormDialog
      :open="dialogOpen"
      :server="editingServer"
      :groups="groupNames"
      :default-group="dialogGroup"
      @save="onSave"
      @close="dialogOpen = false"
    />

    <ConfigHostDialog
      :open="hostDialogOpen"
      :host="editingHost"
      @save="onSaveHost"
      @close="hostDialogOpen = false"
    />
  </section>
</template>

<style scoped>
.screen {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--g-s0);
}

/* ---------- Topbar ---------- */
.topbar {
  height: 56px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 20px;
  border-bottom: 1px solid var(--g-border);
  flex-shrink: 0;
}

.search {
  flex: 1;
  max-width: 520px;
  display: flex;
  align-items: center;
  gap: 9px;
  height: 34px;
  padding: 0 11px;
  border-radius: 10px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  color: var(--g-t3);
  transition:
    border-color 0.12s ease-out,
    box-shadow 0.12s ease-out;
}

.search:focus-within {
  border-color: var(--g-accent);
  box-shadow: 0 0 0 3px var(--g-accent-ring);
}

.search input {
  flex: 1;
  border: 0;
  background: transparent;
  font-family: inherit;
  font-size: 13px;
  color: var(--g-t1);
  outline: none;
  min-width: 0;
}

.search input::placeholder {
  color: var(--g-t3);
}

.search__kbd {
  font-family: var(--g-font-mono);
  font-size: 10.5px;
  color: var(--g-t3);
  background: var(--g-s0);
  border: 1px solid var(--g-border);
  padding: 2px 6px;
  border-radius: 6px;
}

.sort {
  position: relative;
  display: flex;
  align-items: center;
  gap: 7px;
  height: 34px;
  padding: 0 11px;
  border-radius: 10px;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  font-size: 12.5px;
  color: var(--g-t2);
  cursor: pointer;
}

.sort__label {
  color: var(--g-t3);
}

.sort select {
  appearance: none;
  border: 0;
  background: transparent;
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--g-t1);
  outline: none;
  cursor: pointer;
  padding-right: 2px;
}

.viewtoggle {
  display: flex;
  padding: 3px;
  gap: 2px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  border-radius: 9px;
}

.viewtoggle__btn {
  width: 28px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--g-t3);
  cursor: pointer;
  transition:
    background 0.12s ease,
    color 0.12s ease;
}

.viewtoggle__btn--on {
  background: var(--g-s0);
  color: var(--g-t1);
  box-shadow: var(--g-sh1);
}

.newbtn {
  display: flex;
  align-items: center;
  gap: 7px;
  height: 34px;
  padding: 0 14px;
  border: 0;
  border-radius: 10px;
  background: var(--g-accent);
  color: var(--g-accent-fg);
  font-family: inherit;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: var(--g-sh1);
  white-space: nowrap;
  transition: background 0.12s linear;
}

.newbtn:hover {
  background: var(--g-accent-h);
}

.newbtn:active {
  transform: scale(0.985);
}

/* ---------- Contenu ---------- */
.content {
  flex: 1;
  overflow-y: auto;
  padding: 18px 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.group__head {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-bottom: 11px;
}

.group__chevron {
  display: flex;
  border: 0;
  background: transparent;
  color: var(--g-t3);
  cursor: pointer;
  padding: 2px;
  transition: transform 0.14s cubic-bezier(0.2, 0.8, 0.3, 1);
}

.group__chevron--closed {
  transform: rotate(-90deg);
}

.group__name {
  font-size: 12.5px;
  font-weight: 600;
  letter-spacing: 0.03em;
  text-transform: uppercase;
  color: var(--g-t2);
}

.group__mono {
  font-family: var(--g-font-mono);
  font-size: 12px;
  font-weight: 500;
  color: var(--g-t2);
}

.group__count {
  font-family: var(--g-font-mono);
  font-size: 10.5px;
  color: var(--g-t3);
  background: var(--g-s2);
  padding: 1px 6px;
  border-radius: 6px;
}

.group__lock {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 10.5px;
  font-weight: 600;
  color: var(--g-t3);
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  padding: 2px 7px;
  border-radius: 999px;
}

.group__line {
  flex: 1;
  height: 1px;
  background: var(--g-border);
}

.group__hint {
  font-size: 11.5px;
  color: var(--g-t3);
}

/* Grille responsive : 3 col ≥1140, 2 col ≥960, 1 col en dessous. */
.grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

@media (max-width: 1139px) {
  .grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 959px) {
  .grid {
    grid-template-columns: 1fr;
  }
}

.grid--config {
  opacity: 0.72;
}

.addcard {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 66px;
  border: 1px dashed var(--g-border-2);
  border-radius: 12px;
  background: transparent;
  color: var(--g-t3);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 500;
  cursor: pointer;
  transition:
    color 0.12s ease,
    border-color 0.12s ease;
}

.addcard:hover {
  color: var(--g-accent);
  border-color: var(--g-accent-ring);
}

/* ---------- Vue liste ---------- */
.rows {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.row {
  display: flex;
  align-items: center;
  gap: 11px;
  height: 44px;
  padding: 0 12px;
  border: 1px solid var(--g-border);
  border-radius: 10px;
  background: var(--g-s1);
  transition:
    background 0.12s ease,
    border-color 0.12s ease;
}

.row:hover {
  background: var(--g-s2);
  border-color: var(--g-accent-ring);
}

.row__ava {
  width: 26px;
  height: 26px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 700;
  color: #fff;
  flex-shrink: 0;
}

.row__name {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 180px;
  font-size: 13px;
  font-weight: 600;
  color: var(--g-t1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row__dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--g-success);
}

.row__addr {
  flex: 1;
  font-family: var(--g-font-mono);
  font-size: 11.5px;
  color: var(--g-t2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row__tags {
  display: flex;
  gap: 5px;
}

.row__tag {
  font-size: 10.5px;
  color: var(--g-t2);
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  padding: 1px 7px;
  border-radius: 999px;
}

.row__actions {
  display: flex;
  gap: 5px;
  opacity: 0;
  transition: opacity 0.12s ease-out;
}

.row:hover .row__actions {
  opacity: 1;
}

.row__connect {
  height: 26px;
  padding: 0 11px;
  border: 0;
  border-radius: 8px;
  background: var(--g-accent);
  color: var(--g-accent-fg);
  font-family: inherit;
  font-size: 11.5px;
  font-weight: 600;
  cursor: pointer;
}

.row__icon {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  color: var(--g-t2);
  cursor: pointer;
}

.row__icon--danger:hover {
  background: var(--g-danger-soft);
  border-color: var(--g-danger);
  color: var(--g-danger);
}

/* ---------- États ---------- */
.state--error {
  color: var(--g-danger);
  font-size: 12.5px;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  margin-top: 24px;
  padding: 48px;
  border: 1px dashed var(--g-border-2);
  border-radius: 12px;
  color: var(--g-t3);
  font-size: 13px;
}
</style>
