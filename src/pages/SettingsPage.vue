<script setup lang="ts">
import { computed } from "vue";
import SelectButton from "primevue/selectbutton";
import { useThemeStore, type ThemeMode } from "@/stores/theme";
import { useSettingsStore } from "@/stores/settings";

const theme = useThemeStore();
const settings = useSettingsStore();

interface ThemeOption {
  label: string;
  value: ThemeMode;
}
const themeOptions: ThemeOption[] = [
  { label: "Système", value: "system" },
  { label: "Clair", value: "light" },
  { label: "Sombre", value: "dark" },
];

const selectedMode = computed<ThemeMode>({
  get: () => theme.mode,
  set: (value) => theme.setMode(value),
});

const fontFamilies = [
  { label: "Mono système", value: 'ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace' },
  { label: "Menlo", value: "Menlo, monospace" },
  { label: "Courier", value: '"Courier New", Courier, monospace' },
];
</script>

<template>
  <section class="page">
    <header class="page__header">
      <h1><i class="pi pi-cog" /> Paramètres</h1>
      <p>Apparence et terminal. La synchronisation arrivera au Livrable 6.</p>
    </header>

    <h2 class="section">Apparence</h2>
    <div class="setting">
      <div class="setting__label">
        <span class="setting__title">Thème</span>
        <span class="setting__hint">« Système » suit votre OS.</span>
      </div>
      <SelectButton
        v-model="selectedMode"
        :options="themeOptions"
        option-label="label"
        option-value="value"
        :allow-empty="false"
      />
    </div>

    <h2 class="section">Terminal</h2>
    <div class="setting">
      <div class="setting__label">
        <span class="setting__title">Taille de police</span>
        <span class="setting__hint">{{ settings.terminalFontSize }} px</span>
      </div>
      <input
        v-model.number="settings.terminalFontSize"
        type="range"
        min="10"
        max="22"
        step="1"
      />
    </div>

    <div class="setting">
      <div class="setting__label">
        <span class="setting__title">Police</span>
        <span class="setting__hint">Appliquée aux sessions ouvertes.</span>
      </div>
      <select v-model="settings.terminalFontFamily" class="select">
        <option v-for="f in fontFamilies" :key="f.label" :value="f.value">
          {{ f.label }}
        </option>
      </select>
    </div>

    <div class="setting setting--preview">
      <span
        class="preview"
        :style="{ fontFamily: settings.terminalFontFamily, fontSize: settings.terminalFontSize + 'px' }"
      >
        meissa@galvus:~$ echo "Aperçu du terminal 1234"
      </span>
    </div>

    <button class="reset" @click="settings.reset()">
      <i class="pi pi-undo" /> Réinitialiser le terminal
    </button>
  </section>
</template>

<style scoped>
.page__header h1 {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin: 0 0 0.35rem;
  font-size: 1.6rem;
}

.page__header p {
  margin: 0 0 1.5rem;
  color: var(--p-text-muted-color);
}

.section {
  margin: 1.5rem 0 0.75rem;
  font-size: 1rem;
  color: var(--p-text-muted-color);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.setting {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 2rem;
  max-width: 680px;
  padding: 1rem 1.25rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 12px;
  margin-bottom: 0.6rem;
}

.setting__label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.setting__title {
  font-weight: 600;
}

.setting__hint {
  font-size: 0.85rem;
  color: var(--p-text-muted-color);
}

.select {
  padding: 0.45rem 0.6rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 9px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  font: inherit;
  cursor: pointer;
}

.setting--preview {
  max-width: 680px;
  background: #0d1117;
  border-color: #0d1117;
}

.preview {
  color: #e6edf3;
}

.reset {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  margin-top: 1rem;
  padding: 0.5rem 0.9rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 9px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  font: inherit;
  cursor: pointer;
}

.reset:hover {
  background: var(--p-content-hover-background);
}
</style>
