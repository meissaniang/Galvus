<script setup lang="ts">
import { onMounted, ref } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useSettingsStore } from "@/stores/settings";
import { checkForUpdate, dismiss, type UpdateInfo } from "@/services/updateCheck";

/**
 * Bandeau signalant qu'une version plus récente est publiée.
 *
 * Discret et écartable : il ne réapparaît plus pour la version refusée. La
 * vérification est désactivable dans Paramètres › Avancé.
 */
const settings = useSettingsStore();
const update = ref<UpdateInfo | null>(null);

onMounted(async () => {
  if (!settings.updateCheck) return;
  update.value = await checkForUpdate();
});

function close(): void {
  if (update.value) dismiss(update.value.version);
  update.value = null;
}

async function openRelease(): Promise<void> {
  if (update.value) await openUrl(update.value.url);
}
</script>

<template>
  <Transition name="banner">
    <div v-if="update" class="banner">
      <span class="banner__dot" />
      <span class="banner__text">
        <strong>Galvus {{ update.version }}</strong> est disponible.
      </span>
      <button class="banner__link" @click="openRelease">Voir les nouveautés</button>
      <button class="banner__close" title="Ignorer cette version" @click="close">
        <svg width="11" height="11" viewBox="0 0 12 12" fill="none">
          <path
            d="M3 3l6 6M9 3l-6 6"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.banner {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  background: var(--g-accent-soft);
  border-bottom: 1px solid var(--g-accent-ring);
  font-size: 12.5px;
  color: var(--g-t1);
  flex-shrink: 0;
}

.banner-enter-active {
  transition:
    opacity 0.18s ease-out,
    transform 0.18s cubic-bezier(0.2, 0.8, 0.3, 1);
}

.banner-enter-from {
  opacity: 0;
  transform: translateY(-6px);
}

.banner__dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: var(--g-accent);
  flex-shrink: 0;
}

.banner__text {
  flex: 1;
}

.banner__link {
  border: 0;
  background: transparent;
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 600;
  color: var(--g-accent);
  cursor: pointer;
  text-decoration: underline;
  padding: 0;
}

.banner__close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--g-t3);
  cursor: pointer;
  flex-shrink: 0;
}

.banner__close:hover {
  color: var(--g-t1);
  background: var(--g-s2);
}
</style>
