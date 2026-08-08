<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from "vue";
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";
import { useServersStore } from "@/stores/servers";
import { useMyServersStore } from "@/stores/myServers";
import { useConnectionsStore } from "@/stores/connections";
import { useSettingsStore } from "@/stores/settings";
import ServerCard from "@/components/ServerCard.vue";
import OsBadge from "@/components/OsBadge.vue";
import ServerFormDialog, {
  type ServerFormResult,
} from "@/components/ServerFormDialog.vue";
import type { Host, Server, ServerItem, ServerSource } from "@/types/ssh";
import { shortcut } from "@/utils/platform";

/**
 * Écran Serveurs — implémentation fidèle de « ScreenServers.dc.html ».
 *
 * Les entrées de la base chiffrée et celles du `~/.ssh/config` sont ramenées à
 * une vue commune (`ServerItem`) : même carte, mêmes groupes, mêmes actions.
 * Seule une pastille « config » distingue l'origine.
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

// --- Normalisation des deux sources ---
function fromServer(s: Server): ServerItem {
  return {
    key: `local:${s.id}`,
    source: "local",
    id: s.id,
    alias: s.name,
    name: s.name,
    hostname: s.hostname,
    port: s.port,
    username: s.username,
    identityFile: s.identityFile,
    color: s.color,
    favorite: s.favorite,
    tags: s.tags,
    group: s.group,
    os: s.os,
    sourceFile: null,
  };
}

function fromHost(h: Host): ServerItem {
  return {
    key: `config:${h.alias}`,
    source: "config",
    id: null,
    alias: h.alias,
    name: h.alias,
    hostname: h.hostname ?? h.alias,
    port: h.port ?? 22,
    username: h.user,
    identityFile: h.identityFile,
    color: h.color,
    favorite: h.favorite,
    tags: h.tags,
    group: h.group,
    os: h.os,
    sourceFile: h.sourceFile,
  };
}

const allItems = computed<ServerItem[]>(() => [
  ...servers.value.map(fromServer),
  ...hosts.value.map(fromHost),
]);

function match(values: (string | null)[]): boolean {
  const q = search.value.trim().toLowerCase();
  if (!q) return true;
  return values
    .filter((v): v is string => Boolean(v))
    .some((v) => v.toLowerCase().includes(q));
}

const sortedItems = computed<ServerItem[]>(() => {
  const list = allItems.value.filter((i) =>
    match([i.name, i.hostname, i.username, ...i.tags]),
  );
  const sorted = [...list];
  if (serversSort.value === "favorite") {
    sorted.sort(
      (a, b) => Number(b.favorite) - Number(a.favorite) || a.name.localeCompare(b.name),
    );
  } else if (serversSort.value === "recent") {
    // Les entrées de la base d'abord, les plus récentes en tête.
    sorted.sort((a, b) => (b.id ?? -1) - (a.id ?? -1));
  } else {
    sorted.sort((a, b) => a.name.localeCompare(b.name));
  }
  return sorted;
});

/**
 * Section d'affichage d'un serveur.
 *
 * Les entrées du `~/.ssh/config` sont d'abord rangées par fichier de
 * déclaration : avec des `Include`, savoir d'où vient un hôte vaut mieux qu'un
 * « Sans groupe » qui les mélangerait tous. Le groupe s'y ajoute quand il y en
 * a un. Les serveurs de la base chiffrée gardent le comportement d'origine.
 */
interface Section {
  key: string;
  label: string;
  /** Groupe réel, sans le fichier : c'est lui qu'un nouveau serveur héritera. */
  group: string | null;
  source: ServerSource;
  items: ServerItem[];
}

function sectionKey(item: ServerItem): string {
  if (item.source !== "config") return item.group ?? "";
  const file = item.sourceFile ?? "~/.ssh/config";
  return item.group ? `${file} › ${item.group}` : file;
}

