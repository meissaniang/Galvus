<script setup lang="ts">
import { computed, ref } from "vue";
import type { SshKey } from "@/types/ssh";

/**
 * Carte de clé SSH — fidèle à « ScreenKeys.dc.html » : pastille 38px
 * (accent-soft pour ED25519), badge de type, encart empreinte SHA256 copiable,
 * badges privée / publique, warning « algorithme déprécié » pour RSA ≤ 2048.
 */
const props = defineProps<{ keyItem: SshKey }>();
const emit = defineEmits<{
  remove: [key: SshKey];
  copyPublic: [key: SshKey];
  viewPrivate: [key: SshKey];
  fixPermissions: [key: SshKey];
}>();

const isEd25519 = computed(() => props.keyItem.keyType?.toUpperCase() === "ED25519");
const isDeprecated = computed(
  () =>
    props.keyItem.keyType?.toUpperCase() === "RSA" && (props.keyItem.bits ?? 0) <= 2048,
);

const typeLabel = computed(() => {
  const t = props.keyItem.keyType?.toUpperCase() ?? "?";
  return t === "RSA" ? `RSA ${props.keyItem.bits ?? ""}`.trim() : t;
});

const copied = ref(false);

async function copyFingerprint(): Promise<void> {
  if (!props.keyItem.fingerprint) return;
  await navigator.clipboard.writeText(props.keyItem.fingerprint);
  copied.value = true;
  setTimeout(() => (copied.value = false), 1500);
}
</script>

<template>
  <article class="kcard">
    <div class="kcard__head">
      <div class="kcard__ava" :class="{ 'kcard__ava--accent': isEd25519 }">
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
          <circle cx="6.2" cy="6.2" r="3.4" stroke="currentColor" stroke-width="1.6" />
          <path
            d="M8.7 8.7L14.5 14.5M12 13l1.6-1.6"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
          />
        </svg>
      </div>
      <div class="kcard__titles">
        <div class="kcard__name-row">
          <span class="kcard__name">{{ keyItem.name }}</span>
          <span
            class="kcard__type"
            :class="isDeprecated ? 'kcard__type--warn' : 'kcard__type--accent'"
            >{{ typeLabel }}</span
          >
          <span
            v-if="keyItem.encrypted"
            class="kcard__pass"
            title="Protégée par une passphrase"
          >
            passphrase
          </span>
        </div>
        <div v-if="isDeprecated" class="kcard__sub kcard__sub--warn">
          Algorithme déprécié — envisagez une rotation
        </div>
        <div v-else class="kcard__sub">{{ keyItem.comment || keyItem.path }}</div>
      </div>
      <div class="kcard__actions">
        <button
          class="kcard__icon"
          title="Modifier (contenu, passphrase)"
          @click="emit('viewPrivate', keyItem)"
        >
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
          class="kcard__icon"
          title="Copier la clé publique"
          @click="emit('copyPublic', keyItem)"
        >
          <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
            <rect
              x="2.4"
              y="2.4"
              width="6.4"
              height="6.4"
              rx="1.6"
              stroke="currentColor"
              stroke-width="1.3"
            />
            <path
              d="M5.2 11.6h6.4V5.2"
              stroke="currentColor"
              stroke-width="1.3"
              stroke-linecap="round"
            />
          </svg>
        </button>
        <button
          class="kcard__icon kcard__icon--danger"
          title="Supprimer"
          @click="emit('remove', keyItem)"
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

    <button class="kcard__fp" :title="keyItem.fingerprint ?? ''" @click="copyFingerprint">
      <span class="kcard__fp-text">{{ keyItem.fingerprint ?? "—" }}</span>
      <span class="kcard__fp-copy" :class="{ 'kcard__fp-copy--ok': copied }">
        {{ copied ? "copié ✓" : "copier" }}
      </span>
    </button>

    <div class="kcard__badges">
      <span v-if="keyItem.hasPrivate" class="kbadge kbadge--ok">
        <span class="kbadge__dot" />privée
      </span>
      <span v-else class="kbadge kbadge--missing">privée manquante</span>
      <span class="kbadge kbadge--info"><span class="kbadge__dot" />publique</span>
      <span
        v-if="keyItem.inAgent"
        class="kbadge kbadge--agent"
        title="Passphrase mémorisée dans l'agent SSH"
      >
        <span class="kbadge__dot" />agent
      </span>

      <template v-if="keyItem.insecurePermissions">
        <span class="kbadge kbadge--danger" title="ssh refusera d'utiliser cette clé">
          permissions trop ouvertes
        </span>
        <button class="kcard__fix" @click="emit('fixPermissions', keyItem)">
          corriger (600)
        </button>
      </template>
    </div>
  </article>
