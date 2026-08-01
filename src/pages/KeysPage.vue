<script setup lang="ts">
// Page Clés SSH — placeholder. Le scan de ~/.ssh arrivera au Livrable 1.
</script>

<template>
  <section class="page">
    <header class="page__header">
      <h1><i class="pi pi-key" /> Clés SSH</h1>
      <p>Scan de <code>~/.ssh</code>, fingerprints et types. Implémentation au Livrable 1 (MVP).</p>
    </header>

    <div class="page__placeholder">
      <i class="pi pi-key" />
      <p>Aucune clé détectée pour l'instant.</p>
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

.page__header code {
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  background: var(--p-content-hover-background);
  font-size: 0.9em;
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
