<script setup lang="ts">
// Page Paramètres — placeholder. La bascule de thème sera branchée à l'étape 4.
</script>

<template>
  <section class="page">
    <header class="page__header">
      <h1><i class="pi pi-cog" /> Paramètres</h1>
      <p>Thème, langue, police, terminal, raccourcis. Le thème dark/light arrive à l'étape 4.</p>
    </header>

    <div class="page__placeholder">
      <i class="pi pi-cog" />
      <p>Les préférences seront disponibles ici.</p>
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
  margin: 0;
  color: var(--p-text-muted-color);
}

.page__placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  margin-top: 3rem;
  padding: 3rem;
  border: 1px dashed var(--p-content-border-color);
  border-radius: 12px;
  color: var(--p-text-muted-color);
}

.page__placeholder .pi {
  font-size: 2.5rem;
  opacity: 0.5;
}
</style>
