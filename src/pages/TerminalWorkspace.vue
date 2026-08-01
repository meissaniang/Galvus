<script setup lang="ts">
import { useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { useConnectionsStore } from "@/stores/connections";
import TerminalView from "@/components/TerminalView.vue";

const connections = useConnectionsStore();
const { tabs, activeId } = storeToRefs(connections);
const router = useRouter();
</script>

<template>
  <section class="workspace">
    <div v-if="tabs.length > 0" class="tabs">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        class="tab"
        :class="{ 'tab--active': tab.id === activeId }"
        @click="connections.setActive(tab.id)"
      >
        <i class="pi pi-desktop" />
        <span class="tab__label" :title="tab.label">{{ tab.label }}</span>
        <button class="tab__close" title="Fermer" @click.stop="connections.close(tab.id)">
          <i class="pi pi-times" />
        </button>
      </div>
    </div>

    <div class="workspace__body">
      <!-- Toutes les vues restent montées (PTY vivants) ; seule l'active est visible. -->
      <TerminalView
        v-for="tab in tabs"
        v-show="tab.id === activeId"
        :key="tab.id"
        :args="tab.args"
        class="workspace__view"
      />

      <div v-if="tabs.length === 0" class="workspace__empty">
        <i class="pi pi-desktop" />
        <p>Aucune session ouverte.</p>
        <button class="link" @click="router.push({ name: 'servers' })">
          Ouvrir un serveur
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 0.6rem;
}

.tabs {
  display: flex;
  gap: 0.35rem;
  overflow-x: auto;
  padding-bottom: 0.15rem;
}

.tab {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  padding: 0.45rem 0.7rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 9px;
  background: var(--p-content-background);
  color: var(--p-text-muted-color);
  cursor: pointer;
  white-space: nowrap;
  transition:
    background-color 0.15s ease,
    color 0.15s ease,
    border-color 0.15s ease;
}

.tab:hover {
  background: var(--p-content-hover-background);
}

.tab--active {
  border-color: var(--p-primary-color);
  color: var(--p-text-color);
}

.tab__label {
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 0.88rem;
  font-weight: 500;
}

.tab__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: inherit;
  cursor: pointer;
  font-size: 0.7rem;
}

.tab__close:hover {
  background: color-mix(in srgb, #ef4444 25%, transparent);
  color: #ef4444;
}

.workspace__body {
  position: relative;
  flex: 1;
  min-height: 0;
}

.workspace__view {
  position: absolute;
  inset: 0;
}

.workspace__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  height: 100%;
  color: var(--p-text-muted-color);
}

.workspace__empty .pi {
  font-size: 2.5rem;
  opacity: 0.5;
}

.link {
  border: 0;
  background: none;
  color: var(--p-primary-color);
  font: inherit;
  cursor: pointer;
  text-decoration: underline;
}
</style>
