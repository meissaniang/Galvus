<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { storeToRefs } from "pinia";
import { useKeysStore } from "@/stores/keys";
import { keysRepository } from "@/repositories/keysRepository";
import KeyCard from "@/components/KeyCard.vue";
import type { SshKey } from "@/types/ssh";

/**
 * Écran Clés SSH — fidèle à « ScreenKeys.dc.html » : topbar titre + filtre +
 * Importer + Générer, grille 2 colonnes, carte-invite pointillée, dialog de
 * génération (segment de type, jauge de passphrase, état « Génération… »).
 */
const store = useKeysStore();
const { filteredKeys, keys, loading, error, query } = storeToRefs(store);

// --- Génération ---
type KeyType = "ed25519" | "rsa" | "ecdsa";
const genOpen = ref(false);
const gen = reactive({ name: "", keyType: "ed25519" as KeyType, comment: "", passphrase: "" });
const genSubmitted = ref(false);
const busy = ref(false);
const showPass = ref(false);

const KEY_TYPES: { value: KeyType; label: string }[] = [
  { value: "ed25519", label: "ED25519" },
  { value: "rsa", label: "RSA 4096" },
  { value: "ecdsa", label: "ECDSA" },
];

/** Robustesse de la passphrase : longueur + variété de caractères. */
const passStrength = computed(() => {
  const p = gen.passphrase;
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

function openGenerate(): void {
  gen.name = "";
  gen.keyType = "ed25519";
  gen.comment = "";
  gen.passphrase = "";
  genSubmitted.value = false;
  showPass.value = false;
  genOpen.value = true;
}

async function submitGenerate(): Promise<void> {
  genSubmitted.value = true;
  if (!gen.name.trim() || busy.value) return;
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

// --- Actions cartes ---
const notice = ref<string | null>(null);

async function removeKey(key: SshKey): Promise<void> {
  if (window.confirm(`Supprimer la clé « ${key.name} » (privée + publique) ?`)) {
    await store.remove(key.name);
  }
}

async function copyPublic(key: SshKey): Promise<void> {
  try {
    await store.copyPublic(key.name);
    notice.value = `Clé publique « ${key.name} » copiée`;
    setTimeout(() => (notice.value = null), 2500);
  } catch {
    /* erreur affichée via store.error */
  }
}

// --- Visualisation de la clé privée (locale uniquement) ---
const viewKey = ref<{ name: string; content: string } | null>(null);
const privCopied = ref(false);

async function viewPrivate(key: SshKey): Promise<void> {
  try {
    const content = await keysRepository.readPrivate(key.name);
    privCopied.value = false;
    viewKey.value = { name: key.name, content };
  } catch {
    /* erreur affichée via store.error */
  }
}

async function copyPrivate(): Promise<void> {
  if (!viewKey.value) return;
  await navigator.clipboard.writeText(viewKey.value.content);
  privCopied.value = true;
  setTimeout(() => (privCopied.value = false), 1500);
}

onMounted(() => {
  if (keys.value.length === 0) store.load();
});
</script>

<template>
  <section class="screen">
    <!-- Topbar -->
    <div class="topbar" data-tauri-drag-region>
      <div class="topbar__titles">
        <div class="topbar__title">Clés SSH</div>
        <div class="topbar__sub">
          {{ keys.length }} paire{{ keys.length > 1 ? "s" : "" }} détectée{{ keys.length > 1 ? "s" : "" }}
          dans ~/.ssh · jamais transmises
        </div>
      </div>
      <div class="filter">
        <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
          <circle cx="7" cy="7" r="4.4" stroke="currentColor" stroke-width="1.5" />
          <path d="M10.4 10.4L14 14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <input v-model="query" type="text" placeholder="Filtrer les clés" spellcheck="false" />
      </div>
      <button class="btn" @click="importKey">
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
          <path d="M7 9.6V2.6M4.2 6l2.8 2.8L9.8 6M2.4 11.4h9.2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        Importer
      </button>
      <button class="btn btn--primary" @click="openGenerate">
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
          <path d="M7 2.4v9.2M2.4 7h9.2" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" />
        </svg>
        Générer
      </button>
    </div>

    <!-- Contenu -->
    <div class="content">
      <p v-if="error" class="state state--error">{{ error }}</p>
      <p v-if="notice" class="state state--ok">{{ notice }}</p>
      <p v-if="loading && keys.length === 0" class="state">Scan de ~/.ssh…</p>

      <div class="grid">
        <KeyCard
          v-for="item in filteredKeys"
          :key="item.path"
          :key-item="item"
          @remove="removeKey"
          @copy-public="copyPublic"
          @view-private="viewPrivate"
        />

        <button class="invite" @click="openGenerate">
          <span class="invite__ava">
            <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
              <path d="M9 4v10M4 9h10" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" />
            </svg>
          </span>
          <span class="invite__title">Générer une nouvelle paire</span>
          <span class="invite__hint">
            ED25519 recommandé. La clé privée reste dans ~/.ssh avec les droits 600.
          </span>
        </button>
      </div>
    </div>

    <!-- Dialog clé privée (affichage local) -->
    <Transition name="dlg">
      <div v-if="viewKey" class="overlay" @click.self="viewKey = null">
        <div class="dialog dialog--wide" role="dialog" aria-modal="true">
          <header class="dialog__head">
            <div class="dialog__badge">
              <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
                <path d="M1.6 8s2.4-4 6.4-4 6.4 4 6.4 4-2.4 4-6.4 4S1.6 8 1.6 8z" stroke="currentColor" stroke-width="1.3" />
                <circle cx="8" cy="8" r="1.7" stroke="currentColor" stroke-width="1.3" />
              </svg>
            </div>
            <div class="dialog__titles">
              <div class="dialog__title">Clé privée — {{ viewKey.name }}</div>
              <div class="dialog__subtitle">~/.ssh/{{ viewKey.name }} · lecture locale, jamais transmise</div>
            </div>
            <button class="dialog__close" title="Fermer" @click="viewKey = null">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
              </svg>
            </button>
          </header>

          <div class="privwarn">
            <span class="privwarn__mark">!</span>
            Ne partagez jamais cette clé. Quiconque la possède peut se connecter à vos serveurs.
          </div>

          <pre class="privkey">{{ viewKey.content }}</pre>

          <footer class="dialog__foot">
            <button type="button" class="btn" @click="viewKey = null">Fermer</button>
            <button type="button" class="btn btn--primary" @click="copyPrivate">
              {{ privCopied ? "Copiée ✓" : "Copier" }}
            </button>
          </footer>
        </div>
      </div>
    </Transition>

    <!-- Dialog Générer -->
    <Transition name="dlg">
      <div v-if="genOpen" class="overlay" @click.self="!busy && (genOpen = false)">
        <div class="dialog" role="dialog" aria-modal="true">
          <header class="dialog__head">
            <div class="dialog__badge">
              <svg width="15" height="15" viewBox="0 0 18 18" fill="none">
                <circle cx="6.2" cy="6.2" r="3.4" stroke="currentColor" stroke-width="1.6" />
                <path d="M8.7 8.7L14.5 14.5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
              </svg>
            </div>
            <div class="dialog__titles">
              <div class="dialog__title">Générer une clé SSH</div>
              <div class="dialog__subtitle">ssh-keygen exécuté localement</div>
            </div>
          </header>

          <form class="dialog__body" @submit.prevent="submitGenerate">
            <div class="field">
              <label>Type</label>
              <div class="segmented">
                <button
                  v-for="t in KEY_TYPES"
                  :key="t.value"
                  type="button"
                  class="segmented__btn"
                  :class="{ 'segmented__btn--on': gen.keyType === t.value }"
                  @click="gen.keyType = t.value"
                >{{ t.label }}</button>
              </div>
            </div>

            <div class="field">
              <label>Nom du fichier</label>
              <div class="field__prefixed">
                <span class="field__prefix">~/.ssh/</span>
                <input v-model="gen.name" type="text" class="mono" placeholder="id_ed25519_prod" />
              </div>
              <span v-if="genSubmitted && !gen.name.trim()" class="field__err">Requis</span>
            </div>

            <div class="field">
              <label>Passphrase <span class="field__opt">(optionnelle)</span></label>
              <div class="field__pass">
                <input
                  v-model="gen.passphrase"
                  :type="showPass ? 'text' : 'password'"
                  class="mono"
                  placeholder="••••••••••"
                />
                <button type="button" class="field__eye" title="Afficher" @click="showPass = !showPass">
                  <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                    <path d="M1.6 8s2.4-4 6.4-4 6.4 4 6.4 4-2.4 4-6.4 4S1.6 8 1.6 8z" stroke="currentColor" stroke-width="1.3" />
                    <circle cx="8" cy="8" r="1.7" stroke="currentColor" stroke-width="1.3" />
                  </svg>
                </button>
              </div>
              <div v-if="passStrength" class="gauge">
                <div class="gauge__track">
                  <div
                    class="gauge__fill"
                    :style="{ width: passStrength.pct + '%', background: passStrength.color }"
                  />
                </div>
                <span class="gauge__label" :style="{ color: passStrength.color }">
                  {{ passStrength.label }}
                </span>
              </div>
            </div>

            <div class="field">
              <label>Commentaire</label>
              <input v-model="gen.comment" type="text" class="mono" placeholder="ops@galvus.dev" />
            </div>
          </form>

          <footer class="dialog__foot">
            <button type="button" class="btn" :disabled="busy" @click="genOpen = false">Annuler</button>
            <button type="button" class="btn btn--primary" :disabled="busy" @click="submitGenerate">
              <span v-if="busy" class="spinner" />
              {{ busy ? "Génération…" : "Générer" }}
            </button>
          </footer>
        </div>
      </div>
    </Transition>
  </section>
</template>

<style scoped>
.screen {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--g-s0);
}

/* ---------- Topbar ---------- */
.topbar {
  height: 56px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 20px;
  border-bottom: 1px solid var(--g-border);
  flex-shrink: 0;
}

.topbar__titles {
  flex: 1;
  min-width: 0;
}

.topbar__title {
  font-size: 15px;
  font-weight: 600;
  color: var(--g-t1);
}

.topbar__sub {
  font-size: 11.5px;
  color: var(--g-t3);
  margin-top: 1px;
}

.filter {
  display: flex;
  align-items: center;
  gap: 9px;
  height: 34px;
  padding: 0 11px;
  border-radius: 10px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  width: 230px;
  color: var(--g-t3);
  transition: border-color 0.12s ease-out, box-shadow 0.12s ease-out;
}

.filter:focus-within {
  border-color: var(--g-accent);
  box-shadow: 0 0 0 3px var(--g-accent-ring);
}

.filter input {
  flex: 1;
  min-width: 0;
  border: 0;
  background: transparent;
  font-family: inherit;
  font-size: 13px;
  color: var(--g-t1);
  outline: none;
}

.filter input::placeholder {
  color: var(--g-t3);
}

.btn {
  height: 34px;
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 0 13px;
  border-radius: 10px;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  font-family: inherit;
  font-size: 13px;
  font-weight: 500;
  color: var(--g-t1);
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.12s ease;
}

.btn:hover {
  background: var(--g-s2);
}

.btn:disabled {
  opacity: 0.7;
  cursor: default;
}

.btn--primary {
  padding: 0 14px;
  border: 0;
  background: var(--g-accent);
  color: var(--g-accent-fg);
  font-weight: 600;
  box-shadow: var(--g-sh1);
}

.btn--primary:hover {
  background: var(--g-accent-h);
}

/* ---------- Contenu ---------- */
.content {
  flex: 1;
  overflow-y: auto;
  padding: 18px 20px 24px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  align-content: start;
}

@media (max-width: 1000px) {
  .grid {
    grid-template-columns: 1fr;
  }
}

.invite {
  border: 1px dashed var(--g-border-2);
  border-radius: 12px;
  padding: 22px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 9px;
  text-align: center;
  background: transparent;
  cursor: pointer;
  font-family: inherit;
  transition: border-color 0.12s ease;
}

.invite:hover {
  border-color: var(--g-accent-ring);
}

.invite__ava {
  width: 38px;
  height: 38px;
  border-radius: 11px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  color: var(--g-t3);
  display: flex;
  align-items: center;
  justify-content: center;
}

.invite:hover .invite__ava {
  color: var(--g-accent);
}

.invite__title {
  font-size: 13px;
  font-weight: 600;
  color: var(--g-t1);
}

.invite__hint {
  font-size: 11.5px;
  color: var(--g-t3);
  max-width: 260px;
}

.state {
  font-size: 12.5px;
  color: var(--g-t3);
  margin: 0 0 10px;
}

.state--error {
  color: var(--g-danger);
}

.state--ok {
  color: var(--g-success);
}

/* ---------- Dialog ---------- */
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
  width: 470px;
  max-width: 100%;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  border-radius: 14px;
  box-shadow: var(--g-sh3);
  overflow: hidden;
}

.dialog--wide {
  width: 600px;
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

.privwarn {
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

.privwarn__mark {
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

.privkey {
  margin: 12px 18px 16px;
  padding: 12px;
  max-height: 300px;
  overflow: auto;
  border-radius: 10px;
  background: var(--g-s0);
  border: 1px solid var(--g-border);
  font-family: var(--g-font-mono);
  font-size: 11px;
  line-height: 1.55;
  color: var(--g-t2);
  white-space: pre-wrap;
  word-break: break-all;
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

.dialog__body {
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 13px;
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

.field input::placeholder {
  color: var(--g-t3);
}

.field__err {
  font-size: 11px;
  color: var(--g-danger);
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
  transition: background 0.12s ease, color 0.12s ease;
}

.segmented__btn--on {
  background: var(--g-s1);
  color: var(--g-t1);
  font-weight: 600;
  box-shadow: var(--g-sh1);
}

.field__prefixed {
  position: relative;
  display: flex;
  align-items: center;
}

.field__prefix {
  position: absolute;
  left: 11px;
  font-family: var(--g-font-mono);
  font-size: 12.5px;
  color: var(--g-t3);
  pointer-events: none;
}

.field__prefixed input {
  padding-left: 66px;
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
  justify-content: flex-end;
  gap: 9px;
  padding: 13px 18px;
  border-top: 1px solid var(--g-border);
  background: var(--g-s0);
}

.dialog__foot .btn {
  height: 32px;
  font-size: 12.5px;
}

.spinner {
  width: 12px;
  height: 12px;
  border-radius: 999px;
  border: 2px solid currentColor;
  border-top-color: transparent;
  opacity: 0.8;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
