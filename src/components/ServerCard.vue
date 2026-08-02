<script setup lang="ts">
import { computed } from "vue";
import type { Server } from "@/types/ssh";

/**
 * Tuile serveur — fidèle à « ScreenServers.dc.html » :
 * pastille 40px (radius 11), nom + point de connexion, adresse mono,
 * tags en pilules ; au survol : surface-2 + anneau accent + translateY(-1px)
 * et les actions (Connecter / éditer / supprimer) remplacent les tags.
 */
const props = defineProps<{ server: Server; connected?: boolean }>();
const emit = defineEmits<{
  connect: [server: Server];
  edit: [server: Server];
  remove: [server: Server];
  toggleFavorite: [server: Server];
}>();

/** Couleurs de pastille claires → texte encre foncée (règle AA du DS). */
const LIGHT_TILES: Record<string, string> = {
  "#23c48a": "#052018",
  "#2fc2ae": "#04231f",
  "#22b8d9": "#04222a",
  "#e9be3b": "#241a04",
  "#8bcb4a": "#12200a",
};

const accent = computed(() => {
  if (props.server.color) return props.server.color;
  let hue = 0;
  for (const ch of props.server.name) {
    hue = (hue * 31 + ch.charCodeAt(0)) % 360;
  }
  return `hsl(${hue} 65% 48%)`;
});

const tileFg = computed(() => LIGHT_TILES[accent.value.toLowerCase()] ?? "#ffffff");

const initials = computed(() =>
  props.server.name
    .replace(/[^a-zA-Z0-9]/g, "")
    .slice(0, 2)
    .toUpperCase(),
);

const address = computed(() => {
  const target = props.server.username
    ? `${props.server.username}@${props.server.hostname}`
    : props.server.hostname;
  return `${target}:${props.server.port}`;
});
</script>

<template>
  <article class="tile" @dblclick="emit('connect', server)">
    <div class="tile__ava" :style="{ background: accent, color: tileFg }">
      {{ initials }}
    </div>

    <div class="tile__body">
      <div class="tile__head">
        <span class="tile__name">{{ server.name }}</span>
        <span v-if="connected" class="tile__on" title="Session ouverte" />
      </div>
      <div class="tile__addr" :title="address">{{ address }}</div>

      <div class="tile__meta">
        <div class="tile__tags">
          <span v-for="tag in server.tags" :key="tag" class="tile__tag">{{ tag }}</span>
          <span v-if="server.tags.length === 0" class="tile__notags">—</span>
        </div>
        <div class="tile__actions">
          <button class="tile__connect" @click.stop="emit('connect', server)">
            Connecter
          </button>
          <button class="tile__icon" title="Éditer" @click.stop="emit('edit', server)">
            <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
              <path
                d="M9.4 2.4l2.2 2.2-6.4 6.4-2.8.6.6-2.8z"
                stroke="currentColor"
                stroke-width="1.4"
                stroke-linejoin="round"
              />
            </svg>
          </button>
          <button
            class="tile__icon tile__icon--danger"
            title="Supprimer"
            @click.stop="emit('remove', server)"
          >
            <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
              <path
                d="M2.6 4.4h8.8M5.4 4.4V3.2h3.2v1.2M4 4.4l.6 6.6h4.8L10 4.4"
                stroke="currentColor"
                stroke-width="1.3"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <button
      class="tile__star"
      :class="{ 'tile__star--on': server.favorite }"
      :title="server.favorite ? 'Retirer des favoris' : 'Ajouter aux favoris'"
      @click.stop="emit('toggleFavorite', server)"
    >
      <svg
        width="15"
        height="15"
        viewBox="0 0 16 16"
        :fill="server.favorite ? 'var(--g-warning)' : 'none'"
      >
        <path
          d="M8 1.6l1.9 3.9 4.3.6-3.1 3 .74 4.3L8 11.4l-3.83 2 .74-4.3-3.1-3 4.3-.6z"
          :stroke="server.favorite ? 'var(--g-warning)' : 'currentColor'"
          stroke-width="1.2"
          stroke-linejoin="round"
        />
      </svg>
    </button>
  </article>
</template>

<style scoped>
.tile {
  position: relative;
  display: flex;
  gap: 11px;
  padding: 13px;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  border-radius: 12px;
  box-shadow: var(--g-sh1);
  cursor: pointer;
  transition:
    transform 0.14s cubic-bezier(0.2, 0.8, 0.3, 1),
    box-shadow 0.14s ease,
    background 0.14s ease,
    border-color 0.14s ease;
}

.tile:hover {
  background: var(--g-s2);
  border-color: var(--g-accent-ring);
  transform: translateY(-1px);
  box-shadow: var(--g-sh2);
}

.tile__ava {
  width: 40px;
  height: 40px;
  border-radius: 11px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13.5px;
  font-weight: 700;
  flex-shrink: 0;
}

.tile__body {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.tile__head {
  display: flex;
  align-items: center;
  gap: 6px;
}

.tile__name {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--g-t1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tile__on {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--g-success);
  flex-shrink: 0;
}

.tile__addr {
  font-family: var(--g-font-mono);
  font-size: 11.5px;
  color: var(--g-t2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Zone tags / actions superposées : les actions remplacent les tags au survol. */
.tile__meta {
  position: relative;
  height: 26px;
  margin-top: 2px;
}

.tile__tags,
.tile__actions {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  gap: 5px;
  transition:
    opacity 0.12s ease-out,
    transform 0.12s ease-out;
}

.tile__tags {
  flex-wrap: wrap;
  overflow: hidden;
}

.tile__actions {
  opacity: 0;
  transform: translateY(2px);
  pointer-events: none;
  transition-delay: 0.04s;
}

.tile:hover .tile__tags {
  opacity: 0;
}

.tile:hover .tile__actions {
  opacity: 1;
  transform: translateY(0);
  pointer-events: auto;
}

.tile__tag {
  font-size: 10.5px;
  font-weight: 500;
  color: var(--g-t2);
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  padding: 2px 7px;
  border-radius: 999px;
  white-space: nowrap;
}

.tile:hover .tile__tag {
  background: var(--g-s1);
}

.tile__notags {
  font-size: 10.5px;
  color: var(--g-t3);
}

.tile__connect {
  display: flex;
  align-items: center;
  height: 26px;
  padding: 0 11px;
  border: 0;
  border-radius: 8px;
  background: var(--g-accent);
  color: var(--g-accent-fg);
  font-family: inherit;
  font-size: 11.5px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.12s linear;
}

.tile__connect:hover {
  background: var(--g-accent-h);
}

.tile__connect:active {
  transform: scale(0.985);
}

.tile__icon {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  color: var(--g-t2);
  cursor: pointer;
  transition:
    background 0.12s ease,
    color 0.12s ease,
    border-color 0.12s ease;
}

.tile__icon:hover {
  background: var(--g-s2);
  color: var(--g-t1);
}

.tile__icon--danger:hover {
  background: var(--g-danger-soft);
  border-color: var(--g-danger);
  color: var(--g-danger);
}

.tile__star {
  flex-shrink: 0;
  align-self: flex-start;
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 0;
  background: transparent;
  color: var(--g-t3);
  cursor: pointer;
  padding: 0;
  transition: color 0.12s ease;
}

.tile__star:hover {
  color: var(--g-warning);
}
</style>
