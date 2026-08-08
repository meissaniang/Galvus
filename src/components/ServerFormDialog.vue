<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import type { ServerItem, ServerSource } from "@/types/ssh";
import { OSES } from "@/utils/osIcons";
import { pickFile } from "@/services/filePicker";

/**
 * Dialog « Nouveau / Éditer serveur » — fidèle à « ScreenServers.dc.html ».
 *
 * Il sert les deux origines : base chiffrée et `~/.ssh/config`. À la création,
 * un sélecteur permet de choisir l'emplacement ; à l'édition il est figé, un
 * serveur ne migrant pas d'une origine à l'autre.
 */
const props = defineProps<{
  open: boolean;
  item: ServerItem | null;
  groups?: string[];
  defaultGroup?: string | null;
  defaultSource?: ServerSource;
}>();

export interface ServerFormResult {
  source: ServerSource;
  /** Origine avant édition : différente de `source` en cas de déplacement. */
  previousSource: ServerSource | null;
  id: number | null;
  originalAlias: string | null;
  name: string;
  hostname: string;
  port: number;
  username: string | null;
  identityFile: string | null;
  color: string | null;
  favorite: boolean;
  tags: string[];
  group: string | null;
  os: string | null;
}

const emit = defineEmits<{ save: [result: ServerFormResult]; close: [] }>();

const source = ref<ServerSource>("local");
const form = reactive({
  name: "",
  hostname: "",
  port: 22,
  username: "" as string | null,
  identityFile: "" as string | null,
  color: null as string | null,
  favorite: false,
  group: "" as string | null,
  os: null as string | null,
});
/**
 * Tags et couleur ne sont plus saisissables — ils encombraient la création
 * pour un usage rare. Les valeurs existantes sont conservées et réémises
 * telles quelles : retirer un champ de l'IHM ne doit pas effacer la donnée.
 */
const tags = ref<string[]>([]);
const submitted = ref(false);

const isEdit = computed(() => props.item !== null);

/** Vrai quand l'édition change l'emplacement : il faudra migrer l'entrée. */
const isMoving = computed(
  () => props.item !== null && props.item.source !== source.value,
);

/** Dans le fichier de config, le nom EST l'alias : pas d'espace autorisé. */
const nameHasSpace = computed(
  () => source.value === "config" && form.name.trim().includes(" "),
);

watch(
  () => props.open,
  (open) => {
    if (!open) return;
    submitted.value = false;
    const item = props.item;

    source.value = item?.source ?? props.defaultSource ?? "local";
    Object.assign(form, {
      name: item?.name ?? "",
      hostname: item?.hostname ?? "",
      port: item?.port ?? 22,
      username: item?.username ?? "",
      identityFile: item?.identityFile ?? "",
      color: item?.color ?? null,
      favorite: item?.favorite ?? false,
      group: item?.group ?? props.defaultGroup ?? "",
      os: item?.os ?? null,
    });
    tags.value = item ? [...item.tags] : [];
  },
);

async function browseKey(): Promise<void> {
  const path = await pickFile("Choisir un fichier de clé privée");
  if (path) form.identityFile = path;
}

const isValid = () =>
  form.name.trim() !== "" &&
  form.hostname.trim() !== "" &&
  !nameHasSpace.value &&
  form.port >= 1 &&
  form.port <= 65535;

