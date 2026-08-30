<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";
import { useServersStore } from "@/stores/servers";
import { useMyServersStore } from "@/stores/myServers";
import { useConnectionsStore } from "@/stores/connections";
import { useGroupsStore } from "@/stores/groups";
import { useSettingsStore } from "@/stores/settings";
import ServerCard from "@/components/ServerCard.vue";
import OsBadge from "@/components/OsBadge.vue";
import ServerFormDialog, {
  type ServerFormResult,
} from "@/components/ServerFormDialog.vue";
import GroupDialog from "@/components/GroupDialog.vue";
import type { Host, Server, ServerItem, ServerSource } from "@/types/ssh";
import { shortcut } from "@/utils/platform";
import {
  ancestorsOf,
  isWithin,
  joinPath,
  nameOf,
  normalizePath,
  parentOf,
} from "@/utils/groupPath";

/**
 * Écran Serveurs — implémentation fidèle de « ScreenServers.dc.html ».
 *
 * Les entrées de la base chiffrée et celles du `~/.ssh/config` sont ramenées à
 * une vue commune (`ServerItem`) : même carte, mêmes dossiers, mêmes actions.
 * Seule une pastille « config » distingue l'origine.
 *
 * Les groupes se parcourent comme des dossiers : on entre dans l'un d'eux, on
 * y voit ses sous-dossiers et ses serveurs. La recherche, elle, traverse toute
 * l'arborescence — chercher ne doit pas obliger à savoir où l'on a rangé.
 */
const hostsStore = useServersStore();
const myServers = useMyServersStore();
const connections = useConnectionsStore();
const groups = useGroupsStore();
const settings = useSettingsStore();
const router = useRouter();

const { hosts, loading: hostsLoading, error: hostsError } = storeToRefs(hostsStore);
const { servers, error: serversError } = storeToRefs(myServers);
const { error: groupsError } = storeToRefs(groups);
const { serversView, serversSort } = storeToRefs(settings);

const search = ref("");
const searchInput = ref<HTMLInputElement | null>(null);

/** Dossier ouvert. Chaîne vide à la racine. */
const cwd = ref("");

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
 * Tous les dossiers connus, triés.
 *
 * Deux origines les nourrissent : ceux créés explicitement (table `groups`,
 * seule à savoir garder un dossier vide) et ceux déduits du chemin des
 * serveurs — un hôte du `~/.ssh/config` peut arriver avec un `group=` que la
 * base n'a jamais vu.
 */
const folders = computed<string[]>(() => {
  const known = new Set(groups.paths);
  for (const item of allItems.value) {
    for (const ancestor of ancestorsOf(normalizePath(item.group))) known.add(ancestor);
  }
  return [...known].sort((a, b) => a.localeCompare(b));
});

/** Nombre de serveurs rangés dans un dossier, sous-dossiers compris. */
function countIn(path: string): number {
  return allItems.value.filter((item) => isWithin(normalizePath(item.group), path))
    .length;
}

/** Contenu d'un dossier, en toutes lettres. */
function countLabel(count: number): string {
  if (count === 0) return "Vide";
  return `${count} serveur${count > 1 ? "s" : ""}`;
}

/** Sous-dossiers directs du dossier ouvert. */
const subfolders = computed(() =>
  folders.value
    .filter((path) => parentOf(path) === cwd.value)
    .map((path) => ({ path, name: nameOf(path), count: countIn(path) })),
);

const searching = computed(() => search.value.trim() !== "");

/** Fil d'Ariane : la racine, puis chaque niveau ouvert. */
const breadcrumb = computed(() => [
  { label: "Serveurs", path: "" },
  ...ancestorsOf(cwd.value).map((path) => ({ label: nameOf(path), path })),
]);

/** Un lot de serveurs à afficher, avec le dossier d'où il vient. */
interface Section {
  path: string;
  label: string;
  items: ServerItem[];
}

/**
 * Ce que la liste affiche : le contenu du dossier ouvert, ou — en recherche —
 * les résultats de toute l'arborescence, regroupés par dossier d'origine.
 *
 * Un seul chemin de rendu pour les deux cas : la grille et la liste n'ont pas
 * à savoir laquelle des deux situations les alimente.
 */
const sections = computed<Section[]>(() => {
  if (!searching.value) {
    const items = sortedItems.value.filter(
      (item) => normalizePath(item.group) === cwd.value,
    );
    return [{ path: cwd.value, label: "", items }];
  }
  const map = new Map<string, ServerItem[]>();
  for (const item of sortedItems.value) {
    const path = normalizePath(item.group);
    const bucket = map.get(path);
    if (bucket) bucket.push(item);
    else map.set(path, [item]);
  }
  return [...map.entries()]
    .map(([path, items]) => ({ path, label: path || "Serveurs", items }))
    .sort((a, b) => a.label.localeCompare(b.label));
});

