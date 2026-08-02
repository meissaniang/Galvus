<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { keysRepository } from "@/repositories/keysRepository";
import type { SshKey } from "@/types/ssh";

/**
 * Dialog d'édition d'une clé SSH : contenu de la clé privée, de la clé
 * publique, et gestion de la passphrase (`ssh-keygen -p`).
 *
 * Tout se passe localement dans `~/.ssh` ; une sauvegarde `.bak` est créée
 * avant chaque écriture côté Rust.
 */
const props = defineProps<{ open: boolean; keyItem: SshKey | null }>();
const emit = defineEmits<{
  saveContent: [payload: { name: string; kind: "private" | "public"; content: string }];
  changePassphrase: [payload: { name: string; oldPassphrase: string; newPassphrase: string }];
  close: [];
}>();

type Tab = "private" | "public" | "passphrase";
const tab = ref<Tab>("private");

const privateContent = ref("");
const publicContent = ref("");
const loading = ref(false);
const loadError = ref<string | null>(null);
const copied = ref(false);

// Passphrase
const oldPass = ref("");
const newPass = ref("");
const confirmPass = ref("");
const showPass = ref(false);
const passSubmitted = ref(false);

const TABS: { id: Tab; label: string }[] = [
  { id: "private", label: "Clé privée" },
  { id: "public", label: "Clé publique" },
  { id: "passphrase", label: "Passphrase" },
];

