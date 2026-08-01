<script setup lang="ts">
import { computed } from "vue";
import type { Host } from "@/types/ssh";

const props = defineProps<{ host: Host }>();

/** Couleur d'accent stable dérivée de l'alias (teinte HSL déterministe). */
const accent = computed(() => {
  let hue = 0;
  for (const ch of props.host.alias) {
    hue = (hue * 31 + ch.charCodeAt(0)) % 360;
  }
  return `hsl(${hue} 65% 48%)`;
});

const initials = computed(() =>
  props.host.alias.replace(/[^a-zA-Z0-9]/g, "").slice(0, 2).toUpperCase(),
);

const subtitle = computed(() => {
  const parts: string[] = [];
  if (props.host.user) parts.push(props.host.user);
  if (props.host.hostname) parts.push(props.host.hostname);
  return parts.length ? parts.join("@") : "ssh";
});
</script>

<template>
  <article class="host-card">
    <div class="host-card__icon" :style="{ background: accent }">
      <span>{{ initials }}</span>
    </div>
    <div class="host-card__body">
      <h3 class="host-card__title" :title="host.alias">{{ host.alias }}</h3>
      <p class="host-card__subtitle" :title="subtitle">{{ subtitle }}</p>
      <div class="host-card__meta">
        <span v-if="host.port && host.port !== 22" class="host-card__badge">
          <i class="pi pi-sign-in" /> {{ host.port }}
        </span>
        <span v-if="host.proxyJump" class="host-card__badge" :title="host.proxyJump">
          <i class="pi pi-share-alt" /> jump
        </span>
        <span v-if="host.identityFile" class="host-card__badge" title="Clé dédiée">
          <i class="pi pi-key" />
        </span>
      </div>
    </div>
  </article>
</template>

<style scoped>
.host-card {
  display: flex;
  align-items: center;
  gap: 0.85rem;
  padding: 0.9rem 1rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 12px;
  background: var(--p-content-background);
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    transform 0.15s ease,
    box-shadow 0.15s ease;
}

.host-card:hover {
  border-color: var(--p-primary-color);
  transform: translateY(-2px);
  box-shadow: 0 6px 18px rgb(0 0 0 / 0.18);
}

.host-card__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  flex-shrink: 0;
  border-radius: 10px;
  color: #fff;
  font-weight: 700;
  font-size: 0.95rem;
  letter-spacing: 0.02em;
}

.host-card__body {
  min-width: 0;
  flex: 1;
}

.host-card__title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.host-card__subtitle {
  margin: 0.15rem 0 0;
  font-size: 0.85rem;
  color: var(--p-text-muted-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.host-card__meta {
  display: flex;
  gap: 0.4rem;
  margin-top: 0.5rem;
  flex-wrap: wrap;
}

.host-card__badge {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.1rem 0.45rem;
  border-radius: 6px;
  background: var(--p-content-hover-background);
  color: var(--p-text-muted-color);
  font-size: 0.72rem;
}
</style>
