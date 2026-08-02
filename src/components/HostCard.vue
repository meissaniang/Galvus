<script setup lang="ts">
import { computed } from "vue";
import type { Host } from "@/types/ssh";

/**
 * Carte d'hôte importé de ~/.ssh/config — fidèle au DS : bordure pointillée,
 * pastille mono 32px sur surface-2, adresse mono. Lecture seule, clic = connexion.
 */
const props = defineProps<{ host: Host; connected?: boolean }>();
const emit = defineEmits<{ edit: [host: Host]; remove: [host: Host] }>();

const abbr = computed(() =>
  props.host.alias.replace(/[^a-zA-Z0-9]/g, "").slice(0, 3).toLowerCase(),
);

const address = computed(() => {
  const target = props.host.user
    ? `${props.host.user}@${props.host.hostname ?? props.host.alias}`
    : (props.host.hostname ?? props.host.alias);
  return `${target}:${props.host.port ?? 22}`;
});
</script>

<template>
  <article class="hostcard" :title="`ssh ${host.alias}`">
    <div class="hostcard__ava">{{ abbr }}</div>
    <div class="hostcard__body">
      <div class="hostcard__head">
        <span class="hostcard__name">{{ host.alias }}</span>
        <span v-if="connected" class="hostcard__on" title="Session ouverte" />
        <span v-if="host.proxyJump" class="hostcard__jump" :title="`ProxyJump ${host.proxyJump}`">jump</span>
      </div>
      <div class="hostcard__addr">{{ address }}</div>
    </div>
    <div class="hostcard__actions">
      <button class="hostcard__icon" title="Éditer dans ~/.ssh/config" @click.stop="emit('edit', host)">
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
          <path d="M9.4 2.4l2.2 2.2-6.4 6.4-2.8.6.6-2.8z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round" />
        </svg>
      </button>
      <button
        class="hostcard__icon hostcard__icon--danger"
        title="Supprimer du ~/.ssh/config"
        @click.stop="emit('remove', host)"
      >
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
          <path d="M2.6 4.4h8.8M5.4 4.4V3.2h3.2v1.2M4 4.4l.6 6.6h4.8L10 4.4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </article>
</template>

<style scoped>
.hostcard {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 11px 13px;
  background: transparent;
  border: 1px dashed var(--g-border);
  border-radius: 12px;
  cursor: pointer;
  transition: border-color 0.14s ease, background 0.14s ease;
}

.hostcard:hover {
  border-color: var(--g-accent-ring);
  background: var(--g-s1);
}

.hostcard__ava {
  width: 32px;
  height: 32px;
  border-radius: 9px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--g-font-mono);
  font-size: 11px;
  font-weight: 700;
  color: var(--g-t2);
  flex-shrink: 0;
}

.hostcard__body {
  min-width: 0;
  flex: 1;
}

.hostcard__actions {
  display: flex;
  gap: 5px;
  opacity: 0;
  transition: opacity 0.12s ease-out;
}

.hostcard:hover .hostcard__actions {
  opacity: 1;
}

.hostcard__icon {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  color: var(--g-t2);
  cursor: pointer;
  transition: background 0.12s ease, color 0.12s ease, border-color 0.12s ease;
}

.hostcard__icon:hover {
  color: var(--g-t1);
  background: var(--g-s3);
}

.hostcard__icon--danger:hover {
  background: var(--g-danger-soft);
  border-color: var(--g-danger);
  color: var(--g-danger);
}

.hostcard__head {
  display: flex;
  align-items: center;
  gap: 6px;
}

.hostcard__name {
  font-size: 13px;
  font-weight: 600;
  color: var(--g-t1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hostcard__on {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--g-success);
  flex-shrink: 0;
}

.hostcard__jump {
  font-family: var(--g-font-mono);
  font-size: 9.5px;
  color: var(--g-t3);
  border: 1px solid var(--g-border);
  padding: 0 5px;
  border-radius: 5px;
}

.hostcard__addr {
  font-family: var(--g-font-mono);
  font-size: 11px;
  color: var(--g-t3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
