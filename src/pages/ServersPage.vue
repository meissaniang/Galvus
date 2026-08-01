<script setup lang="ts">
import { onMounted } from "vue";
import { storeToRefs } from "pinia";
import { useServersStore } from "@/stores/servers";
import HostCard from "@/components/HostCard.vue";

const store = useServersStore();
const { filteredHosts, hosts, loading, error, query } = storeToRefs(store);

onMounted(() => {
  if (hosts.value.length === 0) {
    store.load();
  }
});
</script>

<template>
  <section class="page">
    <div class="toolbar">
      <div class="toolbar__search">
        <i class="pi pi-search" />
        <input
          v-model="query"
          type="text"
          placeholder="Rechercher un hôte (alias, hôte, utilisateur)…"
          spellcheck="false"
        />
      </div>
      <button class="btn" :disabled="loading" title="Rafraîchir" @click="store.load()">
        <i class="pi" :class="loading ? 'pi-spin pi-spinner' : 'pi-refresh'" />
      </button>
    </div>

    <header class="page__header">
      <h1>Hôtes</h1>
      <span class="page__count">{{ filteredHosts.length }}</span>
    </header>

    <p v-if="error" class="state state--error">
      <i class="pi pi-exclamation-triangle" /> {{ error }}
    </p>

    <p v-else-if="loading && hosts.length === 0" class="state">
      <i class="pi pi-spin pi-spinner" /> Chargement de ~/.ssh/config…
    </p>

    <div v-else-if="filteredHosts.length === 0" class="state state--empty">
      <i class="pi pi-server" />
      <p v-if="hosts.length === 0">Aucun hôte dans <code>~/.ssh/config</code>.</p>
      <p v-else>Aucun hôte ne correspond à « {{ query }} ».</p>
    </div>

    <div v-else class="grid">
      <HostCard v-for="host in filteredHosts" :key="host.alias" :host="host" />
    </div>

    <p class="page__hint">
      <i class="pi pi-info-circle" /> Données réelles lues via <code>ssh -G</code>. La
      connexion terminal arrive à l'étape suivante du Livrable 1.
    </p>
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
  justify-content: center;
  width: 42px;
  border: 1px solid var(--p-content-border-color);
  border-radius: 10px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.btn:hover:not(:disabled) {
  background: var(--p-content-hover-background);
}

.btn:disabled {
  opacity: 0.6;
  cursor: default;
}

.page__header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-bottom: 1rem;
}

.page__header h1 {
  margin: 0;
  font-size: 1.4rem;
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
  color: var(--p-red-500, #ef4444);
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

code {
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  background: var(--p-content-hover-background);
  font-size: 0.9em;
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
