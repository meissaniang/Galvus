<script setup lang="ts">
import { reactive, ref, watch } from "vue";
import type { Server, ServerInput } from "@/types/ssh";
import { pickFile } from "@/services/filePicker";

/**
 * Dialog « Nouveau / Éditer serveur » — fidèle à « ScreenServers.dc.html » :
 * 540px radius 14, grille 2 colonnes, champs surface-2, chips de tags,
 * swatches de couleur du DS, toggle Favori, note de confidentialité.
 */
const props = defineProps<{
  open: boolean;
  server: Server | null;
  groups?: string[];
  defaultGroup?: string | null;
}>();
const emit = defineEmits<{ save: [input: ServerInput]; close: [] }>();

/** Couleurs de pastille du design system. */
const SWATCHES = [
  "#4C8DFF",
  "#A96CF5",
  "#23C48A",
  "#F08A3C",
  "#EC5F9E",
  "#22B8D9",
  "#7B8CA6",
];

function emptyForm(): ServerInput {
  return {
    name: "",
    hostname: "",
    port: 22,
    username: null,
    identityFile: null,
    color: null,
    favorite: false,
    tags: [],
    group: null,
  };
}

const form = reactive<ServerInput>(emptyForm());
const tags = ref<string[]>([]);
const tagDraft = ref("");
const submitted = ref(false);

watch(
  () => props.open,
  (open) => {
    if (!open) return;
    submitted.value = false;
    tagDraft.value = "";
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
            tags: [...source.tags],
            group: source.group,
          }
        : { ...emptyForm(), group: props.defaultGroup ?? null },
    );
    tags.value = source ? [...source.tags] : [];
  },
);

function addTag(): void {
  const value = tagDraft.value.trim().replace(/,$/, "");
  if (value && !tags.value.includes(value)) {
    tags.value.push(value);
  }
  tagDraft.value = "";
}

function removeTag(tag: string): void {
  tags.value = tags.value.filter((t) => t !== tag);
}

function onTagKeydown(event: KeyboardEvent): void {
  if (event.key === "Enter" || event.key === ",") {
    event.preventDefault();
    addTag();
  } else if (event.key === "Backspace" && tagDraft.value === "" && tags.value.length) {
    tags.value.pop();
  }
}

async function browseKey(): Promise<void> {
  const path = await pickFile("Choisir un fichier de clé privée");
  if (path) form.identityFile = path;
}

const isValid = () =>
  form.name.trim() !== "" &&
  form.hostname.trim() !== "" &&
  form.port >= 1 &&
  form.port <= 65535;

function submit(): void {
  submitted.value = true;
  addTag();
  if (!isValid()) return;
  emit("save", {
    name: form.name.trim(),
    hostname: form.hostname.trim(),
    port: Number(form.port),
    username: form.username?.trim() || null,
    identityFile: form.identityFile?.trim() || null,
    color: form.color || null,
    favorite: form.favorite,
    tags: [...tags.value],
    group: form.group?.trim() || null,
  });
}
</script>