/** Nombre de serveurs actuellement affichés, toutes sections confondues. */
const visibleCount = computed(() =>
  sections.value.reduce((total, section) => total + section.items.length, 0),
);

/**
 * Vrai quand il n'y a rigoureusement rien à montrer.
 *
 * Dans ce cas l'encart d'état vide remplace la carte « Ajouter » : deux
 * invitations à créer, l'une sous l'autre, ne valent pas mieux qu'une.
 */
const showEmpty = computed(
  () =>
    !hostsLoading.value &&
    visibleCount.value === 0 &&
    (searching.value || subfolders.value.length === 0),
);

function enter(path: string): void {
  search.value = "";
  cwd.value = path;
}

// Le dossier ouvert peut disparaître sous nos pieds (suppression, renommage
// depuis un autre écran) : on remonte au plus proche ancêtre encore présent
// plutôt que d'afficher un dossier fantôme.
watch(folders, (list) => {
  if (!cwd.value || list.includes(cwd.value)) return;
  cwd.value =
    ancestorsOf(cwd.value)
      .filter((path) => list.includes(path))
      .pop() ?? "";
});

// --- Dossiers : création, renommage, suppression ---
const folderDialog = reactive({
  open: false,
  mode: "create" as "create" | "rename",
  /** Dossier parent à la création, dossier visé au renommage. */
  path: "",
});

function openNewFolder(): void {
  groups.error = null;
  folderDialog.mode = "create";
  folderDialog.path = cwd.value;
  folderDialog.open = true;
}

function openRenameFolder(path: string): void {
  groups.error = null;
  folderDialog.mode = "rename";
  folderDialog.path = path;
  folderDialog.open = true;
}

async function onFolderSubmit(name: string): Promise<void> {
  try {
    if (folderDialog.mode === "create") {
      await groups.create(joinPath(folderDialog.path, name));
    } else {
      const renamed = await groups.rename(folderDialog.path, name);
      // Suivre le renommage quand il touche le dossier ouvert ou l'un de ses
      // ancêtres : rester sur l'ancien chemin afficherait un dossier vide.
      if (isWithin(cwd.value, folderDialog.path)) {
        cwd.value = renamed + cwd.value.slice(folderDialog.path.length);
      }
    }
    folderDialog.open = false;
  } catch {
    /* message affiché dans le dialog */
  }
}

async function removeFolder(path: string): Promise<void> {
  const count = countIn(path);
  const parent = parentOf(path);
  const destination = parent ? `« ${nameOf(parent)} »` : "la racine";
  const consequence =
    count === 0
      ? ""
      : `\n\nSes ${count} serveur${count > 1 ? "s" : ""} ${
          count > 1 ? "remonteront" : "remontera"
        } dans ${destination} : rien n'est supprimé.`;
  if (!window.confirm(`Supprimer le dossier « ${nameOf(path)} » ?${consequence}`)) return;
  await groups.remove(path);
}

// --- Glisser-déposer : ranger un serveur dans un dossier ---
/** Serveur en cours de glissement, ou `null`. */
const dragged = ref<ServerItem | null>(null);
/** Dossier survolé pendant le glissement ; `""` pour la racine, `null` si aucun. */
const dropTarget = ref<string | null>(null);

/** Un dossier n'accepte le serveur que s'il ne s'y trouve pas déjà. */
function accepts(path: string): boolean {
  return dragged.value !== null && normalizePath(dragged.value.group) !== path;
}

function onDragStart(item: ServerItem, event: DragEvent): void {
  dragged.value = item;
  dropTarget.value = null;
  // Sans données, Safari/WebKit refuse d'amorcer le glissement.
  event.dataTransfer?.setData("text/plain", item.key);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "move";
}

function onDragEnd(): void {
  dragged.value = null;
  dropTarget.value = null;
}

/** Le dépôt n'est autorisé que si `preventDefault` est appelé ici. */
function onDragOver(path: string, event: DragEvent): void {
  if (!accepts(path)) return;
  event.preventDefault();
  if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
  dropTarget.value = path;
}

function onDragLeave(path: string, event: DragEvent): void {
  // Passer sur un enfant déclenche un `dragleave` sur le parent : on n'éteint
  // la surbrillance que si le curseur a vraiment quitté la zone.
  const zone = event.currentTarget as HTMLElement;
  const to = event.relatedTarget as Node | null;
  if (to && zone.contains(to)) return;
  if (dropTarget.value === path) dropTarget.value = null;
}

