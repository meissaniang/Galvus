<script setup lang="ts">
import { reactive, ref, watch } from "vue";
import type { ConfigHostInput, Host } from "@/types/ssh";
import { pickFile } from "@/services/filePicker";

/**
 * Dialog d'édition d'une entrée `Host` du `~/.ssh/config`.
 * Même gabarit que le dialog serveur (540px, radius 14, grille 2 colonnes) ;
 * l'enregistrement réécrit le bloc dans le fichier, avec sauvegarde `.bak`.
 */
const props = defineProps<{ open: boolean; host: Host | null }>();
const emit = defineEmits<{ save: [input: ConfigHostInput]; close: [] }>();

const form = reactive<ConfigHostInput>({
  alias: "",
  hostname: null,
  user: null,
  port: 22,
  identityFile: null,
  proxyJump: null,
});
const submitted = ref(false);

watch(
  () => props.open,
  (open) => {
    if (!open || !props.host) return;
    submitted.value = false;
    Object.assign(form, {
      alias: props.host.alias,
      hostname: props.host.hostname,
      user: props.host.user,
      port: props.host.port ?? 22,
      identityFile: props.host.identityFile,
      proxyJump: props.host.proxyJump,
    });
  },
);

async function browseKey(): Promise<void> {
  const path = await pickFile("Choisir un fichier de clé privée");
  if (path) form.identityFile = path;
}

function submit(): void {
  submitted.value = true;
  const alias = form.alias.trim();
  if (!alias || alias.includes(" ")) return;
  emit("save", {
    alias,
    hostname: form.hostname?.trim() || null,
    user: form.user?.trim() || null,
    port: Number(form.port) || 22,
    identityFile: form.identityFile?.trim() || null,
    proxyJump: form.proxyJump?.trim() || null,
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
            <div class="dialog__title">Éditer l'hôte — {{ host?.alias }}</div>
            <div class="dialog__subtitle">
              Écrit dans ~/.ssh/config · sauvegarde config.bak
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
            <label>Alias (Host)</label>
            <input
              v-model="form.alias"
              type="text"
              class="mono"
              placeholder="vps-meissa-1"
            />
            <span v-if="submitted && !form.alias.trim()" class="field__err">Requis</span>
            <span v-else-if="submitted && form.alias.includes(' ')" class="field__err">
              Un seul mot, sans espace
            </span>
          </div>

          <div class="field">
            <label>HostName</label>
            <input
              v-model="form.hostname"
              type="text"
              class="mono"
              placeholder="145.241.165.236"
            />
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
            <label>User</label>
            <input v-model="form.user" type="text" class="mono" placeholder="meissa" />
          </div>

          <div class="field">
            <label>ProxyJump</label>
            <input
              v-model="form.proxyJump"
              type="text"
              class="mono"
              placeholder="bastion"
            />
          </div>

          <div class="field field--wide">
            <label>IdentityFile</label>
            <div class="field__file">
              <input
                v-model="form.identityFile"
                type="text"
                class="mono"
                placeholder="~/.ssh/ma-cle"
              />
              <button type="button" class="field__browse" @click="browseKey">
                Parcourir
              </button>
            </div>
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
            Le reste du fichier est préservé
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
}

.field label {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--g-t2);
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
  transition:
    border-color 0.12s ease-out,
    box-shadow 0.12s ease-out;
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
