<script setup lang="ts">
import { computed } from "vue";
import type { SshKey } from "@/types/ssh";

const props = defineProps<{ keyItem: SshKey }>();
const emit = defineEmits<{ remove: [key: SshKey] }>();

const typeLabel = computed(() => {
  const t = props.keyItem.keyType ?? "?";
  return props.keyItem.bits ? `${t} · ${props.keyItem.bits}` : t;
});

/** Empreinte raccourcie pour l'affichage. */
const shortFingerprint = computed(() => {
  const fp = props.keyItem.fingerprint;
  if (!fp) return "—";
  return fp.length > 24 ? `${fp.slice(0, 24)}…` : fp;
});
</script>

<template>
  <article class="key-card">
    <div class="key-card__icon">
      <i class="pi pi-key" />
    </div>
    <button class="key-card__delete" title="Supprimer" @click="emit('remove', keyItem)">
      <i class="pi pi-trash" />
    </button>
    <div class="key-card__body">
      <div class="key-card__head">
        <h3 class="key-card__title" :title="keyItem.name">{{ keyItem.name }}</h3>
        <span class="key-card__type">{{ typeLabel }}</span>
      </div>
      <p class="key-card__fingerprint" :title="keyItem.fingerprint ?? ''">
        {{ shortFingerprint }}
      </p>
      <div class="key-card__meta">
        <span
          class="key-card__badge"
          :class="keyItem.hasPrivate ? 'key-card__badge--ok' : 'key-card__badge--warn'"
        >
          <i :class="keyItem.hasPrivate ? 'pi pi-lock' : 'pi pi-lock-open'" />
          {{ keyItem.hasPrivate ? "privée + publique" : "publique seule" }}
        </span>
        <span v-if="keyItem.comment" class="key-card__comment" :title="keyItem.comment">
          {{ keyItem.comment }}
        </span>
      </div>
    </div>
  </article>
</template>

<style scoped>
.key-card {
  display: flex;
  align-items: flex-start;
  gap: 0.85rem;
  padding: 0.9rem 1rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 12px;
  background: var(--p-content-background);
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}

.key-card {
  position: relative;
}

.key-card:hover {
  border-color: var(--p-primary-color);
  box-shadow: 0 6px 18px rgb(0 0 0 / 0.18);
}

.key-card__delete {
  position: absolute;
  top: 8px;
  right: 8px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  color: var(--p-text-muted-color);
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s ease, background-color 0.15s ease, color 0.15s ease;
}

.key-card:hover .key-card__delete {
  opacity: 1;
}

.key-card__delete:hover {
  background: color-mix(in srgb, #ef4444 18%, transparent);
  color: #ef4444;
}

.key-card__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  flex-shrink: 0;
  border-radius: 10px;
  background: var(--p-content-hover-background);
  color: var(--p-primary-color);
  font-size: 1.2rem;
}

.key-card__body {
  min-width: 0;
  flex: 1;
}

.key-card__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.5rem;
}

.key-card__title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.key-card__type {
  flex-shrink: 0;
  padding: 0.1rem 0.45rem;
  border-radius: 6px;
  background: var(--p-highlight-background);
  color: var(--p-highlight-color);
  font-size: 0.72rem;
  font-weight: 600;
}

.key-card__fingerprint {
  margin: 0.3rem 0 0;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 0.78rem;
  color: var(--p-text-muted-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.key-card__meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.55rem;
  flex-wrap: wrap;
}

.key-card__badge {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.1rem 0.45rem;
  border-radius: 6px;
  font-size: 0.72rem;
}

.key-card__badge--ok {
  background: color-mix(in srgb, var(--p-primary-color) 18%, transparent);
  color: var(--p-primary-color);
}

.key-card__badge--warn {
  background: var(--p-content-hover-background);
  color: var(--p-text-muted-color);
}

.key-card__comment {
  font-size: 0.75rem;
  color: var(--p-text-muted-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