function submit(): void {
  submitted.value = true;
  if (!isValid()) return;
  emit("save", {
    source: source.value,
    previousSource: props.item?.source ?? null,
    id: props.item?.id ?? null,
    originalAlias: props.item?.alias ?? null,
    name: form.name.trim(),
    hostname: form.hostname.trim(),
    port: Number(form.port),
    username: form.username?.trim() || null,
    identityFile: form.identityFile?.trim() || null,
    color: form.color || null,
    favorite: form.favorite,
    tags: [...tags.value],
    group: form.group?.trim() || null,
    os: form.os || null,
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
              <rect
                x="2.5"
                y="2.8"
                width="13"
                height="5"
                rx="1.8"
                stroke="currentColor"
                stroke-width="1.5"
              />
              <rect
                x="2.5"
                y="10.2"
                width="13"
                height="5"
                rx="1.8"
                stroke="currentColor"
                stroke-width="1.5"
              />
            </svg>
          </div>
          <div class="dialog__titles">
            <div class="dialog__title">
              {{ isEdit ? "Éditer le serveur" : "Nouveau serveur" }}
            </div>
            <div class="dialog__subtitle">
              {{
                source === "config"
                  ? "Enregistré dans ~/.ssh/config"
                  : "Stocké localement, chiffré au repos"
              }}
            </div>
          </div>
          <button class="dialog__close" title="Fermer" @click="emit('close')">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path
                d="M3 3l6 6M9 3l-6 6"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </header>

        <form class="dialog__body" @submit.prevent="submit">
          <div class="field field--wide">
            <label>Emplacement</label>
            <div class="segmented">
              <button
                type="button"
                class="segmented__btn"
                :class="{ 'segmented__btn--on': source === 'local' }"
                @click="source = 'local'"
              >
                Base chiffrée
              </button>
              <button
                type="button"
                class="segmented__btn"
                :class="{ 'segmented__btn--on': source === 'config' }"
                @click="source = 'config'"
              >
                ~/.ssh/config
              </button>
            </div>
            <span v-if="isMoving" class="field__hint field__hint--move">
              Le serveur sera déplacé vers
              {{ source === "config" ? "~/.ssh/config" : "la base chiffrée" }} : recréé à
              destination, puis retiré de son emplacement actuel.
            </span>
            <span v-else class="field__hint">
              {{
                source === "config"
                  ? "Portable et sauvegardable : l'entrée est lisible par ssh, scp et tout autre client."
                  : "Base locale chiffrée, clé dans le trousseau du système."
              }}
            </span>
          </div>

          <div class="field field--wide">
            <label>{{ source === "config" ? "Alias (Host)" : "Nom" }}</label>
            <input
              v-model="form.name"
              type="text"
              :class="{ mono: source === 'config' }"
              :placeholder="source === 'config' ? 'vps-prod' : 'db-replica-02'"
            />
            <span v-if="submitted && !form.name.trim()" class="field__err">Requis</span>
            <span v-else-if="submitted && nameHasSpace" class="field__err">
              Un seul mot, sans espace
            </span>
          </div>

          <div class="field">
            <label>Hôte</label>
            <input
              v-model="form.hostname"
              type="text"
              class="mono"
              placeholder="10.4.12.61"
            />
            <span v-if="submitted && !form.hostname.trim()" class="field__err"
              >Requis</span
            >
          </div>

          <div class="field">
            <label>Port</label>
            <input
              v-model.number="form.port"
              type="number"
              class="mono"
              min="1"
              max="65535"
            />
          </div>

          <div class="field">
            <label>Utilisateur</label>
            <input v-model="form.username" type="text" class="mono" placeholder="root" />
          </div>

          <div class="field">
            <label>Fichier de clé</label>
            <div class="field__file">
              <input
                v-model="form.identityFile"
                type="text"
                class="mono"
                placeholder="~/.ssh/id_ed25519"
              />
              <button type="button" class="field__browse" @click="browseKey">
                Parcourir
              </button>
            </div>
          </div>

          <div class="field">
            <label>Groupe</label>
            <input
              v-model="form.group"
              type="text"
              list="galvus-groups"
              placeholder="Production"
            />
            <datalist id="galvus-groups">
              <option v-for="g in groups ?? []" :key="g" :value="g" />
            </datalist>
          </div>

          <div class="field--os">
            <label for="f-os">Système</label>
            <select id="f-os" v-model="form.os" class="input">
              <option :value="null">Détecter automatiquement</option>
              <option v-for="os in OSES" :key="os.id" :value="os.id">
                {{ os.label }}
              </option>
            </select>
            <p class="field__hint">
              Reconnu depuis la bannière de connexion. À renseigner à la main si le
              serveur n'en affiche pas.
            </p>
          </div>

          <div class="field--footer">
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
              <path
                d="M7 4.2v.1M7 6.2v3.4"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
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
  transition:
    transform 0.18s cubic-bezier(0.2, 0.8, 0.3, 1),
    opacity 0.18s ease-out;
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
  transition:
    transform 0.12s ease-in,
    opacity 0.12s ease-in;
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
  flex-shrink: 0;
}

.dialog__titles {
  flex: 1;
  min-width: 0;
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
  flex-shrink: 0;
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
.field__hint {
  font-size: 11px;
  color: var(--g-t3);
}

.segmented {
  display: flex;
  padding: 3px;
  gap: 3px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  border-radius: 10px;
}

.segmented__btn {
  flex: 1;
  height: 28px;
  border: 0;
  border-radius: 7px;
  background: transparent;
  font-family: inherit;
  font-size: 12px;
  font-weight: 500;
  color: var(--g-t2);
  cursor: pointer;
  transition:
    background 0.12s ease,
    color 0.12s ease;
}

.segmented__btn--on {
  background: var(--g-s1);
  color: var(--g-t1);
  font-weight: 600;
  box-shadow: var(--g-sh1);
}

.field input {
  height: 34px;
  border-radius: 9px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  padding: 0 11px;
  font-family: inherit;
  font-size: 13px;
  color: var(--g-t1);
  outline: none;
  transition:
    border-color 0.12s ease-out,
    box-shadow 0.12s ease-out;
  box-sizing: border-box;
  width: 100%;
}

.field input.mono {
  font-family: var(--g-font-mono);
  font-size: 12.5px;
}

.field input:focus,
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

/* Rangée couleur + favori. */
.field--footer {
  grid-column: span 2;
  display: flex;
  align-items: center;
  padding-top: 2px;
}

.field--os {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 14px;
}

.field--os label {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--g-t2);
}

.field__hint {
  margin: 0;
  font-size: 11px;
  color: var(--g-t3);
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
  transition:
    background 0.12s ease,
    color 0.12s ease;
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
