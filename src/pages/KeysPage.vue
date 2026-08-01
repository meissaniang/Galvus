<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { storeToRefs } from "pinia";
import { useKeysStore } from "@/stores/keys";
import { keysRepository } from "@/repositories/keysRepository";
import KeyCard from "@/components/KeyCard.vue";
import type { SshKey } from "@/types/ssh";

const store = useKeysStore();
const { filteredKeys, keys, loading, error, query } = storeToRefs(store);

// --- Génération ---
const genOpen = ref(false);
const gen = reactive({
  name: "",
  keyType: "ed25519" as "ed25519" | "rsa" | "ecdsa",
  comment: "",
  passphrase: "",
});
const genSubmitted = ref(false);
const busy = ref(false);

function openGenerate(): void {
  gen.name = "";
  gen.keyType = "ed25519";
  gen.comment = "";
  gen.passphrase = "";
  genSubmitted.value = false;
  genOpen.value = true;
}

async function submitGenerate(): Promise<void> {
  genSubmitted.value = true;
  if (!gen.name.trim()) return;
  busy.value = true;
  try {
    await store.generate({
      name: gen.name.trim(),
      keyType: gen.keyType,
      comment: gen.comment.trim(),
      passphrase: gen.passphrase,
    });
    genOpen.value = false;
  } catch {
    /* erreur affichée via store.error */
  } finally {
    busy.value = false;
  }
}

// --- Import ---
async function importKey(): Promise<void> {
  const path = await keysRepository.pickKeyFile();
  if (!path) return;
  const base = path.split("/").pop()?.replace(/\.pub$/, "") ?? "cle-importee";
  const name = window.prompt("Nom du fichier dans ~/.ssh :", base);
  if (!name) return;
  try {
    await store.importFrom(path, name.trim());
  } catch {
    /* erreur affichée via store.error */
  }
}

// --- Suppression ---
async function removeKey(key: SshKey): Promise<void> {
  if (window.confirm(`Supprimer la clé « ${key.name} » (privée + publique) ?`)) {
    await store.remove(key.name);
  }
}

onMounted(() => {
  if (keys.value.length === 0) store.load();
});
</script>

<template>
  <section class="page">
    <div class="toolbar">
      <div class="toolbar__search">
        <i class="pi pi-search" />
        <input v-model="query" type="text" placeholder="Rechercher une clé…" spellcheck="false" />
      </div>
      <button class="btn" @click="importKey"><i class="pi pi-upload" /> Importer</button>
      <button class="btn btn--primary" @click="openGenerate">
        <i class="pi pi-plus" /> Générer
      </button>
    </div>

    <header class="page__header">
      <h1>Clés SSH</h1>
      <span class="page__count">{{ filteredKeys.length }}</span>
    </header>

    <p v-if="error" class="state state--error">
      <i class="pi pi-exclamation-triangle" /> {{ error }}
    </p>

    <p v-else-if="loading && keys.length === 0" class="state">
      <i class="pi pi-spin pi-spinner" /> Scan de ~/.ssh…
    </p>

    <div v-else-if="filteredKeys.length === 0" class="state state--empty">
      <i class="pi pi-key" />
      <p v-if="keys.length === 0">Aucune clé. Génère ou importe une clé ci-dessus.</p>
      <p v-else>Aucune clé ne correspond à « {{ query }} ».</p>
    </div>

    <div v-else class="grid">
      <KeyCard
        v-for="item in filteredKeys"
        :key="item.path"
        :key-item="item"
        @remove="removeKey"
      />
    </div>

    <p class="page__hint">
      <i class="pi pi-info-circle" /> Génération via <code>ssh-keygen</code>, permissions
      privées en 600. Aucune clé privée n'est copiée ailleurs que dans <code>~/.ssh</code>.
    </p>

    <!-- Dialog génération -->
    <div v-if="genOpen" class="overlay" @click.self="genOpen = false">
      <div class="dialog" role="dialog" aria-modal="true">
        <header class="dialog__head">
          <h2>Générer une clé SSH</h2>
          <button class="icon-btn" @click="genOpen = false"><i class="pi pi-times" /></button>
        </header>
        <form class="dialog__body" @submit.prevent="submitGenerate">
          <label class="field">
            <span>Nom du fichier *</span>
            <input v-model="gen.name" type="text" placeholder="ma-cle" />
            <small v-if="genSubmitted && !gen.name.trim()" class="err">Requis</small>
          </label>
          <label class="field">
            <span>Type</span>
            <select v-model="gen.keyType">
              <option value="ed25519">ED25519 (recommandé)</option>
              <option value="rsa">RSA 4096</option>
              <option value="ecdsa">ECDSA</option>
            </select>
          </label>
          <label class="field field--wide">
            <span>Commentaire</span>
            <input v-model="gen.comment" type="text" placeholder="moi@machine" />
          </label>
          <label class="field field--wide">
            <span>Passphrase (optionnelle)</span>
            <input v-model="gen.passphrase" type="password" placeholder="Laisser vide = sans passphrase" />
          </label>
        </form>
        <footer class="dialog__foot">
          <button class="btn" @click="genOpen = false">Annuler</button>
          <button class="btn btn--primary" :disabled="busy" @click="submitGenerate">
            <i v-if="busy" class="pi pi-spin pi-spinner" /> Générer
          </button>
        </footer>
      </div>
    </div>
  </section>
