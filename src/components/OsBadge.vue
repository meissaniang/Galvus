<script setup lang="ts">
import { computed } from "vue";
import { osInfo } from "@/utils/osIcons";

/**
 * Pastille d'un serveur : le logo du système quand il est connu, les initiales
 * du nom sinon.
 *
 * Les initiales restent le repli plutôt qu'un logo générique : tant que le
 * système n'a pas été reconnu, mieux vaut une information exacte — le nom —
 * qu'un pingouin qui laisserait croire à une détection.
 */
const props = withDefaults(
  defineProps<{
    os?: string | null;
    name: string;
    /** Couleur de fond quand elle est imposée (couleur choisie par l'utilisateur). */
    color?: string | null;
    /** Encre posée sur cette couleur — les pastilles claires exigent du foncé. */
    fg?: string;
    size?: number;
  }>(),
  { os: null, color: null, fg: "#ffffff", size: 34 },
);

const info = computed(() => osInfo(props.os));

const initials = computed(() =>
  props.name
    .replace(/[^a-zA-Z0-9]/g, "")
    .slice(0, 2)
    .toUpperCase(),
);

/**
 * Sans couleur imposée, le logo s'affiche dans la teinte de la marque sur fond
 * neutre — bien plus reconnaissable qu'un aplat coloré. Avec une couleur
 * choisie, elle prime et le logo passe en blanc.
 */
const branded = computed(() => info.value !== null && !props.color);
</script>

<template>
  <div
    class="osb"
    :class="{ 'osb--branded': branded }"
    :style="{
      width: `${size}px`,
      height: `${size}px`,
      background: branded ? undefined : (color ?? undefined),
      color: branded ? undefined : fg,
    }"
    :title="info?.label ?? undefined"
  >
    <svg
      v-if="info"
      :width="size * 0.58"
      :height="size * 0.58"
      viewBox="0 0 24 24"
      :fill="branded ? info.color : 'currentColor'"
      aria-hidden="true"
    >
      <path :d="info.path" />
    </svg>
    <span v-else class="osb__initials" :style="{ fontSize: `${size * 0.36}px` }">
      {{ initials }}
    </span>
  </div>
</template>

<style scoped>
.osb {
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  flex-shrink: 0;
  color: #ffffff;
  font-weight: 700;
  letter-spacing: 0.3px;
}

/* Fond neutre : le logo porte lui-même la couleur de la marque. */
.osb--branded {
  background: var(--g-s2);
  border: 1px solid var(--g-border);
}

.osb__initials {
  line-height: 1;
}
</style>