</template>

<style scoped>
.kcard {
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  border-radius: 12px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 11px;
  box-shadow: var(--g-sh1);
  transition:
    border-color 0.14s ease,
    box-shadow 0.14s ease;
}

.kcard:hover {
  border-color: var(--g-accent-ring);
  box-shadow: var(--g-sh2);
}

.kcard__head {
  display: flex;
  align-items: flex-start;
  gap: 11px;
}

.kcard__ava {
  width: 38px;
  height: 38px;
  border-radius: 11px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  color: var(--g-t2);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.kcard__ava--accent {
  background: var(--g-accent-soft);
  border-color: transparent;
  color: var(--g-accent);
}

.kcard__titles {
  flex: 1;
  min-width: 0;
}

.kcard__name-row {
  display: flex;
  align-items: center;
  gap: 7px;
  flex-wrap: wrap;
}

.kcard__name {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--g-t1);
}

.kcard__type {
  font-family: var(--g-font-mono);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.03em;
  padding: 2px 6px;
  border-radius: 5px;
}

.kcard__type--accent {
  color: var(--g-accent);
  background: var(--g-accent-soft);
}

.kcard__type--warn {
  color: var(--g-warning);
  background: var(--g-s2);
  border: 1px solid var(--g-border);
}

.kcard__pass {
  font-size: 10px;
  font-weight: 600;
  color: var(--g-t3);
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  padding: 2px 6px;
  border-radius: 5px;
}

.kcard__sub {
  font-size: 11.5px;
  color: var(--g-t3);
  margin-top: 3px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.kcard__sub--warn {
  color: var(--g-warning);
}

.kcard__actions {
  display: flex;
  gap: 5px;
}

.kcard__icon {
  width: 26px;
  height: 26px;
  border-radius: 8px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--g-t2);
  cursor: pointer;
  transition:
    background 0.12s ease,
    color 0.12s ease,
    border-color 0.12s ease;
}

.kcard__icon:hover {
  color: var(--g-t1);
  background: var(--g-s3);
}

.kcard__icon--danger:hover {
  background: var(--g-danger-soft);
  border-color: var(--g-danger);
  color: var(--g-danger);
}

.kcard__fp {
  background: var(--g-s0);
  border: 1px solid var(--g-border);
  border-radius: 9px;
  padding: 9px 11px;
  display: flex;
  align-items: center;
  gap: 9px;
  cursor: pointer;
  font-family: inherit;
  text-align: left;
  transition: border-color 0.12s ease;
}

.kcard__fp:hover {
  border-color: var(--g-border-2);
}

.kcard__fp-text {
  font-family: var(--g-font-mono);
  font-size: 11px;
  color: var(--g-t2);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.kcard__fp-copy {
  font-size: 10.5px;
  font-weight: 600;
  color: var(--g-t3);
  flex-shrink: 0;
}

.kcard__fp-copy--ok {
  color: var(--g-success);
}

.kcard__badges {
  display: flex;
  align-items: center;
  gap: 6px;
}

.kbadge {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 10.5px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 999px;
}

.kbadge__dot {
  width: 5px;
  height: 5px;
  border-radius: 999px;
  background: currentColor;
}

.kbadge--ok {
  color: var(--g-success);
  background: var(--g-accent-soft);
}

.kbadge--info {
  color: var(--g-info);
  background: var(--g-s2);
  border: 1px solid var(--g-border);
}

.kbadge--missing {
  color: var(--g-t3);
  background: var(--g-s2);
  border: 1px dashed var(--g-border-2);
}

.kbadge--agent {
  color: var(--g-accent);
  background: var(--g-accent-soft);
}

.kbadge--danger {
  color: var(--g-danger);
  background: var(--g-danger-soft);
  border: 1px solid var(--g-danger);
}

.kcard__fix {
  border: 0;
  background: transparent;
  font-family: inherit;
  font-size: 10.5px;
  font-weight: 600;
  color: var(--g-accent);
  cursor: pointer;
  text-decoration: underline;
  padding: 0;
}
</style>