async function onDrop(path: string): Promise<void> {
  const item = dragged.value;
  onDragEnd();
  if (!item || normalizePath(item.group) === path) return;
  await onSave(resultFrom(item, { group: path || null }));
}

/** Libellés des sessions ouvertes → point vert « connecté » sur les cartes. */
const connectedLabels = computed(
  () => new Set(connections.tabs.flatMap((t) => t.panes.map((p) => p.label))),
);

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

/** Un nouveau serveur naît dans le dossier ouvert : c'est ce qu'on attend. */
function openNew(): void {
  editingItem.value = null;
  dialogGroup.value = cwd.value || null;
  dialogSource.value = "local";
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
    // Un dossier tapé dans le formulaire doit exister pour lui survivre : sans
    // cela, il s'évaporerait dès que son dernier serveur en sortirait.
    await groups.ensure(result.group);
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

/**
 * Réécrit une fiche existante en ne changeant que les champs passés.
 *
 * Déplacer un serveur ou basculer son favori passe par le même chemin
 * d'enregistrement qu'une édition au formulaire : une seule règle de mise à
 * jour, valable pour les deux origines.
 */
function resultFrom(
  item: ServerItem,
  changes: Partial<ServerFormResult>,
): ServerFormResult {
  return {
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
    favorite: item.favorite,
    tags: item.tags,
    group: item.group,
    os: item.os,
    ...changes,
  };
}

async function toggleFavorite(item: ServerItem): Promise<void> {
  await onSave(resultFrom(item, { favorite: !item.favorite }));
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
  groups.load();
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

      <!-- Fil d'Ariane : où l'on se trouve, et les actions du dossier -->
      <nav class="crumbs">
        <template v-for="(crumb, index) in breadcrumb" :key="crumb.path || '__root'">
          <svg
            v-if="index > 0"
            class="crumbs__sep"
            width="10"
            height="10"
            viewBox="0 0 12 12"
            fill="none"
          >
            <path
              d="M4.4 2.6l3.2 3.4-3.2 3.4"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          <button
            class="crumbs__item"
            :class="{
              'crumbs__item--current': index === breadcrumb.length - 1,
              'crumbs__item--over': dropTarget === crumb.path,
            }"
            @click="enter(crumb.path)"
            @dragover="onDragOver(crumb.path, $event)"
            @dragleave="onDragLeave(crumb.path, $event)"
            @drop.prevent="onDrop(crumb.path)"
          >
            {{ crumb.label }}
          </button>
        </template>

        <span class="crumbs__line" />

        <template v-if="cwd">
          <button class="crumbs__act" @click="openRenameFolder(cwd)">Renommer</button>
          <button class="crumbs__act crumbs__act--danger" @click="removeFolder(cwd)">
            Supprimer
          </button>
        </template>
        <button class="crumbs__act crumbs__act--new" @click="openNewFolder">
          <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
            <path
              d="M7 3.6v6.8M3.6 7h6.8"
              stroke="currentColor"
              stroke-width="1.7"
              stroke-linecap="round"
            />
          </svg>
          Nouveau dossier
        </button>
      </nav>

      <p v-if="groupsError" class="state state--error">{{ groupsError }}</p>

      <!-- Sous-dossiers du dossier ouvert -->
      <div
        v-if="!searching && subfolders.length"
        :class="serversView === 'grid' ? 'grid' : 'rows'"
      >
        <div
          v-for="folder in subfolders"
          :key="folder.path"
          class="folder"
          :class="[
            serversView === 'grid' ? 'folder--tile' : 'folder--row',
            { 'folder--over': dropTarget === folder.path },
          ]"
          role="button"
          tabindex="0"
          @click="enter(folder.path)"
          @keydown.enter.prevent="enter(folder.path)"
          @keydown.space.prevent="enter(folder.path)"
          @dragover="onDragOver(folder.path, $event)"
          @dragleave="onDragLeave(folder.path, $event)"
          @drop.prevent="onDrop(folder.path)"
        >
          <span class="folder__ava">
            <svg viewBox="0 0 18 18" fill="none">
              <path
                d="M2.4 5.2a1.8 1.8 0 011.8-1.8h2.6l1.6 1.9h5.4a1.8 1.8 0 011.8 1.8v5.7a1.8 1.8 0 01-1.8 1.8H4.2a1.8 1.8 0 01-1.8-1.8z"
                stroke="currentColor"
                stroke-width="1.3"
                stroke-linejoin="round"
              />
            </svg>
          </span>
          <span class="folder__body">
            <span class="folder__name">{{ folder.name }}</span>
            <span class="folder__count">{{ countLabel(folder.count) }}</span>
          </span>
          <span class="folder__actions">
            <button
              class="folder__act"
              title="Renommer"
              @click.stop="openRenameFolder(folder.path)"
            >
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
              class="folder__act folder__act--danger"
              title="Supprimer"
              @click.stop="removeFolder(folder.path)"
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

      <!-- Serveurs : contenu du dossier, ou résultats de recherche par dossier -->
      <section v-for="section in sections" :key="section.path || '__root'" class="group">
        <header v-if="searching && section.items.length" class="group__head">
          <span class="group__name">{{ section.label }}</span>
          <span class="group__count">{{ section.items.length }}</span>
          <span class="group__line" />
          <button class="group__open" @click="enter(section.path)">
            Ouvrir le dossier
          </button>
        </header>

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
            @dragstart="onDragStart"
            @dragend="onDragEnd"
          />
          <button v-if="!searching && !showEmpty" class="addcard" @click="openNew()">
            <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
              <path
                d="M7 2.4v9.2M2.4 7h9.2"
                stroke="currentColor"
                stroke-width="1.7"
                stroke-linecap="round"
              />
            </svg>
            {{ cwd ? `Ajouter à ${nameOf(cwd)}` : "Ajouter un serveur" }}
          </button>
        </div>

        <!-- Vue liste -->
        <div v-else class="rows">
          <div
            v-for="item in section.items"
            :key="item.key"
            class="row"
            draggable="true"
            @dblclick="connect(item)"
            @dragstart="onDragStart(item, $event)"
            @dragend="onDragEnd"
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
              <span v-for="tag in item.tags" :key="tag" class="row__tag">{{ tag }}</span>
            </span>
            <span class="row__actions">
              <button class="row__connect" @click.stop="connect(item)">Connecter</button>
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
      </section>

      <!-- État vide -->
      <div v-if="showEmpty" class="empty">
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
        <p v-if="searching">Aucun résultat pour « {{ search }} ».</p>
        <p v-else-if="cwd">Ce dossier est vide.</p>
        <p v-else>Aucun serveur. Clique sur « Nouveau serveur » pour commencer.</p>
      </div>
    </div>

    <ServerFormDialog
      :open="dialogOpen"
      :item="editingItem"
      :groups="folders"
      :default-group="dialogGroup"
      :default-source="dialogSource"
      @save="onSave"
      @close="dialogOpen = false"
    />

    <GroupDialog
      :open="folderDialog.open"
      :mode="folderDialog.mode"
      :path="folderDialog.path"
      :error="groupsError"
      @submit="onFolderSubmit"
      @close="folderDialog.open = false"
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

