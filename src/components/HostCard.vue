<script setup lang="ts">
import { computed } from "vue";
import type { Host } from "@/types/ssh";

/**
 * Carte d'hôte importé de ~/.ssh/config — fidèle au DS : bordure pointillée,
 * pastille mono 32px sur surface-2, adresse mono. Lecture seule, clic = connexion.
 */
const props = defineProps<{ host: Host; connected?: boolean }>();

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
