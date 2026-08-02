<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useThemeStore, type ThemeMode } from "@/stores/theme";
import { useSettingsStore, type Accent } from "@/stores/settings";
import { shortcut } from "@/utils/platform";

/**
 * Écran Paramètres — fidèle à « ScreenSettings.dc.html » : sous-navigation
 * 196px, thème en 3 cartes-aperçu, couleur d'accent, réglages terminal avec
 * aperçu ANSI live, toast « Préférences enregistrées ».
 */
const theme = useThemeStore();
const settings = useSettingsStore();
const { savedAt } = storeToRefs(settings);

type Section = "apparence" | "terminal" | "raccourcis" | "securite" | "avance";
const section = ref<Section>("apparence");

const SECTIONS: { id: Section; label: string }[] = [
  { id: "apparence", label: "Apparence" },
  { id: "terminal", label: "Terminal" },
  { id: "raccourcis", label: "Raccourcis" },
  { id: "securite", label: "Sécurité" },
  { id: "avance", label: "Avancé" },
];

const THEMES: { value: ThemeMode; label: string; side: string; main: string }[] = [
  { value: "system", label: "Système", side: "#10161F", main: "#F5F8FB" },
  { value: "light", label: "Clair", side: "#EAEFF5", main: "#F5F8FB" },
  { value: "dark", label: "Sombre", side: "#10161F", main: "#161D27" },
];

const ACCENTS: { value: Accent; color: string }[] = [
  { value: "emerald", color: "#23C48A" },
  { value: "cyan", color: "#1FC7E8" },
];

const FONTS = [
  {
    label: "JetBrains Mono",
    value: '"JetBrains Mono", "SF Mono", ui-monospace, Menlo, Consolas, monospace',
  },
  {
    label: "SF Mono / système",
    value: 'ui-monospace, "SF Mono", SFMono-Regular, Menlo, Consolas, monospace',
  },
  { label: "Menlo", value: "Menlo, monospace" },
  { label: "Courier", value: '"Courier New", Courier, monospace' },
];

const SHORTCUTS = [
  { keys: shortcut("K"), action: "Rechercher un serveur" },
  { keys: shortcut("T"), action: "Ouvrir l'espace terminal" },
  { keys: shortcut("D"), action: "Split vertical du pane actif" },
  { keys: shortcut("\u21e7D"), action: "Split horizontal du pane actif" },
  { keys: shortcut("W"), action: "Fermer le pane actif" },
];

// Toast « Préférences enregistrées » (apparaît à chaque sauvegarde).
const toastVisible = ref(false);
let toastTimer: ReturnType<typeof setTimeout> | null = null;
watch(savedAt, () => {
  toastVisible.value = true;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (toastVisible.value = false), 2500);
});

const previewStyle = computed(() => ({
  fontFamily: settings.terminalFontFamily,
  fontSize: settings.terminalFontSize + "px",
  lineHeight: String(settings.terminalLineHeight),
  fontVariantLigatures: settings.terminalLigatures ? "normal" : "none",
}));
</script>