/* ---------- Fil d'Ariane ---------- */
.crumbs {
  display: flex;
  align-items: center;
  gap: 5px;
  min-height: 28px;
}

.crumbs__item {
  border: 0;
  border-radius: 7px;
  padding: 3px 7px;
  background: transparent;
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--g-t3);
  cursor: pointer;
  transition:
    background 0.12s ease,
    color 0.12s ease;
}

.crumbs__item:hover:not(.crumbs__item--current) {
  background: var(--g-s2);
  color: var(--g-t1);
}

/* Cible de dépôt survolée pendant un glissement. */
.crumbs__item--over {
  background: var(--g-accent-soft);
  color: var(--g-accent);
  box-shadow: inset 0 0 0 1px var(--g-accent-ring);
}

.crumbs__item--current {
  color: var(--g-t1);
  font-weight: 600;
  cursor: default;
}

.crumbs__sep {
  color: var(--g-t3);
  flex-shrink: 0;
}

.crumbs__line {
  flex: 1;
  height: 1px;
  background: var(--g-border);
  margin: 0 6px;
}

.crumbs__act {
  display: flex;
  align-items: center;
  gap: 5px;
  height: 26px;
  padding: 0 9px;
  border: 1px solid var(--g-border);
  border-radius: 8px;
  background: var(--g-s1);
  font-family: inherit;
  font-size: 11.5px;
  font-weight: 500;
  color: var(--g-t2);
  cursor: pointer;
  white-space: nowrap;
  transition:
    background 0.12s ease,
    color 0.12s ease,
    border-color 0.12s ease;
}

.crumbs__act:hover {
  background: var(--g-s2);
  color: var(--g-t1);
}

.crumbs__act--new:hover {
  color: var(--g-accent);
  border-color: var(--g-accent-ring);
}

