<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";

/**
 * Shell applicatif : sidebar de navigation + zone de contenu routée.
 * Les entrées de menu sont dérivées des routes déclarant un `meta.title`.
 */
const router = useRouter();

interface NavItem {
  name: string;
  title: string;
  icon: string;
}

const navItems = computed<NavItem[]>(() =>
  router
    .getRoutes()
    .filter((r) => r.meta?.title && r.name)
    .map((r) => ({
      name: String(r.name),
      title: String(r.meta.title),
      icon: String(r.meta.icon ?? "pi pi-circle"),
    })),
);
</script>

<template>
  <div class="app-shell">
    <aside class="app-sidebar">
      <div class="app-brand">
        <i class="pi pi-bolt" />
        <span class="app-brand__name">Galvus</span>
      </div>

      <nav class="app-nav">
        <router-link
          v-for="item in navItems"
          :key="item.name"
          :to="{ name: item.name }"
          class="app-nav__link"
          active-class="app-nav__link--active"
        >
          <i :class="item.icon" />
          <span>{{ item.title }}</span>
        </router-link>
      </nav>
    </aside>

    <main class="app-content">
      <router-view />
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  color: var(--p-text-color);
  background: var(--p-content-background);
}

.app-sidebar {
  display: flex;
  flex-direction: column;
  width: 240px;
  flex-shrink: 0;
  padding: 1rem 0.75rem;
  gap: 1.25rem;
  border-right: 1px solid var(--p-content-border-color);
  background: var(--p-surface-50);
}

:global(.app-dark) .app-sidebar {
  background: var(--p-surface-900);
}

.app-brand {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.5rem 0.6rem;
  font-size: 1.15rem;
  font-weight: 700;
}

.app-brand .pi {
  color: var(--p-primary-color);
  font-size: 1.3rem;
}

.app-nav {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.app-nav__link {
  display: flex;
  align-items: center;
  gap: 0.7rem;
  padding: 0.6rem 0.75rem;
  border-radius: var(--p-border-radius-md, 8px);
  color: var(--p-text-muted-color);
  text-decoration: none;
  font-weight: 500;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.app-nav__link:hover {
  background: var(--p-content-hover-background);
  color: var(--p-text-color);
}

.app-nav__link--active {
  background: var(--p-highlight-background);
  color: var(--p-highlight-color);
}

.app-content {
  flex: 1;
  overflow: auto;
  padding: 1.5rem 2rem;
}
</style>