const sections = computed<Section[]>(() => {
  const map = new Map<string, Section>();
  for (const item of sortedItems.value) {
    const key = sectionKey(item);
    let section = map.get(key);
    if (!section) {
      section = {
        key,
        label: key || "Sans groupe",
        group: item.group,
        source: item.source,
        items: [],
      };
      map.set(key, section);
    }
    section.items.push(item);
  }
  return [...map.values()].sort((a, b) => {
    if (a.key === "") return 1;
    if (b.key === "") return -1;
    return a.key.localeCompare(b.key);
  });
});

const groupNames = computed(() =>
  [
    ...new Set(allItems.value.map((i) => i.group).filter((g): g is string => Boolean(g))),
  ].sort(),
);

/** Libellés des sessions ouvertes → point vert « connecté » sur les cartes. */
const connectedLabels = computed(
  () => new Set(connections.tabs.flatMap((t) => t.panes.map((p) => p.label))),
);

function toggleGroup(name: string): void {
  if (collapsed.has(name)) collapsed.delete(name);
  else collapsed.add(name);
}

// --- Connexion ---
function connect(item: ServerItem): void {
  const args: string[] =
    item.source === "config"
      ? // OpenSSH résout lui-même les options de l'alias.
        [item.alias]
      : [
          ...(item.port !== 22 ? ["-p", String(item.port)] : []),
          ...(item.identityFile ? ["-i", item.identityFile] : []),
          item.username ? `${item.username}@${item.hostname}` : item.hostname,
        ];
  connections.open(item.name, args, item.key);
  router.push({ name: "terminal" });
}

// --- Création / édition ---
const dialogOpen = ref(false);
const editingItem = ref<ServerItem | null>(null);
const dialogGroup = ref<string | null>(null);
const dialogSource = ref<ServerSource>("local");

function openNew(group: string | null = null, source: ServerSource = "local"): void {
  editingItem.value = null;
  dialogGroup.value = group;
  // Ajouter depuis une section du fichier de config y crée l'entrée : c'est
  // ce que le contexte laisse attendre.
  dialogSource.value = source;
  dialogOpen.value = true;
}

function openEdit(item: ServerItem): void {
  editingItem.value = item;
  dialogGroup.value = null;
  dialogOpen.value = true;
}

async function onSave(result: ServerFormResult): Promise<void> {
  // Changer d'emplacement revient à recréer l'entrée à destination puis à
  // retirer l'ancienne. La création passe d'abord : en cas d'échec, rien n'est
  // perdu.
  const moving =
    result.previousSource !== null && result.previousSource !== result.source;

  const configInput = () => ({
    alias: result.name,
    hostname: result.hostname,
    user: result.username,
    port: result.port,
    identityFile: result.identityFile,
    proxyJump: null,
    group: result.group,
    color: result.color,
    tags: result.tags,
    favorite: result.favorite,
    os: result.os,
  });
  const localInput = () => ({
    name: result.name,
    hostname: result.hostname,
    port: result.port,
    username: result.username,
    identityFile: result.identityFile,
    color: result.color,
    favorite: result.favorite,
    tags: result.tags,
    group: result.group,
    os: result.os,
  });

  try {
    if (result.source === "config") {
      if (!moving && result.originalAlias) {
        await hostsStore.update(result.originalAlias, configInput());
      } else {
        await hostsStore.create(configInput());
        if (moving && result.id !== null) await myServers.remove(result.id);
      }
    } else {
      if (!moving && result.id !== null) {
        await myServers.update(result.id, localInput());
      } else {
        await myServers.create(localInput());
        if (moving && result.originalAlias) {
          await hostsStore.remove(result.originalAlias);
        }
      }
    }
    dialogOpen.value = false;
  } catch {
    /* erreur affichée par le store correspondant */
  }
}

async function onRemove(item: ServerItem): Promise<void> {
  const where = item.source === "config" ? " de ~/.ssh/config" : "";
  if (!window.confirm(`Supprimer « ${item.name} »${where} ?`)) return;
  if (item.source === "config") await hostsStore.remove(item.alias);
  else if (item.id !== null) await myServers.remove(item.id);
}