<template>
  <section class="screen">
    <!-- Sous-navigation -->
    <aside class="subnav">
      <div class="subnav__label">Paramètres</div>
      <button
        v-for="s in SECTIONS"
        :key="s.id"
        class="subnav__item"
        :class="{ 'subnav__item--on': section === s.id }"
        @click="section = s.id"
      >
        {{ s.label }}
      </button>
      <div class="subnav__spacer" />
      <div class="subnav__version">
        <div class="subnav__vname">Galvus 0.1.0</div>
        <div class="subnav__vsub">100 % local</div>
      </div>
    </aside>

    <!-- Contenu -->
    <div class="content">
      <!-- ===== Apparence ===== -->
      <template v-if="section === 'apparence'">
        <div class="head">
          <div class="head__title">Apparence</div>
          <div class="head__sub">Thème de l'interface et rendu du terminal</div>
        </div>

        <div class="card">
          <div class="card__title">Thème</div>
          <div class="card__sub">
            Bascule instantanée, transition des couleurs en 160 ms
          </div>
          <div class="themes">
            <button
              v-for="t in THEMES"
              :key="t.value"
              class="themecard"
              :class="{ 'themecard--on': theme.mode === t.value }"
              @click="theme.setMode(t.value)"
            >
              <span class="themecard__preview">
                <span class="themecard__side" :style="{ background: t.side }" />
                <span class="themecard__main" :style="{ background: t.main }" />
              </span>
              <span class="themecard__row">
                <span class="radio" :class="{ 'radio--on': theme.mode === t.value }">
                  <span v-if="theme.mode === t.value" class="radio__dot" />
                </span>
                <span
                  class="themecard__label"
                  :class="{ 'themecard__label--on': theme.mode === t.value }"
                >
                  {{ t.label }}
                </span>
              </span>
            </button>
          </div>
          <div class="accentrow">
            <span class="accentrow__label">Couleur d'accent</span>
            <button
              v-for="a in ACCENTS"
              :key="a.value"
              class="accentsw"
              :class="{ 'accentsw--on': settings.accent === a.value }"
              :style="{ background: a.color, '--sw': a.color }"
              :title="a.value === 'emerald' ? 'Vert Galvus' : 'Cyan électrique'"
              @click="settings.accent = a.value"
            />
          </div>
        </div>
      </template>

      <!-- ===== Terminal ===== -->
      <template v-if="section === 'apparence' || section === 'terminal'">
        <div v-if="section === 'terminal'" class="head">
          <div class="head__title">Terminal</div>
          <div class="head__sub">Police, taille et interlignage</div>
        </div>

        <div class="card card--split">
          <div class="tset">
            <div v-if="section === 'apparence'">
              <div class="card__title">Terminal</div>
              <div class="card__sub">Police, taille et interlignage</div>
            </div>

            <div class="field">
              <label>Police</label>
              <select v-model="settings.terminalFontFamily" class="select mono">
                <option v-for="f in FONTS" :key="f.label" :value="f.value">
                  {{ f.label }}
                </option>
              </select>
            </div>

            <div class="field">
              <div class="field__row">
                <label>Taille</label>
                <span class="chipval">{{ settings.terminalFontSize }} px</span>
              </div>
              <input
                v-model.number="settings.terminalFontSize"
                type="range"
                min="10"
                max="22"
                step="1"
                class="slider"
              />
            </div>

            <div class="field">
              <div class="field__row">
                <label>Interlignage</label>
                <span class="chipval">{{ settings.terminalLineHeight.toFixed(1) }}</span>
              </div>
              <input
                v-model.number="settings.terminalLineHeight"
                type="range"
                min="1"
                max="2"
                step="0.1"
                class="slider"
              />
            </div>

            <label class="switchrow">
              <span>Ligatures</span>
              <button
                type="button"
                class="toggle"
                :class="{ 'toggle--on': settings.terminalLigatures }"
                role="switch"
                :aria-checked="settings.terminalLigatures"
                @click="settings.terminalLigatures = !settings.terminalLigatures"
              >
                <span class="toggle__knob" />
              </button>
            </label>

            <label class="switchrow">
              <span>Curseur clignotant</span>
              <button
                type="button"
                class="toggle"
                :class="{ 'toggle--on': settings.terminalCursorBlink }"
                role="switch"
                :aria-checked="settings.terminalCursorBlink"
                @click="settings.terminalCursorBlink = !settings.terminalCursorBlink"
              >
                <span class="toggle__knob" />
              </button>
            </label>
          </div>

          <div class="tpreview">
            <div class="tpreview__label">Aperçu</div>
            <div class="tpreview__term" :style="previewStyle">
              <div>
                <span class="c-green">deploy@web-01</span><span class="c-dim">:</span
                ><span class="c-blue">~/app</span><span class="c-gray">$ </span>git status
                -sb
              </div>
              <div><span class="c-violet">## main...origin/main</span></div>
              <div><span class="c-green"> M </span>src/components/ServerCard.vue</div>
              <div><span class="c-amber">?? </span>src/assets/theme.css</div>
              <div class="c-dim">→ ligatures : != &gt;= =&gt; ---</div>
              <div class="tpreview__swatches">
                <span style="background: #f0565f" /><span
                  style="background: #e9a23b"
                /><span style="background: #23c48a" /><span
                  style="background: #22b8d9"
                /><span style="background: #4c8dff" /><span
                  style="background: #a96cf5"
                /><span style="background: #d6e1ec" />
              </div>
            </div>
          </div>
        </div>
      </template>

      <!-- ===== Raccourcis ===== -->
      <template v-if="section === 'raccourcis'">
        <div class="head">
          <div class="head__title">Raccourcis</div>
          <div class="head__sub">Raccourcis clavier globaux de l'application</div>
        </div>
        <div class="card">
          <div v-for="s in SHORTCUTS" :key="s.keys" class="shortcut">
            <span class="shortcut__keys">{{ s.keys }}</span>
            <span class="shortcut__action">{{ s.action }}</span>
          </div>
        </div>
      </template>

      <!-- ===== Sécurité ===== -->
      <template v-if="section === 'securite'">
        <div class="head">
          <div class="head__title">Sécurité</div>
          <div class="head__sub">Comment Galvus protège vos données</div>
        </div>
        <div class="card">
          <div class="sec">
            <span class="sec__title">Base locale chiffrée</span>
            <span class="sec__detail"
              >Serveurs et tunnels stockés dans SQLite chiffrée (SQLCipher,
              AES-256).</span
            >
          </div>
          <div class="sec">
            <span class="sec__title">Clé dans le trousseau natif</span>
            <span class="sec__detail"
              >La clé de chiffrement vit dans le Trousseau macOS — jamais sur disque en
              clair.</span
            >
          </div>
          <div class="sec">
            <span class="sec__title">SSH natif</span>
            <span class="sec__detail"
              >Connexions via le binaire OpenSSH du système. Aucune réimplémentation du
              protocole.</span
            >
          </div>
          <div class="sec">
            <span class="sec__title">100 % local</span>
            <span class="sec__detail"
              >Aucune donnée ne quitte votre machine. Pas de télémétrie.</span
            >
          </div>
        </div>
      </template>

      <!-- ===== Avancé ===== -->
      <template v-if="section === 'avance'">
        <div class="head">
          <div class="head__title">Avancé</div>
          <div class="head__sub">Maintenance et réinitialisation</div>
        </div>
        <div class="card">
          <div class="sec">
            <span class="sec__title">Journaux</span>
            <span class="sec__detail"
              >Fichiers de logs avec rotation (5 Mo) dans le dossier de logs de
              l'application.</span
            >
          </div>
          <div class="advrow">
            <div>
              <div class="sec__title">Réinitialiser les réglages du terminal</div>
              <div class="sec__detail">
                Police, taille, interlignage et curseur reviennent aux valeurs par défaut.
              </div>
            </div>
            <button class="resetbtn" @click="settings.reset()">Réinitialiser</button>
          </div>
        </div>
      </template>

      <!-- Toast sauvegarde -->
      <Transition name="toast">
        <div v-if="toastVisible" class="toast">
          <div class="toast__mark">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path
                d="M3.2 7.2l2.6 2.6 5-5.6"
                stroke="currentColor"
                stroke-width="1.9"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
          </div>
          <div class="toast__body">
            <div class="toast__title">Préférences enregistrées</div>
            <div class="toast__sub">stockées localement sur cette machine</div>
          </div>
          <span class="toast__time">à l'instant</span>
        </div>
      </Transition>
    </div>
  </section>