<template>
  <Transition name="dlg">
    <div v-if="open" class="overlay" @click.self="emit('close')">
      <div class="dialog" role="dialog" aria-modal="true">
        <header class="dialog__head">
          <div class="dialog__badge">
            <svg width="15" height="15" viewBox="0 0 18 18" fill="none">
              <rect x="2.5" y="2.8" width="13" height="5" rx="1.8" stroke="currentColor" stroke-width="1.5" />
              <rect x="2.5" y="10.2" width="13" height="5" rx="1.8" stroke="currentColor" stroke-width="1.5" />
            </svg>
          </div>
          <div class="dialog__titles">
            <div class="dialog__title">{{ server ? "Éditer le serveur" : "Nouveau serveur" }}</div>
            <div class="dialog__subtitle">Stocké localement, chiffré au repos</div>
          </div>
          <button class="dialog__close" title="Fermer" @click="emit('close')">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </button>
        </header>

        <form class="dialog__body" @submit.prevent="submit">
          <div class="field field--wide">
            <label>Nom</label>
            <input v-model="form.name" type="text" placeholder="db-replica-02" />
            <span v-if="submitted && !form.name.trim()" class="field__err">Requis</span>
          </div>

          <div class="field">
            <label>Hôte</label>
            <input v-model="form.hostname" type="text" class="mono" placeholder="10.4.12.61" />
            <span v-if="submitted && !form.hostname.trim()" class="field__err">Requis</span>
          </div>

          <div class="field">
            <label>Port</label>
            <input v-model.number="form.port" type="number" class="mono" min="1" max="65535" />
          </div>

          <div class="field">
            <label>Utilisateur</label>
            <input v-model="form.username" type="text" class="mono" placeholder="postgres" />
          </div>

          <div class="field">
            <label>Fichier de clé</label>
            <div class="field__file">
              <input v-model="form.identityFile" type="text" class="mono" placeholder="~/.ssh/id_ed25519" />
              <button type="button" class="field__browse" @click="browseKey">Parcourir</button>
            </div>
          </div>

          <div class="field">
            <label>Groupe</label>
            <input v-model="form.group" type="text" list="galvus-groups" placeholder="Production" />
            <datalist id="galvus-groups">
              <option v-for="g in groups ?? []" :key="g" :value="g" />
            </datalist>
          </div>

          <div class="field">
            <label>Tags</label>
            <div class="tags">
              <span v-for="tag in tags" :key="tag" class="tags__chip">
                {{ tag }}
                <button type="button" class="tags__x" @click="removeTag(tag)">
                  <svg width="8" height="8" viewBox="0 0 10 10" fill="none">
                    <path d="M2.5 2.5l5 5M7.5 2.5l-5 5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                  </svg>
                </button>
              </span>
              <input
                v-model="tagDraft"
                type="text"
                class="tags__input"
                :placeholder="tags.length === 0 ? 'postgres, replica…' : '+'"
                @keydown="onTagKeydown"
                @blur="addTag"
              />
            </div>
          </div>

          <div class="field--footer">
            <div class="swatches">
              <label>Couleur de la pastille</label>
              <div class="swatches__row">
                <button
                  type="button"
                  class="swatch swatch--auto"
                  :class="{ 'swatch--on': form.color === null }"
                  title="Automatique"
                  @click="form.color = null"
                >A</button>
                <button
                  v-for="c in SWATCHES"
                  :key="c"
                  type="button"
                  class="swatch"
                  :class="{ 'swatch--on': form.color === c }"
                  :style="{ background: c, '--sw': c }"
                  @click="form.color = c"
                />
              </div>
            </div>
            <label class="fav">
              <span class="fav__label">Favori</span>
              <button
                type="button"
                class="fav__toggle"
                :class="{ 'fav__toggle--on': form.favorite }"
                role="switch"
                :aria-checked="form.favorite"
                @click="form.favorite = !form.favorite"
              >
                <span class="fav__knob" />
              </button>
            </label>
          </div>
        </form>

        <footer class="dialog__foot">
          <span class="dialog__privacy">
            <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
              <circle cx="7" cy="7" r="5.4" stroke="currentColor" stroke-width="1.3" />
              <path d="M7 4.2v.1M7 6.2v3.4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
            Aucune donnée ne quitte votre machine
          </span>
          <div class="dialog__cta">
            <button type="button" class="btn" @click="emit('close')">Annuler</button>
            <button type="button" class="btn btn--primary" @click="submit">
              Enregistrer
            </button>
          </div>
        </footer>
      </div>
    </div>
  </Transition>
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
  background: rgba(6, 10, 15, 0.62);
  backdrop-filter: blur(3px);
}

.dialog {
  width: 540px;
  max-width: 100%;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  border-radius: 14px;
  box-shadow: var(--g-sh3);
  overflow: hidden;
}

/* Animations du DS : backdrop 120ms, carte 180ms scale .97 + translateY. */
.dlg-enter-active {
  transition: opacity 0.12s ease-out;
}
.dlg-enter-active .dialog {
  transition: transform 0.18s cubic-bezier(0.2, 0.8, 0.3, 1), opacity 0.18s ease-out;
}
.dlg-enter-from {
  opacity: 0;
}
.dlg-enter-from .dialog {
  transform: scale(0.97) translateY(8px);
  opacity: 0;
}
.dlg-leave-active {
  transition: opacity 0.12s ease-in;
}
.dlg-leave-active .dialog {
  transition: transform 0.12s ease-in, opacity 0.12s ease-in;
}
.dlg-leave-to {
  opacity: 0;
}
.dlg-leave-to .dialog {
  transform: scale(0.98);
  opacity: 0;
}

.dialog__head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 18px 14px;
  border-bottom: 1px solid var(--g-border);
}

.dialog__badge {
  width: 30px;
  height: 30px;
  border-radius: 9px;
  background: var(--g-accent-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--g-accent);
}

.dialog__titles {
  flex: 1;
}

.dialog__title {
  font-size: 15px;
  font-weight: 600;
  color: var(--g-t1);
}

.dialog__subtitle {
  font-size: 12px;
  color: var(--g-t3);
  margin-top: 1px;
}

.dialog__close {
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--g-t3);
  background: var(--g-s2);
  cursor: pointer;
}

.dialog__close:hover {
  color: var(--g-t1);
  background: var(--g-s3);
}