async function toggleFavorite(item: ServerItem): Promise<void> {
  await onSave({
    source: item.source,
    previousSource: item.source,
    id: item.id,
    originalAlias: item.source === "config" ? item.alias : null,
    name: item.name,
    hostname: item.hostname,
    port: item.port,
    username: item.username,
    identityFile: item.identityFile,
    color: item.color,
    favorite: !item.favorite,
    tags: item.tags,
    group: item.group,
    os: item.os,
  });
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

function addressOf(item: ServerItem): string {
  const target = item.username ? `${item.username}@${item.hostname}` : item.hostname;
  return `${target}:${item.port}`;
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
      <p v-if="hostsError" class="state state--error">{{ hostsError }}</p>

      <section v-for="section in sections" :key="section.key || '__none'" class="group">
        <header class="group__head">
          <button
            class="group__chevron"
            :class="{ 'group__chevron--closed': collapsed.has(section.key) }"
            @click="toggleGroup(section.key)"
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
          <span class="group__name">{{ section.label }}</span>
          <span class="group__count">{{ section.items.length }}</span>
          <span class="group__line" />
        </header>

        <template v-if="!collapsed.has(section.key)">
          <!-- Vue grille -->
          <div v-if="serversView === 'grid'" class="grid">
            <ServerCard
              v-for="item in section.items"
              :key="item.key"
              :item="item"
              :connected="connectedLabels.has(item.name)"
              @connect="connect"
              @edit="openEdit"
              @remove="onRemove"
              @toggle-favorite="toggleFavorite"
            />
            <button class="addcard" @click="openNew(section.group, section.source)">
              <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
                <path
                  d="M7 2.4v9.2M2.4 7h9.2"
                  stroke="currentColor"
                  stroke-width="1.7"
                  stroke-linecap="round"
                />
              </svg>
              {{ section.group ? `Ajouter à ${section.group}` : "Ajouter un serveur" }}
            </button>
          </div>

          <!-- Vue liste -->
          <div v-else class="rows">
            <div
              v-for="item in section.items"
              :key="item.key"
              class="row"
              @dblclick="connect(item)"
            >
              <OsBadge
                class="row__ava"
                :os="item.os"
                :name="item.name"
                :color="item.color"
                :size="24"
              />
              <span class="row__name">
                {{ item.name }}
                <span v-if="connectedLabels.has(item.name)" class="row__dot" />
                <span v-if="item.source === 'config'" class="row__origin">config</span>
              </span>
              <span class="row__addr">{{ addressOf(item) }}</span>
              <span class="row__tags">
                <span v-for="tag in item.tags" :key="tag" class="row__tag">{{
                  tag
                }}</span>
              </span>
              <span class="row__actions">
                <button class="row__connect" @click.stop="connect(item)">
                  Connecter
                </button>
                <button class="row__icon" title="Éditer" @click.stop="openEdit(item)">
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
                  @click.stop="onRemove(item)"
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

      <!-- État vide -->
      <div v-if="sortedItems.length === 0 && !hostsLoading" class="empty">
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
      :item="editingItem"
      :groups="groupNames"
      :default-group="dialogGroup"
      :default-source="dialogSource"
      @save="onSave"
      @close="dialogOpen = false"
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

.group__count {
  font-family: var(--g-font-mono);
  font-size: 10.5px;
  color: var(--g-t3);
  background: var(--g-s2);
  padding: 1px 6px;
  border-radius: 6px;
}

.group__line {
  flex: 1;
  height: 1px;
  background: var(--g-border);
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
  border-radius: 8px;
}

.row__name {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 200px;
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

.row__origin {
  font-family: var(--g-font-mono);
  font-size: 9.5px;
  font-weight: 500;
  color: var(--g-t3);
  border: 1px solid var(--g-border);
  padding: 0 5px;
  border-radius: 5px;
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