</template>

<style scoped>
.page {
  height: 100%;
  overflow-y: auto;
  padding: 18px 20px 24px;
  box-sizing: border-box;
}

.toolbar {
  display: flex;
  gap: 0.6rem;
  margin-bottom: 1.5rem;
}
.toolbar__search {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
  padding: 0 0.85rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 10px;
  background: var(--p-content-background);
  color: var(--p-text-muted-color);
}
.toolbar__search:focus-within {
  border-color: var(--p-primary-color);
}
.toolbar__search input {
  flex: 1;
  padding: 0.65rem 0;
  border: 0;
  background: transparent;
  color: var(--p-text-color);
  font: inherit;
  outline: none;
}
.btn {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0 1rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 10px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  font: inherit;
  cursor: pointer;
  white-space: nowrap;
}
.btn:hover {
  background: var(--p-content-hover-background);
}
.btn--primary {
  border-color: var(--p-primary-color);
  background: var(--p-primary-color);
  color: var(--p-primary-contrast-color, #fff);
}
.btn--primary:disabled {
  opacity: 0.6;
  cursor: default;
}
.page__header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-bottom: 1rem;
}
.page__header h1 {
  margin: 0;
  font-size: 1.4rem;
}
.page__count {
  padding: 0.05rem 0.5rem;
  border-radius: 999px;
  background: var(--p-content-hover-background);
  color: var(--p-text-muted-color);
  font-size: 0.8rem;
  font-weight: 600;
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 0.85rem;
}
.state {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--p-text-muted-color);
}
.state--error {
  color: #ef4444;
}
.state--empty {
  flex-direction: column;
  justify-content: center;
  gap: 0.75rem;
  margin-top: 2rem;
  padding: 3rem;
  border: 1px dashed var(--p-content-border-color);
  border-radius: 12px;
}
.state--empty .pi {
  font-size: 2.5rem;
  opacity: 0.5;
}
code {
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  background: var(--p-content-hover-background);
  font-size: 0.9em;
}
.page__hint {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  margin-top: 1.75rem;
  font-size: 0.82rem;
  color: var(--p-text-muted-color);
}

/* Dialog */
.overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  background: rgb(0 0 0 / 0.5);
}
.dialog {
  width: 100%;
  max-width: 480px;
  border: 1px solid var(--p-content-border-color);
  border-radius: 14px;
  background: var(--p-content-background);
  box-shadow: 0 20px 60px rgb(0 0 0 / 0.35);
}
.dialog__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--p-content-border-color);
}
.dialog__head h2 {
  margin: 0;
  font-size: 1.1rem;
}
.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--p-text-muted-color);
  cursor: pointer;
}
.icon-btn:hover {
  background: var(--p-content-hover-background);
}
.dialog__body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.9rem 1rem;
  padding: 1.25rem;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}
.field--wide {
  grid-column: 1 / -1;
}
.field > span {
  font-size: 0.82rem;
  color: var(--p-text-muted-color);
}
.field input,
.field select {
  padding: 0.55rem 0.7rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 9px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  font: inherit;
  outline: none;
}
.field input:focus,
.field select:focus {
  border-color: var(--p-primary-color);
}
.err {
  color: #ef4444;
  font-size: 0.75rem;
}
.dialog__foot {
  display: flex;
  justify-content: flex-end;
  gap: 0.6rem;
  padding: 1rem 1.25rem;
  border-top: 1px solid var(--p-content-border-color);
}
</style>
