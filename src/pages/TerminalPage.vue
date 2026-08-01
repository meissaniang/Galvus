<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useConnectionsStore } from "@/stores/connections";
import TerminalView from "@/components/TerminalView.vue";

const route = useRoute();
const router = useRouter();
const connections = useConnectionsStore();

const id = computed(() => String(route.params.id));
const target = computed(() => connections.get(id.value));

function goBack(): void {
  router.push({ name: "servers" });
}
</script>

<template>
  <section class="terminal-page">
    <header class="terminal-page__bar">
      <button class="back" title="Retour" @click="goBack">
        <i class="pi pi-arrow-left" />
      </button>
      <span class="terminal-page__title">
        <i class="pi pi-desktop" /> {{ target?.label ?? "Session" }}
      </span>
      <span v-if="target" class="terminal-page__hint">
        <code>ssh {{ target.args.join(" ") }}</code>
      </span>
    </header>

    <TerminalView v-if="target" :key="id" :args="target.args" class="terminal-page__view" />

    <div v-else class="terminal-page__missing">
      <i class="pi pi-info-circle" />
      <p>Session introuvable (relancée après rechargement).</p>
      <button class="back" @click="goBack">Retour aux serveurs</button>
    </div>
  </section>
</template>

<style scoped>
.terminal-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 0.85rem;
}

.terminal-page__bar {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.back {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  min-width: 36px;
  height: 36px;
  padding: 0 0.6rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 9px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.back:hover {
  background: var(--p-content-hover-background);
}

.terminal-page__title {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 1.1rem;
  font-weight: 600;
}

.terminal-page__hint {
  margin-left: auto;
  font-size: 0.82rem;
  color: var(--p-text-muted-color);
}

.terminal-page__hint code {
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  background: var(--p-content-hover-background);
}

.terminal-page__view {
  flex: 1;
  min-height: 0;
}

.terminal-page__missing {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  flex: 1;
  color: var(--p-text-muted-color);
}
</style>
