<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { useThemeStore } from "@/stores/theme";

/**
 * Shell applicatif : sidebar de navigation + zone de contenu routée.
 * Les entrées de menu sont dérivées des routes déclarant un `meta.title`.
 */
const router = useRouter();
const theme = useThemeStore();

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

      <button
        type="button"
        class="app-theme-toggle"
        :title="theme.isDark ? 'Passer en clair' : 'Passer en sombre'"
        @click="theme.toggle()"
      >
        <i :class="theme.isDark ? 'pi pi-sun' : 'pi pi-moon'" />
        <span>{{ theme.isDark ? "Mode clair" : "Mode sombre" }}</span>
      </button>
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
  font-size: 1.3rem;
  font-weight: 800;
  letter-spacing: -0.01em;
  color: var(--p-text-color);
}

.app-brand .pi {
  color: var(--p-primary-color);
  font-size: 1.4rem;
}

.app-nav {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.app-nav__link {
  position: relative;
  display: flex;
  align-items: center;
  gap: 0.7rem;
  padding: 0.6rem 0.8rem;
  border-radius: var(--p-border-radius-md, 8px);
  /* Contraste renforcé : lisible sur fond sombre comme clair. */
  color: color-mix(in srgb, var(--p-text-color) 78%, transparent);
  text-decoration: none;
  font-weight: 500;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.app-nav__link .pi {
  font-size: 1.05rem;
}

.app-nav__link:hover {
  background: var(--p-content-hover-background);
  color: var(--p-text-color);
}

.app-nav__link--active {
  background: color-mix(in srgb, var(--p-primary-color) 16%, transparent);
  color: var(--p-primary-color);
  font-weight: 600;
}

/* Barre d'accent à gauche de l'item actif. */
.app-nav__link--active::before {
  content: "";
  position: absolute;
  left: 0;
  top: 20%;
  bottom: 20%;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background: var(--p-primary-color);
}

.app-theme-toggle {
  display: flex;
  align-items: center;
  gap: 0.7rem;
  margin-top: auto;
  padding: 0.6rem 0.75rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: var(--p-border-radius-md, 8px);
  background: transparent;
  color: color-mix(in srgb, var(--p-text-color) 78%, transparent);
  font: inherit;
  font-weight: 500;
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.app-theme-toggle:hover {
  background: var(--p-content-hover-background);
  color: var(--p-text-color);
}

.app-content {
  flex: 1;
  overflow: auto;
  padding: 1.5rem 2rem;
}
</style>