.crumbs__act--danger:hover {
  background: var(--g-danger-soft);
  border-color: var(--g-danger);
  color: var(--g-danger);
}

/* ---------- Dossiers ---------- */
/*
 * Un dossier se présente comme un serveur : tuile en vue grille, ligne en vue
 * liste. Même balisage, deux habillages — les colonnes de `.folder--row` sont
 * calées sur celles de `.row` pour que dossiers et serveurs s'alignent.
 */
.folder {
  display: flex;
  align-items: center;
  gap: 11px;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  cursor: pointer;
  outline: none;
}

.folder__ava {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  color: var(--g-accent);
  flex-shrink: 0;
}

.folder__body {
  display: flex;
  flex: 1;
  min-width: 0;
}

.folder__name {
  font-weight: 600;
  color: var(--g-t1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.folder__count {
  color: var(--g-t3);
  white-space: nowrap;
}

.folder__actions {
  display: flex;
  gap: 5px;
  opacity: 0;
  transition: opacity 0.12s ease-out;
}

.folder:hover .folder__actions,
.folder:focus-within .folder__actions {
  opacity: 1;
}

.folder__act {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--g-border);
  border-radius: 8px;
  background: var(--g-s0);
  color: var(--g-t2);
  cursor: pointer;
}

.folder__act:hover {
  color: var(--g-t1);
}

.folder__act--danger:hover {
  background: var(--g-danger-soft);
  border-color: var(--g-danger);
  color: var(--g-danger);
}

/*
 * Dossier survolé pendant un glissement. Le liseré plein remplace l'anneau
 * translucide du survol ordinaire : « on peut lâcher ici » doit se distinguer
 * de « la souris passe par là ».
 */
.folder--over {
  background: var(--g-accent-soft);
  border-color: var(--g-accent);
}

.folder--over .folder__ava {
  border-color: var(--g-accent-ring);
}

/* Tuile : même gabarit que ServerCard (66 px, ombre, léger soulèvement). */
.folder--tile {
  padding: 13px;
  border-radius: 12px;
  box-shadow: var(--g-sh1);
  transition:
    transform 0.14s cubic-bezier(0.2, 0.8, 0.3, 1),
    box-shadow 0.14s ease,
    background 0.14s ease,
    border-color 0.14s ease;
}

.folder--tile:hover,
.folder--tile:focus-visible {
  background: var(--g-s2);
  border-color: var(--g-accent-ring);
  transform: translateY(-1px);
  box-shadow: var(--g-sh2);
}

.folder--tile .folder__ava {
  width: 40px;
  height: 40px;
  border-radius: 11px;
}

.folder--tile .folder__ava svg {
  width: 18px;
  height: 18px;
}

.folder--tile .folder__body {
  flex-direction: column;
  gap: 4px;
}

.folder--tile .folder__name {
  font-size: 13.5px;
}

.folder--tile .folder__count {
  font-size: 11.5px;
}

/* Ligne : même gabarit que `.row`, colonnes comprises. */
.folder--row {
  height: 44px;
  padding: 0 12px;
  border-radius: 10px;
  transition:
    background 0.12s ease,
    border-color 0.12s ease;
}

.folder--row:hover,
.folder--row:focus-visible {
  background: var(--g-s2);
  border-color: var(--g-accent-ring);
}

.folder--row .folder__ava {
  width: 24px;
  height: 24px;
  border-radius: 8px;
}

.folder--row .folder__ava svg {
  width: 13px;
  height: 13px;
}

.folder--row .folder__body {
  flex-direction: row;
  align-items: center;
  gap: 11px;
}

.folder--row .folder__name {
  /* Même largeur que `.row__name` : les noms se lisent sur une seule colonne. */
  width: 200px;
  flex-shrink: 0;
  font-size: 13px;
}

.folder--row .folder__count {
  flex: 1;
  font-size: 11.5px;
}

.group__head {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-bottom: 11px;
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

.group__open {
  border: 0;
  border-radius: 7px;
  padding: 3px 8px;
  background: transparent;
  font-family: inherit;
  font-size: 11.5px;
  font-weight: 500;
  color: var(--g-t3);
  cursor: pointer;
}

.group__open:hover {
  background: var(--g-s2);
  color: var(--g-accent);
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
  cursor: grab;
  transition:
    background 0.12s ease,
    border-color 0.12s ease;
}

.row:hover {
  background: var(--g-s2);
  border-color: var(--g-accent-ring);
}

/* Le serveur se saisit à la souris. */
.row:active {
  cursor: grabbing;
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
