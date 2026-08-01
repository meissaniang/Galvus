<script setup lang="ts">
// Page Serveurs — placeholder. La liste réelle arrivera au Livrable 1.
</script>

<template>
  <section class="page">
    <header class="page__header">
      <h1><i class="pi pi-server" /> Serveurs</h1>
      <p>Gérez vos connexions SSH. La liste et la connexion arrivent au Livrable 1 (MVP).</p>
    </header>

    <div class="page__placeholder">
      <i class="pi pi-server" />
      <p>Aucun serveur pour l'instant.</p>
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
