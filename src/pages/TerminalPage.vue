<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import TerminalView from "@/components/TerminalView.vue";

const route = useRoute();
const router = useRouter();

const host = computed(() => String(route.params.alias));
</script>

<template>
  <section class="terminal-page">
    <header class="terminal-page__bar">
      <button class="back" title="Retour" @click="router.push({ name: 'servers' })">
        <i class="pi pi-arrow-left" />
      </button>
      <span class="terminal-page__title">
        <i class="pi pi-desktop" /> {{ host }}
      </span>
      <span class="terminal-page__hint">connexion via <code>ssh {{ host }}</code></span>
    </header>

    <TerminalView :key="host" :host="host" class="terminal-page__view" />
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
  width: 36px;
  height: 36px;
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
</style>