</template>

<style scoped>
.screen {
  display: flex;
  height: 100%;
  min-height: 0;
  background: var(--g-s0);
}

/* ---------- Sous-navigation ---------- */
.subnav {
  width: 196px;
  flex-shrink: 0;
  border-right: 1px solid var(--g-border);
  padding: 18px 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.subnav__label {
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--g-t3);
  padding: 0 10px 8px;
}

.subnav__item {
  height: 32px;
  display: flex;
  align-items: center;
  padding: 0 10px;
  border: 0;
  border-radius: 9px;
  background: transparent;
  font-family: inherit;
  font-size: 12.5px;
  color: var(--g-t2);
  cursor: pointer;
  text-align: left;
  transition:
    background 0.12s ease,
    color 0.12s ease;
}

.subnav__item:hover {
  background: var(--g-s2);
  color: var(--g-t1);
}

.subnav__item--on {
  background: var(--g-s2);
  color: var(--g-t1);
  font-weight: 600;
  box-shadow: var(--g-sh1);
}

.subnav__spacer {
  flex: 1;
}

.subnav__version {
  padding: 10px;
  border-radius: 10px;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
}

.subnav__vname {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--g-t1);
}

.subnav__vsub {
  font-size: 11px;
  color: var(--g-t3);
  margin-top: 2px;
}

