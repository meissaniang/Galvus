<script setup lang="ts">
import { reactive, ref, watch } from "vue";
import type { Server, ServerInput } from "@/types/ssh";

const props = defineProps<{ open: boolean; server: Server | null }>();
const emit = defineEmits<{ save: [input: ServerInput]; close: [] }>();

function emptyForm(): ServerInput {
  return {
    name: "",
    hostname: "",
    port: 22,
    username: null,
    identityFile: null,
    color: null,
    favorite: false,
  };
}

const form = reactive<ServerInput>(emptyForm());
const submitted = ref(false);

// (Ré)initialise le formulaire à chaque ouverture.
watch(
  () => props.open,
  (open) => {
    if (!open) return;
    submitted.value = false;
    const source = props.server;
    Object.assign(
      form,
      source
        ? {
            name: source.name,
            hostname: source.hostname,
            port: source.port,
            username: source.username,
            identityFile: source.identityFile,
            color: source.color,
            favorite: source.favorite,
          }
        : emptyForm(),
    );
  },
);

const isValid = () =>
  form.name.trim() !== "" &&
  form.hostname.trim() !== "" &&
  form.port >= 1 &&
  form.port <= 65535;

function submit(): void {
  submitted.value = true;
  if (!isValid()) return;
  emit("save", {
    name: form.name.trim(),
    hostname: form.hostname.trim(),
    port: Number(form.port),
    username: form.username?.trim() || null,
    identityFile: form.identityFile?.trim() || null,
    color: form.color || null,
    favorite: form.favorite,
  });
}
</script>

<template>
  <div v-if="open" class="overlay" @click.self="emit('close')">
    <div class="dialog" role="dialog" aria-modal="true">
      <header class="dialog__head">
        <h2>{{ server ? "Éditer le serveur" : "Nouveau serveur" }}</h2>
        <button class="icon-btn" title="Fermer" @click="emit('close')">
          <i class="pi pi-times" />
        </button>
      </header>

      <form class="dialog__body" @submit.prevent="submit">
        <label class="field field--wide">
          <span>Nom *</span>
          <input v-model="form.name" type="text" placeholder="Mon serveur" />
          <small v-if="submitted && !form.name.trim()" class="err">Requis</small>
        </label>

        <label class="field field--wide">
          <span>Hôte *</span>
          <input v-model="form.hostname" type="text" placeholder="192.168.1.10 ou example.com" />
          <small v-if="submitted && !form.hostname.trim()" class="err">Requis</small>
        </label>

        <label class="field">
          <span>Port</span>
          <input v-model.number="form.port" type="number" min="1" max="65535" />
        </label>

        <label class="field">
          <span>Utilisateur</span>
          <input v-model="form.username" type="text" placeholder="root" />
        </label>

        <label class="field field--wide">
          <span>Fichier de clé (IdentityFile)</span>
          <input v-model="form.identityFile" type="text" placeholder="~/.ssh/ma-cle" />
        </label>

        <label class="field field--color">
          <span>Couleur</span>
          <input v-model="form.color" type="color" />
        </label>

        <label class="field field--check">
          <input v-model="form.favorite" type="checkbox" />
          <span>Favori</span>
        </label>
      </form>

      <footer class="dialog__foot">
        <button class="btn" @click="emit('close')">Annuler</button>
        <button class="btn btn--primary" @click="submit">
          {{ server ? "Enregistrer" : "Créer" }}
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
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
  max-width: 520px;
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
  font-size: 1.15rem;
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

.field input[type="text"],
.field input[type="number"] {
  padding: 0.55rem 0.7rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 9px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  font: inherit;
  outline: none;
}

.field input[type="text"]:focus,
.field input[type="number"]:focus {
  border-color: var(--p-primary-color);
}

.field--color input[type="color"] {
  width: 48px;
  height: 34px;
  padding: 0;
  border: 1px solid var(--p-content-border-color);
  border-radius: 8px;
  background: none;
  cursor: pointer;
}

.field--check {
  flex-direction: row;
  align-items: center;
  gap: 0.5rem;
  align-self: end;
}

.field--check input {
  width: 16px;
  height: 16px;
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

.btn {
  padding: 0.55rem 1rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 9px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  font: inherit;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.btn:hover {
  background: var(--p-content-hover-background);
}

.btn--primary {
  border-color: var(--p-primary-color);
  background: var(--p-primary-color);
  color: var(--p-primary-contrast-color, #fff);
}

.btn--primary:hover {
  filter: brightness(1.05);
}
</style>
