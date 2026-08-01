<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";
import { useServersStore } from "@/stores/servers";
import { useMyServersStore } from "@/stores/myServers";
import { useConnectionsStore } from "@/stores/connections";
import HostCard from "@/components/HostCard.vue";
import ServerCard from "@/components/ServerCard.vue";
import ServerFormDialog from "@/components/ServerFormDialog.vue";
import type { Host, Server, ServerInput } from "@/types/ssh";

const hostsStore = useServersStore();
const myServers = useMyServersStore();
const connections = useConnectionsStore();
const router = useRouter();

const { hosts, loading: hostsLoading, error: hostsError } = storeToRefs(hostsStore);
const { servers, error: serversError } = storeToRefs(myServers);

const search = ref("");

function match(values: (string | null)[]): boolean {
  const q = search.value.trim().toLowerCase();
  if (!q) return true;
  return values
    .filter((v): v is string => Boolean(v))
    .some((v) => v.toLowerCase().includes(q));
}

const filteredServers = computed(() =>
  servers.value.filter((s) => match([s.name, s.hostname, s.username])),
);
const filteredHosts = computed(() =>
  hosts.value.filter((h) => match([h.alias, h.hostname, h.user])),
);

// --- Connexions (ouvrent un onglet dans l'espace terminal) ---
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

// --- CRUD via dialog ---
const dialogOpen = ref(false);
const editingServer = ref<Server | null>(null);

function openNew(): void {
  editingServer.value = null;
  dialogOpen.value = true;
}
function openEdit(server: Server): void {
  editingServer.value = server;
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

onMounted(() => {
  if (hosts.value.length === 0) hostsStore.load();
  if (servers.value.length === 0) myServers.load();
});
</script>

<template>
  <section class="page">
    <div class="toolbar">
      <div class="toolbar__search">
        <i class="pi pi-search" />
        <input
          v-model="search"
          type="text"
          placeholder="Rechercher un serveur ou un hôte…"
          spellcheck="false"
        />
      </div>
      <button class="btn btn--primary" @click="openNew">
        <i class="pi pi-plus" /> Nouveau serveur
      </button>
    </div>

    <p v-if="serversError" class="state state--error">
      <i class="pi pi-exclamation-triangle" /> {{ serversError }}
    </p>

    <!-- Mes serveurs (base chiffrée) -->
    <template v-if="filteredServers.length > 0">
      <header class="page__header">
        <h2>Mes serveurs</h2>
        <span class="page__count">{{ filteredServers.length }}</span>
      </header>
      <div class="grid">
        <ServerCard
          v-for="server in filteredServers"
          :key="server.id"
          :server="server"
          @connect="connectServer"
          @edit="openEdit"
          @remove="onRemove"
        />
      </div>
    </template>

    <!-- Hôtes ~/.ssh/config (lecture seule) -->
    <header v-if="filteredHosts.length > 0" class="page__header page__header--spaced">
      <h2>Depuis <code>~/.ssh/config</code></h2>
      <span class="page__count">{{ filteredHosts.length }}</span>
    </header>
    <p v-if="hostsError" class="state state--error">
      <i class="pi pi-exclamation-triangle" /> {{ hostsError }}
    </p>
    <div v-if="filteredHosts.length > 0" class="grid">
      <HostCard
        v-for="host in filteredHosts"
        :key="host.alias"
        :host="host"
        @click="connectHost(host)"
      />
    </div>

    <!-- Vide -->
    <div
      v-if="filteredServers.length === 0 && filteredHosts.length === 0 && !hostsLoading"
      class="state state--empty"
    >
      <i class="pi pi-server" />
      <p v-if="search">Aucun résultat pour « {{ search }} ».</p>
      <p v-else>Aucun serveur. Clique sur « Nouveau serveur » pour commencer.</p>
    </div>

    <p class="page__hint">
      <i class="pi pi-info-circle" /> Serveurs stockés dans une base
      <strong>chiffrée</strong> (clé dans le trousseau). Double-clic ou « Connecter »
      pour ouvrir une session ; clic sur un hôte du config pour s'y connecter.
    </p>

    <ServerFormDialog
      :open="dialogOpen"
      :server="editingServer"
      @save="onSave"
      @close="dialogOpen = false"
    />
  </section>
</template>

<style scoped>
.toolbar {
  display: flex;
  gap: 0.6rem;
  margin-bottom: 1.5rem;
}

.toolbar__search {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
  padding: 0 0.85rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 10px;
  background: var(--p-content-background);
  color: var(--p-text-muted-color);
}

.toolbar__search:focus-within {
  border-color: var(--p-primary-color);
}

.toolbar__search input {
  flex: 1;
  padding: 0.65rem 0;
  border: 0;
  background: transparent;
  color: var(--p-text-color);
  font: inherit;
  outline: none;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0 1rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 10px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  font: inherit;
  cursor: pointer;
  transition: background-color 0.15s ease;
  white-space: nowrap;
}

.btn:hover {
  background: var(--p-content-hover-background);
}

.btn--primary {
  border-color: var(--p-primary-color);
  background: var(--p-primary-color);
  color: var(--p-primary-contrast-color, #fff);
}

.btn--primary:hover {
  filter: brightness(1.05);
  background: var(--p-primary-color);
}

.page__header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-bottom: 1rem;
}

.page__header--spaced {
  margin-top: 2rem;
}

.page__header h2 {
  margin: 0;
  font-size: 1.15rem;
}

.page__header code {
  font-size: 0.9em;
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  background: var(--p-content-hover-background);
}

.page__count {
  padding: 0.05rem 0.5rem;
  border-radius: 999px;
  background: var(--p-content-hover-background);
  color: var(--p-text-muted-color);
  font-size: 0.8rem;
  font-weight: 600;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 0.85rem;
}

.state {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--p-text-muted-color);
}

.state--error {
  color: #ef4444;
}

.state--empty {
  flex-direction: column;
  justify-content: center;
  gap: 0.75rem;
  margin-top: 2rem;
  padding: 3rem;
  border: 1px dashed var(--p-content-border-color);
  border-radius: 12px;
}

.state--empty .pi {
  font-size: 2.5rem;
  opacity: 0.5;
}

.page__hint {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-top: 1.75rem;
  font-size: 0.82rem;
  color: var(--p-text-muted-color);
}
</style>
