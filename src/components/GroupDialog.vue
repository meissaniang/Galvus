<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { SEPARATOR, nameOf } from "@/utils/groupPath";

/**
 * Dialog « Nouveau dossier » / « Renommer le dossier ».
 *
 * Volontairement minimal : un dossier n'a qu'un nom. Son emplacement se
 * choisit par la navigation — on crée là où l'on se trouve.
 */
const props = defineProps<{
  open: boolean;
  mode: "create" | "rename";
  /** Dossier courant à la création, dossier renommé sinon. Vide = racine. */
  path: string;
  error?: string | null;
}>();

const emit = defineEmits<{ submit: [name: string]; close: [] }>();

const name = ref("");
const input = ref<HTMLInputElement | null>(null);
const submitted = ref(false);

const isRename = computed(() => props.mode === "rename");
const title = computed(() =>
  isRename.value ? "Renommer le dossier" : "Nouveau dossier",
);
const location = computed(() => (isRename.value ? props.path : props.path || "Racine"));

const trimmed = computed(() => name.value.trim());
const hasSeparator = computed(() => trimmed.value.includes(SEPARATOR));
const isValid = computed(() => trimmed.value !== "" && !hasSeparator.value);

watch(
  () => props.open,
  async (open) => {
    if (!open) return;
    submitted.value = false;
    name.value = isRename.value ? nameOf(props.path) : "";
    await nextTick();
    input.value?.focus();
    input.value?.select();
  },
);

function submit(): void {
  submitted.value = true;
  if (!isValid.value) return;
  emit("submit", trimmed.value);
}
</script>

<template>
  <Transition name="dlg">
    <div v-if="open" class="overlay" @click.self="emit('close')">
      <div class="dialog" role="dialog" aria-modal="true">
        <header class="dialog__head">
          <div class="dialog__badge">
            <svg width="15" height="15" viewBox="0 0 18 18" fill="none">
              <path
                d="M2.4 5.2a1.8 1.8 0 011.8-1.8h2.6l1.6 1.9h5.4a1.8 1.8 0 011.8 1.8v5.7a1.8 1.8 0 01-1.8 1.8H4.2a1.8 1.8 0 01-1.8-1.8z"
                stroke="currentColor"
                stroke-width="1.3"
                stroke-linejoin="round"
              />
            </svg>
          </div>
          <div class="dialog__titles">
            <p class="dialog__title">{{ title }}</p>
            <p class="dialog__subtitle">{{ location }}</p>
          </div>
          <button class="dialog__close" type="button" @click="emit('close')">
            <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
              <path
                d="M3.4 3.4l7.2 7.2M10.6 3.4l-7.2 7.2"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </header>

        <div class="dialog__body">
          <div class="field">
            <label for="g-name">Nom</label>
            <input
              id="g-name"
              ref="input"
              v-model="name"
              type="text"
              placeholder="Production"
              spellcheck="false"
              @keydown.enter.prevent="submit"
              @keydown.esc.prevent="emit('close')"
            />
            <p v-if="submitted && !trimmed" class="field__err">Le nom est requis.</p>
            <p v-else-if="hasSeparator" class="field__err">
              Un nom ne peut pas contenir «&nbsp;{{ SEPARATOR }}&nbsp;» — pour imbriquer,
              entre dans le dossier parent puis crée le sous-dossier.
            </p>
            <p v-else-if="error" class="field__err">{{ error }}</p>
          </div>
        </div>

        <footer class="dialog__foot">
          <button type="button" class="btn" @click="emit('close')">Annuler</button>
          <button type="button" class="btn btn--primary" @click="submit">
            {{ isRename ? "Renommer" : "Créer" }}
          </button>
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
  width: 400px;
  max-width: 100%;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  border-radius: 14px;
  box-shadow: var(--g-sh3);
  overflow: hidden;
}

.dlg-enter-active,
.dlg-leave-active {
  transition: opacity 0.14s ease;
}

.dlg-enter-active .dialog,
.dlg-leave-active .dialog {
  transition: transform 0.14s cubic-bezier(0.2, 0.8, 0.3, 1);
}

.dlg-enter-from,
.dlg-leave-to {
  opacity: 0;
}

.dlg-enter-from .dialog,
.dlg-leave-to .dialog {
  transform: scale(0.97) translateY(6px);
}

.dialog__head {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--g-border);
}

.dialog__badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 9px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  color: var(--g-accent);
  flex-shrink: 0;
}

.dialog__titles {
  flex: 1;
  min-width: 0;
}

.dialog__title {
  margin: 0;
  font-size: 13.5px;
  font-weight: 600;
  color: var(--g-t1);
}

.dialog__subtitle {
  margin: 2px 0 0;
  font-size: 11.5px;
  color: var(--g-t3);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dialog__close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--g-t3);
  cursor: pointer;
}

.dialog__close:hover {
  background: var(--g-s2);
  color: var(--g-t1);
}

.dialog__body {
  padding: 16px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field label {
  font-size: 11.5px;
  font-weight: 500;
  color: var(--g-t2);
}

.field input {
  height: 34px;
  padding: 0 11px;
  border-radius: 9px;
  background: var(--g-s0);
  border: 1px solid var(--g-border);
  font-family: inherit;
  font-size: 13px;
  color: var(--g-t1);
  outline: none;
  transition:
    border-color 0.12s ease-out,
    box-shadow 0.12s ease-out;
}

.field input:focus {
  border-color: var(--g-accent);
  box-shadow: 0 0 0 3px var(--g-accent-ring);
}

.field input::placeholder {
  color: var(--g-t3);
}

.field__err {
  margin: 0;
  font-size: 11.5px;
  color: var(--g-danger);
}

.dialog__foot {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 9px;
  padding: 13px 16px;
  border-top: 1px solid var(--g-border);
  background: var(--g-s0);
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