/* ---------- Contenu ---------- */
.content {
  flex: 1;
  min-width: 0;
  padding: 20px 22px;
  display: flex;
  flex-direction: column;
  gap: 18px;
  overflow-y: auto;
  position: relative;
}

.head__title {
  font-size: 16px;
  font-weight: 600;
  color: var(--g-t1);
}

.head__sub {
  font-size: 12px;
  color: var(--g-t3);
  margin-top: 2px;
}

.card {
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--g-sh1);
}

.card__title {
  font-size: 13px;
  font-weight: 600;
  color: var(--g-t1);
  margin-bottom: 3px;
}

.card__sub {
  font-size: 11.5px;
  color: var(--g-t3);
  margin-bottom: 13px;
}

/* Cartes-aperçu de thème */
.themes {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 11px;
}

.themecard {
  border: 1px solid var(--g-border);
  border-radius: 11px;
  padding: 9px;
  background: var(--g-s0);
  cursor: pointer;
  font-family: inherit;
  text-align: left;
  transition:
    border-color 0.14s ease,
    box-shadow 0.14s ease,
    background 0.14s ease;
}

.themecard--on {
  border-color: var(--g-accent);
  background: var(--g-accent-soft);
  box-shadow: 0 0 0 3px var(--g-accent-ring);
}

.themecard__preview {
  height: 64px;
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  border: 1px solid var(--g-border);
}

.themecard__side {
  width: 32%;
}

.themecard__main {
  flex: 1;
}

.themecard__row {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-top: 9px;
}

.radio {
  width: 14px;
  height: 14px;
  border-radius: 999px;
  border: 1.5px solid var(--g-border-2);
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
}

.radio--on {
  border-color: var(--g-accent);
  background: var(--g-accent);
}

.radio__dot {
  width: 5px;
  height: 5px;
  border-radius: 999px;
  background: var(--g-accent-fg);
}

.themecard__label {
  font-size: 12.5px;
  color: var(--g-t2);
}

.themecard__label--on {
  font-weight: 600;
  color: var(--g-t1);
}

.accentrow {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-top: 14px;
  padding-top: 14px;
  border-top: 1px solid var(--g-border);
}

.accentrow__label {
  font-size: 12.5px;
  color: var(--g-t1);
  flex: 1;
}

.accentsw {
  width: 22px;
  height: 22px;
  border-radius: 7px;
  border: 0;
  cursor: pointer;
  padding: 0;
}

.accentsw--on {
  box-shadow:
    0 0 0 2px var(--g-s1),
    0 0 0 4px var(--sw);
}

/* Réglages terminal */
.card--split {
  display: flex;
  gap: 18px;
}

.tset {
  width: 300px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 13px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field label {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--g-t2);
}

.field__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.chipval {
  font-family: var(--g-font-mono);
  font-size: 11.5px;
  color: var(--g-t1);
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  padding: 2px 7px;
  border-radius: 6px;
}

.select {
  height: 34px;
  border-radius: 9px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  padding: 0 11px;
  font-family: inherit;
  font-size: 12.5px;
  color: var(--g-t1);
  outline: none;
  width: 100%;
  box-sizing: border-box;
  cursor: pointer;
}

