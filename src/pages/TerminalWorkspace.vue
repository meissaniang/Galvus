<script setup lang="ts">
import { useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { useConnectionsStore } from "@/stores/connections";
import TerminalView from "@/components/TerminalView.vue";

const connections = useConnectionsStore();
const { tabs, activeTabId } = storeToRefs(connections);
const router = useRouter();
</script>

<template>
  <section class="workspace">
    <div v-if="tabs.length > 0" class="bar">
      <div class="tabs">
        <div
          v-for="tab in tabs"
          :key="tab.id"
          class="tab"
          :class="{ 'tab--active': tab.id === activeTabId }"
          @click="connections.setActiveTab(tab.id)"
        >
          <i class="pi pi-desktop" />
          <span class="tab__label" :title="connections.tabTitle(tab)">
            {{ connections.tabTitle(tab) }}
          </span>
          <button class="tab__close" title="Fermer l'onglet" @click.stop="connections.closeTab(tab.id)">
            <i class="pi pi-times" />
          </button>
        </div>
      </div>

      <div class="split-actions">
        <button title="Split vertical (côte à côte)" @click="connections.splitActive('row')">
          <i class="pi pi-arrows-h" />
        </button>
        <button title="Split horizontal (empilé)" @click="connections.splitActive('column')">
          <i class="pi pi-arrows-v" />
        </button>
      </div>
    </div>

    <div class="workspace__body">
      <!-- Tous les onglets et panes restent montés (PTY vivants) ; seul l'onglet actif est visible. -->
      <div
        v-for="tab in tabs"
        v-show="tab.id === activeTabId"
        :key="tab.id"
        class="panes"
        :class="`panes--${tab.direction}`"
      >
        <div
          v-for="pane in tab.panes"
          :key="pane.id"
          class="pane"
          :class="{ 'pane--active': tab.panes.length > 1 && pane.id === tab.activePaneId }"
          @mousedown="connections.setActivePane(tab.id, pane.id)"
        >
          <button
            v-if="tab.panes.length > 1"
            class="pane__close"
            title="Fermer ce pane"
            @click="connections.closePane(tab.id, pane.id)"
          >
            <i class="pi pi-times" />
          </button>
          <TerminalView :args="pane.args" class="pane__view" />
        </div>
      </div>

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
  padding: 12px 16px 16px;
  box-sizing: border-box;
}

.bar {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.tabs {
  display: flex;
  gap: 0.35rem;
  overflow-x: auto;
  flex: 1;
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

.split-actions {
  display: flex;
  gap: 0.3rem;
  flex-shrink: 0;
}

.split-actions button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border: 1px solid var(--p-content-border-color);
  border-radius: 8px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  cursor: pointer;
}

.split-actions button:hover {
  background: var(--p-content-hover-background);
}

.workspace__body {
  position: relative;
  flex: 1;
  min-height: 0;
}

.panes {
  position: absolute;
  inset: 0;
  display: flex;
  gap: 0.5rem;
}

.panes--row {
  flex-direction: row;
}

.panes--column {
  flex-direction: column;
}

.pane {
  position: relative;
  flex: 1;
  min-width: 0;
  min-height: 0;
  border: 1px solid transparent;
  border-radius: 10px;
}

.pane--active {
  border-color: color-mix(in srgb, var(--p-primary-color) 55%, transparent);
}

.pane__view {
  width: 100%;
  height: 100%;
}

.pane__close {
  position: absolute;
  top: 6px;
  right: 6px;
  z-index: 2;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: 0;
  border-radius: 6px;
  background: rgb(0 0 0 / 0.45);
  color: #fff;
  cursor: pointer;
  font-size: 0.72rem;
}

.pane__close:hover {
  background: #ef4444;
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