.dialog__body {
  padding: 16px 18px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 13px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field--wide {
  grid-column: span 2;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field label,
.field--wide label,
.swatches label {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--g-t2);
}

.field input,
.field--wide input {
  height: 34px;
  border-radius: 9px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  padding: 0 11px;
  font-family: inherit;
  font-size: 13px;
  color: var(--g-t1);
  outline: none;
  transition: border-color 0.12s ease-out, box-shadow 0.12s ease-out;
  box-sizing: border-box;
  width: 100%;
}

.field input.mono,
.field--wide input.mono {
  font-family: var(--g-font-mono);
  font-size: 12.5px;
}

.field input:focus,
.field--wide input:focus,
.tags:focus-within {
  border-color: var(--g-accent);
  box-shadow: 0 0 0 3px var(--g-accent-ring);
}

.field input::placeholder {
  color: var(--g-t3);
}

.field__err {
  font-size: 11px;
  color: var(--g-danger);
}

.field__file {
  position: relative;
  display: flex;
  align-items: center;
}

.field__file input {
  padding-right: 84px;
}

.field__browse {
  position: absolute;
  right: 4px;
  height: 26px;
  padding: 0 7px;
  border-radius: 6px;
  border: 1px solid var(--g-border);
  background: var(--g-s1);
  font-family: inherit;
  font-size: 11px;
  font-weight: 600;
  color: var(--g-t2);
  cursor: pointer;
}

.field__browse:hover {
  color: var(--g-t1);
  background: var(--g-s2);
}

/* Tags — chips + saisie inline. */
.tags {
  min-height: 34px;
  border-radius: 9px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 5px;
  padding: 0 9px;
  transition: border-color 0.12s ease-out, box-shadow 0.12s ease-out;
}

.tags__chip {
  margin: 3px 0;
}

.tags__chip {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 10.5px;
  font-weight: 500;
  color: var(--g-t1);
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  padding: 2px 6px 2px 8px;
  border-radius: 999px;
}

.tags__x {
  display: flex;
  border: 0;
  background: transparent;
  color: var(--g-t3);
  cursor: pointer;
  padding: 0;
}

.tags__x:hover {
  color: var(--g-danger);
}

.tags__input {
  flex: 1;
  min-width: 40px;
  border: 0;
  background: transparent;
  font-family: inherit;
  font-size: 12px;
  color: var(--g-t1);
  outline: none;
}

.tags__input::placeholder {
  color: var(--g-t3);
}

/* Rangée couleur + favori. */
.field--footer {
  grid-column: span 2;
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px;
  padding-top: 2px;
}

.swatches {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.swatches__row {
  display: flex;
  gap: 7px;
}

.swatch {
  width: 24px;
  height: 24px;
  border-radius: 8px;
  border: 0;
  cursor: pointer;
  padding: 0;
}

.swatch--on {
  box-shadow: 0 0 0 2px var(--g-s1), 0 0 0 4px var(--sw, var(--g-accent));
}

.swatch--auto {
  background: var(--g-s2);
  border: 1px solid var(--g-border-2);
  color: var(--g-t2);
  font-size: 11px;
  font-weight: 700;
  --sw: var(--g-border-2);
}

.fav {
  display: flex;
  align-items: center;
  gap: 9px;
}

.fav__label {
  font-size: 12.5px;
  color: var(--g-t2);
}

.fav__toggle {
  width: 38px;
  height: 22px;
  border-radius: 999px;
  border: 0;
  background: var(--g-s3);
  display: flex;
  align-items: center;
  padding: 0 3px;
  cursor: pointer;
  transition: background 0.14s ease;
  box-sizing: border-box;
}

.fav__toggle--on {
  background: var(--g-accent);
  justify-content: flex-end;
}

.fav__knob {
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: #fff;
  box-shadow: var(--g-sh1);
}

.dialog__foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 13px 18px;
  border-top: 1px solid var(--g-border);
  background: var(--g-s0);
}

.dialog__privacy {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 12px;
  color: var(--g-t3);
}

.dialog__cta {
  display: flex;
  gap: 9px;
}

.btn {
  height: 32px;
  display: flex;
  align-items: center;
  padding: 0 14px;
  border-radius: 9px;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--g-t2);
  cursor: pointer;
  transition: background 0.12s ease, color 0.12s ease;
}

.btn:hover {
  background: var(--g-s2);
  color: var(--g-t1);
}

.btn--primary {
  padding: 0 16px;
  border: 0;
  background: var(--g-accent);
  color: var(--g-accent-fg);
  font-weight: 600;
  box-shadow: var(--g-sh1);
}

.btn--primary:hover {
  background: var(--g-accent-h);
  color: var(--g-accent-fg);
}
</style>