.select.mono {
  font-family: var(--g-font-mono);
}

.select:focus {
  border-color: var(--g-accent);
  box-shadow: 0 0 0 3px var(--g-accent-ring);
}

/* Slider aligné sur le DS (piste 4px, poignée 16px bordure accent). */
.slider {
  appearance: none;
  width: 100%;
  height: 4px;
  border-radius: 999px;
  background: var(--g-s3);
  outline: none;
  accent-color: var(--g-accent);
}

.slider::-webkit-slider-thumb {
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: var(--g-s1);
  border: 2px solid var(--g-accent);
  box-shadow: var(--g-sh1);
  cursor: pointer;
}

.switchrow {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 9px 11px;
  background: var(--g-s0);
  border: 1px solid var(--g-border);
  border-radius: 10px;
  font-size: 12.5px;
  color: var(--g-t1);
}

.toggle {
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

.toggle--on {
  background: var(--g-accent);
  justify-content: flex-end;
}

.toggle__knob {
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: #fff;
  box-shadow: var(--g-sh1);
}

.tpreview {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tpreview__label {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--g-t2);
}

.tpreview__term {
  flex: 1;
  border-radius: 11px;
  background: var(--g-term-bg);
  border: 1px solid var(--g-border);
  padding: 13px;
  color: var(--g-term-fg);
  overflow: hidden;
}

.c-green {
  color: #23c48a;
}
.c-blue {
  color: #4c8dff;
}
.c-violet {
  color: #a96cf5;
}
.c-amber {
  color: #e9a23b;
}
.c-gray {
  color: #9aa7b8;
}
.c-dim {
  color: #6b7a8d;
}

.tpreview__swatches {
  margin-top: 6px;
  display: flex;
  gap: 6px;
}

.tpreview__swatches span {
  width: 13px;
  height: 13px;
  border-radius: 3px;
}

/* Raccourcis / Sécurité / Avancé */
.shortcut {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 9px 0;
  border-bottom: 1px solid var(--g-border);
}

.shortcut:last-child {
  border-bottom: 0;
}

.shortcut__keys {
  font-family: var(--g-font-mono);
  font-size: 11.5px;
  color: var(--g-t1);
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  padding: 3px 8px;
  border-radius: 6px;
  min-width: 46px;
  text-align: center;
}

.shortcut__action {
  font-size: 12.5px;
  color: var(--g-t2);
}

.sec {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 9px 0;
  border-bottom: 1px solid var(--g-border);
}

.sec:last-child {
  border-bottom: 0;
}

.sec__title {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--g-t1);
}

.sec__detail {
  font-size: 11.5px;
  color: var(--g-t3);
}

.advrow {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 0 4px;
}

.resetbtn {
  height: 32px;
  padding: 0 14px;
  border-radius: 9px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--g-t1);
  cursor: pointer;
  white-space: nowrap;
}

.resetbtn:hover {
  background: var(--g-s3);
}

/* Toast */
.toast {
  position: sticky;
  bottom: 0;
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 12px 14px;
  border-radius: 12px;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  box-shadow: var(--g-sh2);
  max-width: 420px;
}

.toast-enter-active {
  transition:
    transform 0.18s cubic-bezier(0.2, 0.8, 0.3, 1),
    opacity 0.18s ease-out;
}

.toast-enter-from {
  transform: translateY(10px);
  opacity: 0;
}

.toast-leave-active {
  transition: opacity 0.14s ease-in;
}

.toast-leave-to {
  opacity: 0;
}

.toast__mark {
  width: 26px;
  height: 26px;
  border-radius: 8px;
  background: var(--g-accent);
  color: var(--g-accent-fg);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.toast__body {
  flex: 1;
}

.toast__title {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--g-t1);
}

.toast__sub {
  font-size: 11px;
  color: var(--g-t3);
  margin-top: 1px;
}

.toast__time {
  font-size: 11px;
  color: var(--g-t3);
}
</style>
