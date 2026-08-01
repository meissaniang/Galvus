<script setup lang="ts">
import { computed } from "vue";
import SelectButton from "primevue/selectbutton";
import { useThemeStore, type ThemeMode } from "@/stores/theme";

const theme = useThemeStore();

interface ThemeOption {
  label: string;
  value: ThemeMode;
  icon: string;
}

const themeOptions: ThemeOption[] = [
  { label: "Système", value: "system", icon: "pi pi-desktop" },
  { label: "Clair", value: "light", icon: "pi pi-sun" },
  { label: "Sombre", value: "dark", icon: "pi pi-moon" },
];

// Liaison bidirectionnelle avec le store (setMode applique + persiste).
const selectedMode = computed<ThemeMode>({
  get: () => theme.mode,
  set: (value) => theme.setMode(value),
});
</script>

<template>
  <section class="page">
    <header class="page__header">
      <h1><i class="pi pi-cog" /> Paramètres</h1>
      <p>Thème, langue, police, terminal, raccourcis. D'autres réglages arriveront au Livrable 2.</p>
    </header>

    <div class="setting">
      <div class="setting__label">
        <span class="setting__title">Apparence</span>
        <span class="setting__hint"
          >Choisissez le thème de l'interface. « Système » suit votre OS.</span
        >
      </div>
      <SelectButton
        v-model="selectedMode"
        :options="themeOptions"
        option-label="label"
        option-value="value"
        :allow-empty="false"
        aria-labelledby="theme-select"
      >
        <template #option="slotProps">
          <i :class="slotProps.option.icon" />
          <span class="setting__opt-label">{{ slotProps.option.label }}</span>
        </template>
      </SelectButton>
    </div>
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
  margin: 0 0 2rem;
  color: var(--p-text-muted-color);
}

.setting {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 2rem;
  max-width: 640px;
  padding: 1.1rem 1.25rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 12px;
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
  font-size: 0.875rem;
  color: var(--p-text-muted-color);
}

.setting__opt-label {
  margin-left: 0.4rem;
}
</style>