watch(
  () => props.open,
  async (open) => {
    if (!open || !props.keyItem) return;
    tab.value = "private";
    copied.value = false;
    loadError.value = null;
    oldPass.value = "";
    newPass.value = "";
    confirmPass.value = "";
    showPass.value = false;
    passSubmitted.value = false;

    loading.value = true;
    try {
      const name = props.keyItem.name;
      privateContent.value = props.keyItem.hasPrivate
        ? await keysRepository.readPrivate(name)
        : "";
      publicContent.value = await keysRepository.readPublic(name);
    } catch (e) {
      loadError.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  },
);

/** Robustesse de la nouvelle passphrase (mêmes règles que la génération). */
const strength = computed(() => {
  const p = newPass.value;
  if (!p) return null;
  let score = Math.min(p.length / 16, 1) * 0.6;
  if (/[a-z]/.test(p) && /[A-Z]/.test(p)) score += 0.13;
  if (/\d/.test(p)) score += 0.13;
  if (/[^a-zA-Z0-9]/.test(p)) score += 0.14;
  const pct = Math.round(Math.min(score, 1) * 100);
  if (pct < 40) return { pct, label: "faible", color: "var(--g-danger)" };
  if (pct < 70) return { pct, label: "moyenne", color: "var(--g-warning)" };
  return { pct, label: "solide", color: "var(--g-success)" };
});

const mismatch = computed(
  () => newPass.value !== "" && confirmPass.value !== "" && newPass.value !== confirmPass.value,
);

/** Libellé de l'action selon l'état actuel de la clé. */
const passAction = computed(() => {
  if (!props.keyItem?.encrypted) return "Définir la passphrase";
  return newPass.value === "" ? "Retirer la passphrase" : "Changer la passphrase";
});

async function copyCurrent(): Promise<void> {
  const text = tab.value === "public" ? publicContent.value : privateContent.value;
  await navigator.clipboard.writeText(text);
  copied.value = true;
  setTimeout(() => (copied.value = false), 1500);
}

function saveContent(): void {
  if (!props.keyItem) return;
  const kind = tab.value === "public" ? "public" : "private";
  emit("saveContent", {
    name: props.keyItem.name,
    kind,
    content: kind === "public" ? publicContent.value : privateContent.value,
  });
}

function submitPassphrase(): void {
  if (!props.keyItem) return;
  passSubmitted.value = true;
  if (mismatch.value) return;
  emit("changePassphrase", {
    name: props.keyItem.name,
    oldPassphrase: oldPass.value,
    newPassphrase: newPass.value,
  });
}
</script>

<template>
  <Transition name="dlg">
    <div v-if="open && keyItem" class="overlay" @click.self="emit('close')">
      <div class="dialog" role="dialog" aria-modal="true">
        <header class="dialog__head">
          <div class="dialog__badge">
            <svg width="15" height="15" viewBox="0 0 18 18" fill="none">
              <circle cx="6.2" cy="6.2" r="3.4" stroke="currentColor" stroke-width="1.6" />
              <path d="M8.7 8.7L14.5 14.5M12 13l1.6-1.6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
            </svg>
          </div>
          <div class="dialog__titles">
            <div class="dialog__title">Modifier la clé — {{ keyItem.name }}</div>
            <div class="dialog__subtitle">
              ~/.ssh/{{ keyItem.name }} · sauvegarde .bak avant écriture
            </div>
          </div>
          <button class="dialog__close" title="Fermer" @click="emit('close')">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
          </button>
        </header>

        <div class="tabs">
          <button
            v-for="t in TABS"
            :key="t.id"
            class="tabs__btn"
            :class="{ 'tabs__btn--on': tab === t.id }"
            :disabled="t.id === 'private' && !keyItem.hasPrivate"
            @click="tab = t.id"
          >{{ t.label }}</button>
        </div>

        <p v-if="loadError" class="err">{{ loadError }}</p>
        <p v-else-if="loading" class="muted">Lecture de la clé…</p>

        <!-- Contenu clé privée / publique -->
        <template v-else-if="tab !== 'passphrase'">
          <div v-if="tab === 'private'" class="warn">
            <span class="warn__mark">!</span>
            Ne partagez jamais cette clé. Toute modification incorrecte la rendra inutilisable.
          </div>

          <textarea
            v-if="tab === 'private'"
            v-model="privateContent"
            class="editor"
            spellcheck="false"
            autocapitalize="off"
            autocorrect="off"
          />
          <textarea
            v-else
            v-model="publicContent"
            class="editor editor--short"
            spellcheck="false"
            autocapitalize="off"
            autocorrect="off"
          />

          <footer class="dialog__foot">
            <button type="button" class="btn" @click="copyCurrent">
              {{ copied ? "Copié ✓" : "Copier" }}
            </button>
            <div class="dialog__cta">
              <button type="button" class="btn" @click="emit('close')">Fermer</button>
              <button type="button" class="btn btn--primary" @click="saveContent">
                Enregistrer
              </button>
            </div>
          </footer>
        </template>

        <!-- Passphrase -->
        <template v-else>
          <div class="passbody">
            <div class="passstate">
              <span
                class="passstate__badge"
                :class="keyItem.encrypted ? 'passstate__badge--on' : 'passstate__badge--off'"
              >
                {{ keyItem.encrypted ? "protégée par une passphrase" : "sans passphrase" }}
              </span>
            </div>

            <div v-if="keyItem.encrypted" class="field">
              <label>Passphrase actuelle</label>
              <input
                v-model="oldPass"
                :type="showPass ? 'text' : 'password'"
                class="mono"
                placeholder="••••••••"
              />
            </div>

            <div class="field">
              <label>
                Nouvelle passphrase
                <span class="field__opt">(vide = retirer la protection)</span>
              </label>
              <div class="field__pass">
                <input
                  v-model="newPass"
                  :type="showPass ? 'text' : 'password'"
                  class="mono"
                  placeholder="••••••••"
                />
                <button type="button" class="field__eye" title="Afficher" @click="showPass = !showPass">
                  <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                    <path d="M1.6 8s2.4-4 6.4-4 6.4 4 6.4 4-2.4 4-6.4 4S1.6 8 1.6 8z" stroke="currentColor" stroke-width="1.3" />
                    <circle cx="8" cy="8" r="1.7" stroke="currentColor" stroke-width="1.3" />
                  </svg>
                </button>
              </div>
              <div v-if="strength" class="gauge">
                <div class="gauge__track">
                  <div class="gauge__fill" :style="{ width: strength.pct + '%', background: strength.color }" />
                </div>
                <span class="gauge__label" :style="{ color: strength.color }">{{ strength.label }}</span>
              </div>
            </div>

            <div v-if="newPass" class="field">
              <label>Confirmer</label>
              <input
                v-model="confirmPass"
                :type="showPass ? 'text' : 'password'"
                class="mono"
                placeholder="••••••••"
              />
              <span v-if="passSubmitted && mismatch" class="field__err">
                Les passphrases ne correspondent pas
              </span>
            </div>
          </div>

          <footer class="dialog__foot">
            <span class="foot__hint">Opération réalisée par <code>ssh-keygen -p</code></span>
            <div class="dialog__cta">
              <button type="button" class="btn" @click="emit('close')">Annuler</button>
              <button type="button" class="btn btn--primary" @click="submitPassphrase">
                {{ passAction }}
              </button>
            </div>
          </footer>
        </template>
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
  width: 620px;
  max-width: 100%;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  border-radius: 14px;
  box-shadow: var(--g-sh3);
  overflow: hidden;
}

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
.dlg-leave-to {
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
  color: var(--g-accent);
  display: flex;
  align-items: center;
  justify-content: center;
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

.tabs {
  display: flex;
  gap: 3px;
  padding: 3px;
  margin: 14px 18px 0;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  border-radius: 10px;
}

.tabs__btn {
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
  transition: background 0.12s ease, color 0.12s ease;
}

.tabs__btn--on {
  background: var(--g-s1);
  color: var(--g-t1);
  font-weight: 600;
  box-shadow: var(--g-sh1);
}

.tabs__btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.warn {
  display: flex;
  align-items: center;
  gap: 9px;
  margin: 14px 18px 0;
  padding: 9px 12px;
  border-radius: 10px;
  background: var(--g-danger-soft);
  border: 1px solid var(--g-danger);
  font-size: 12px;
  color: var(--g-t1);
}

.warn__mark {
  width: 20px;
  height: 20px;
  border-radius: 6px;
  background: var(--g-danger);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
}

.editor {
  display: block;
  width: calc(100% - 36px);
  margin: 12px 18px;
  height: 260px;
  padding: 12px;
  border-radius: 10px;
  background: var(--g-s0);
  border: 1px solid var(--g-border);
  font-family: var(--g-font-mono);
  font-size: 11px;
  line-height: 1.55;
  color: var(--g-t2);
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.12s ease-out, box-shadow 0.12s ease-out;
}

.editor--short {
  height: 130px;
}

.editor:focus {
  border-color: var(--g-accent);
  box-shadow: 0 0 0 3px var(--g-accent-ring);
  color: var(--g-t1);
}

.passbody {
  padding: 14px 18px 4px;
  display: flex;
  flex-direction: column;
  gap: 13px;
}

.passstate__badge {
  display: inline-flex;
  align-items: center;
  font-size: 10.5px;
  font-weight: 600;
  padding: 3px 9px;
  border-radius: 999px;
}

.passstate__badge--on {
  color: var(--g-success);
  background: var(--g-accent-soft);
}

.passstate__badge--off {
  color: var(--g-t3);
  background: var(--g-s2);
  border: 1px dashed var(--g-border-2);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field label {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--g-t2);
}

.field__opt {
  font-weight: 400;
  color: var(--g-t3);
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
  width: 100%;
  box-sizing: border-box;
  transition: border-color 0.12s ease-out, box-shadow 0.12s ease-out;
}

.field input.mono {
  font-family: var(--g-font-mono);
  font-size: 12.5px;
}

.field input:focus {
  border-color: var(--g-accent);
  box-shadow: 0 0 0 3px var(--g-accent-ring);
}

.field__pass {
  position: relative;
  display: flex;
  align-items: center;
}

.field__pass input {
  padding-right: 38px;
}

.field__eye {
  position: absolute;
  right: 8px;
  display: flex;
  border: 0;
  background: transparent;
  color: var(--g-t3);
  cursor: pointer;
  padding: 4px;
}

.field__eye:hover {
  color: var(--g-t1);
}

.field__err {
  font-size: 11px;
  color: var(--g-danger);
}

.gauge {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-top: 2px;
}

.gauge__track {
  flex: 1;
  height: 4px;
  border-radius: 999px;
  background: var(--g-s2);
  overflow: hidden;
}

.gauge__fill {
  height: 100%;
  transition: width 0.18s cubic-bezier(0.2, 0.8, 0.3, 1);
}

.gauge__label {
  font-size: 10.5px;
  font-weight: 600;
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

.foot__hint {
  font-size: 11.5px;
  color: var(--g-t3);
}

.foot__hint code {
  font-family: var(--g-font-mono);
  font-size: 11px;
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

.err {
  margin: 14px 18px;
  font-size: 12.5px;
  color: var(--g-danger);
}

.muted {
  margin: 14px 18px;
  font-size: 12.5px;
  color: var(--g-t3);
}
</style>
