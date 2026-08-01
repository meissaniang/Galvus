<script setup lang="ts">
import { computed } from "vue";
import type { Server } from "@/types/ssh";

const props = defineProps<{ server: Server }>();
const emit = defineEmits<{
  connect: [server: Server];
  edit: [server: Server];
  remove: [server: Server];
}>();

/** Couleur d'accent : celle choisie, sinon dérivée du nom. */
const accent = computed(() => {
  if (props.server.color) return props.server.color;
  let hue = 0;
  for (const ch of props.server.name) {
    hue = (hue * 31 + ch.charCodeAt(0)) % 360;
  }
  return `hsl(${hue} 65% 48%)`;
});

const initials = computed(() =>
  props.server.name.replace(/[^a-zA-Z0-9]/g, "").slice(0, 2).toUpperCase(),
);

const subtitle = computed(() => {
  const target = props.server.username
    ? `${props.server.username}@${props.server.hostname}`
    : props.server.hostname;
  return props.server.port !== 22 ? `${target}:${props.server.port}` : target;
});
</script>

<template>
  <article class="server-card" @dblclick="emit('connect', server)">
    <div class="server-card__top">
      <div class="server-card__icon" :style="{ background: accent }">
        <span>{{ initials }}</span>
      </div>
      <div class="server-card__body">
        <h3 class="server-card__title" :title="server.name">
          <i v-if="server.favorite" class="pi pi-star-fill server-card__star" />
          {{ server.name }}
        </h3>
        <p class="server-card__subtitle" :title="subtitle">{{ subtitle }}</p>
      </div>
    </div>

    <div class="server-card__actions">
      <button class="act act--primary" title="Se connecter" @click="emit('connect', server)">
        <i class="pi pi-play" /> Connecter
      </button>
      <button class="act" title="Éditer" @click="emit('edit', server)">
        <i class="pi pi-pencil" />
      </button>
      <button class="act act--danger" title="Supprimer" @click="emit('remove', server)">
        <i class="pi pi-trash" />
      </button>
    </div>
  </article>
</template>

<style scoped>
.server-card {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.9rem 1rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 12px;
  background: var(--p-content-background);
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}

.server-card:hover {
  border-color: var(--p-primary-color);
  box-shadow: 0 6px 18px rgb(0 0 0 / 0.18);
}

.server-card__top {
  display: flex;
  align-items: center;
  gap: 0.85rem;
}

.server-card__icon {
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
}

.server-card__body {
  min-width: 0;
  flex: 1;
}

.server-card__title {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.server-card__star {
  color: #f5b301;
  font-size: 0.8rem;
}

.server-card__subtitle {
  margin: 0.15rem 0 0;
  font-size: 0.85rem;
  color: var(--p-text-muted-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.server-card__actions {
  display: flex;
  gap: 0.4rem;
}

.act {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.35rem;
  padding: 0.4rem 0.6rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 8px;
  background: transparent;
  color: var(--p-text-color);
  font: inherit;
  font-size: 0.82rem;
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    border-color 0.15s ease,
    color 0.15s ease;
}

.act:hover {
  background: var(--p-content-hover-background);
}

.act--primary {
  flex: 1;
  border-color: color-mix(in srgb, var(--p-primary-color) 40%, transparent);
  color: var(--p-primary-color);
}

.act--primary:hover {
  background: color-mix(in srgb, var(--p-primary-color) 14%, transparent);
}

.act--danger:hover {
  border-color: color-mix(in srgb, #ef4444 50%, transparent);
  color: #ef4444;
}
</style>
