<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { storeToRefs } from "pinia";
import { useTunnelsStore } from "@/stores/tunnels";
import { useMyServersStore } from "@/stores/myServers";
import { useServersStore } from "@/stores/servers";
import type { Tunnel, TunnelInput, TunnelKind } from "@/types/ssh";

/**
 * Écran Tunnels — fidèle à « ScreenTunnels.dc.html » : liste dense avec voyant
 * pulsé, badge de type L/R/D, mapping de ports avec flèche accent, filtre
 * Tous/Actifs, panneau latéral de création (300px) avec commande équivalente
 * et bannière d'erreur quand un tunnel échoue.
 */
const store = useTunnelsStore();
const myServers = useMyServersStore();
const hostsStore = useServersStore();

const { tunnels, lastError, error } = storeToRefs(store);
const { servers } = storeToRefs(myServers);
const { hosts } = storeToRefs(hostsStore);

type Filter = "all" | "active";
const filter = ref<Filter>("all");
const panelOpen = ref(false);

const runningCount = computed(() => store.runningIds.length);
const stoppedCount = computed(() => tunnels.value.length - runningCount.value);

const subtitle = computed(() => {
  const parts = [
    `${runningCount.value} actif${runningCount.value > 1 ? "s" : ""}`,
    `${stoppedCount.value} arrêté${stoppedCount.value > 1 ? "s" : ""}`,
  ];
  if (lastError.value) parts.push("1 en erreur");
  return parts.join(" · ");
});

const visibleTunnels = computed(() =>
  filter.value === "active"
    ? tunnels.value.filter((t) => store.isRunning(t.id))
    : tunnels.value,
);

/** Cibles SSH proposées : serveurs enregistrés + hôtes du config. */
const sshTargets = computed(() => {
  const fromServers = servers.value.map((s) => ({
    label: s.name,
    value: s.username ? `${s.username}@${s.hostname}` : s.hostname,
  }));
  const fromHosts = hosts.value.map((h) => ({ label: h.alias, value: h.alias }));
  return [...fromServers, ...fromHosts];
});

// --- Formulaire panneau ---
const form = reactive({
  name: "",
  kind: "local" as TunnelKind,
  sshTarget: "",
  listenPort: 15432,
  targetHost: "localhost",
  targetPort: 5432,
  autostart: true,
});
const submitted = ref(false);

const KINDS: { value: TunnelKind; label: string }[] = [
  { value: "local", label: "Local L" },
  { value: "remote", label: "Remote R" },
  { value: "dynamic", label: "Dyn. D" },
];

const needsTarget = computed(() => form.kind !== "dynamic");

/** Commande ssh équivalente, affichée en direct. */
const equivalentCommand = computed(() => {
  const target = form.sshTarget || "<serveur>";
  if (form.kind === "dynamic") return `ssh -N -D ${form.listenPort} ${target}`;
  const flag = form.kind === "local" ? "-L" : "-R";
  return `ssh -N ${flag} ${form.listenPort}:${form.targetHost || "localhost"}:${form.targetPort} ${target}`;
});

function openPanel(): void {
  submitted.value = false;
  panelOpen.value = true;
}

async function submit(): Promise<void> {
  submitted.value = true;
  if (!form.name.trim() || !form.sshTarget.trim() || !form.listenPort) return;
  await store.create({
    name: form.name.trim(),
    kind: form.kind,
    sshTarget: form.sshTarget.trim(),
    listenPort: Number(form.listenPort),
    targetHost: needsTarget.value ? form.targetHost.trim() || "localhost" : null,
    targetPort: needsTarget.value ? Number(form.targetPort) || null : null,
  } satisfies TunnelInput);
  if (form.autostart) {
    const created = tunnels.value.find(
      (t) => t.name === form.name.trim() && t.listenPort === Number(form.listenPort),
    );
    if (created) await store.start(created.id);
  }
  form.name = "";
  submitted.value = false;
  panelOpen.value = false;
}

function mapping(t: Tunnel): { left: string; right: string | null } {
  if (t.kind === "dynamic")
    return { left: `socks5 · 127.0.0.1:${t.listenPort}`, right: null };
  if (t.kind === "remote")
    return { left: `distant:${t.listenPort}`, right: `${t.targetHost}:${t.targetPort}` };
  return { left: `127.0.0.1:${t.listenPort}`, right: `${t.targetHost}:${t.targetPort}` };
}

function kindBadgeClass(t: Tunnel): string {
  if (lastError.value?.id === t.id) return "tb--danger";
  if (t.kind === "local") return store.isRunning(t.id) ? "tb--accent" : "tb--muted";
  if (t.kind === "dynamic") return "tb--info";
  return "tb--muted";
}

async function removeTunnel(t: Tunnel): Promise<void> {
  if (window.confirm(`Supprimer le tunnel « ${t.name} » ?`)) {
    await store.remove(t.id);
  }
}

const errorTunnelName = computed(
  () => tunnels.value.find((t) => t.id === lastError.value?.id)?.name ?? "",
);

onMounted(() => {
  store.load();
  if (servers.value.length === 0) myServers.load();
  if (hosts.value.length === 0) hostsStore.load();
});
</script>

<template>
  <section class="screen">
    <!-- Topbar -->
    <div class="topbar" data-galvus-drag>
      <div class="topbar__titles">
        <div class="topbar__title">Tunnels</div>
        <div class="topbar__sub">{{ subtitle }}</div>
      </div>
      <div class="segmented">
        <button
          class="segmented__btn"
          :class="{ 'segmented__btn--on': filter === 'all' }"
          @click="filter = 'all'"
        >
          Tous
        </button>
        <button
          class="segmented__btn"
          :class="{ 'segmented__btn--on': filter === 'active' }"
          @click="filter = 'active'"
        >
          Actifs
        </button>
      </div>
      <button class="newbtn" @click="openPanel">
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none">
          <path
            d="M7 2.4v9.2M2.4 7h9.2"
            stroke="currentColor"
            stroke-width="1.9"
            stroke-linecap="round"
          />
        </svg>
        Nouveau tunnel
      </button>
    </div>

    <div class="body">
      <!-- Liste -->
      <div class="list">
        <p v-if="error" class="state state--error">{{ error }}</p>

        <div v-if="visibleTunnels.length > 0" class="cols">
          <span></span><span>Type</span><span>Nom / serveur</span><span>Mapping</span
          ><span></span>
        </div>

        <div
          v-for="t in visibleTunnels"
          :key="t.id"
          class="trow"
          :class="{
            'trow--off': !store.isRunning(t.id) && lastError?.id !== t.id,
            'trow--error': lastError?.id === t.id,
          }"
        >
          <span
            class="trow__dot"
            :class="{
              'trow__dot--on': store.isRunning(t.id),
              'trow__dot--error': lastError?.id === t.id,
            }"
          />
          <span class="tb" :class="kindBadgeClass(t)">
            {{ t.kind === "local" ? "L" : t.kind === "remote" ? "R" : "D" }}
          </span>
          <span class="trow__names">
            <span class="trow__name">{{ t.name }}</span>
            <span v-if="lastError?.id === t.id" class="trow__target trow__target--error">
              {{ lastError.message }}
            </span>
            <span v-else class="trow__target">{{ t.sshTarget }}</span>
          </span>
          <span class="trow__map">
            {{ mapping(t).left }}
            <template v-if="mapping(t).right">
              <span
                class="trow__arrow"
                :class="{ 'trow__arrow--error': lastError?.id === t.id }"
                >→</span
              >
              {{ mapping(t).right }}
            </template>
          </span>
          <span class="trow__actions">
            <button v-if="store.isRunning(t.id)" class="act" @click="store.stop(t.id)">
              Arrêter
            </button>
            <button
              v-else-if="lastError?.id === t.id"
              class="act act--retry"
              @click="store.start(t.id)"
            >
              Réessayer
            </button>
            <button v-else class="act act--primary" @click="store.start(t.id)">
              Démarrer
            </button>
            <button class="act-icon" title="Supprimer" @click="removeTunnel(t)">
              <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
                <path
                  d="M2.6 4.4h8.8M5.4 4.4V3.2h3.2v1.2M4 4.4l.6 6.6h4.8L10 4.4"
                  stroke="currentColor"
                  stroke-width="1.3"
                  stroke-linecap="round"
                />
              </svg>
            </button>
          </span>
        </div>

        <!-- Bannière d'erreur -->
        <div v-if="lastError" class="banner">
          <div class="banner__mark">!</div>
          <div class="banner__body">
            <div class="banner__title">Échec du tunnel {{ errorTunnelName }}</div>
            <div class="banner__detail">{{ lastError.message }}</div>
          </div>
          <button class="banner__dismiss" @click="store.dismissError()">Fermer</button>
        </div>

        <div v-if="visibleTunnels.length === 0" class="empty">
          <p v-if="filter === 'active'">Aucun tunnel actif.</p>
          <p v-else>Aucun tunnel. Clique sur « Nouveau tunnel » pour commencer.</p>
        </div>
      </div>

      <!-- Panneau latéral de création -->
      <Transition name="panel">
        <aside v-if="panelOpen" class="panel">
          <div>
            <div class="panel__title">Nouveau tunnel</div>
            <div class="panel__sub">Redirection via le binaire ssh système</div>
          </div>

          <div class="field">
            <label>Type de redirection</label>
            <div class="segmented segmented--full">
              <button
                v-for="k in KINDS"
                :key="k.value"
                class="segmented__btn"
                :class="{ 'segmented__btn--on': form.kind === k.value }"
                @click="form.kind = k.value"
              >
                {{ k.label }}
              </button>
            </div>
          </div>

          <div class="field">
            <label>Nom</label>
            <input v-model="form.name" type="text" placeholder="pg-replica" />
            <span v-if="submitted && !form.name.trim()" class="field__err">Requis</span>
          </div>

          <div class="field">
            <label>Serveur</label>
            <select v-model="form.sshTarget" class="select">
              <option value="" disabled>Choisir…</option>
              <option v-for="t in sshTargets" :key="t.label" :value="t.value">
                {{ t.label }}
              </option>
            </select>
            <span v-if="submitted && !form.sshTarget.trim()" class="field__err"
              >Requis</span
            >
          </div>

          <div class="pair">
            <div class="field">
              <label>Port local</label>
              <input
                v-model.number="form.listenPort"
                type="number"
                class="mono"
                min="1"
                max="65535"
              />
            </div>
            <div v-if="needsTarget" class="field">
              <label>Port distant</label>
              <input
                v-model.number="form.targetPort"
                type="number"
                class="mono"
                min="1"
                max="65535"
              />
            </div>
          </div>

          <div v-if="needsTarget" class="field">
            <label>Hôte distant</label>
            <input
              v-model="form.targetHost"
              type="text"
              class="mono"
              placeholder="localhost"
            />
          </div>

          <label class="autostart">
            <span>Démarrer automatiquement</span>
            <button
              type="button"
              class="toggle"
              :class="{ 'toggle--on': form.autostart }"
              role="switch"
              :aria-checked="form.autostart"
              @click="form.autostart = !form.autostart"
            >
              <span class="toggle__knob" />
            </button>
          </label>

          <div class="cmd">
            <div class="cmd__label">Commande équivalente</div>
            <div class="cmd__text">{{ equivalentCommand }}</div>
          </div>

          <div class="panel__spacer" />

          <div class="panel__cta">
            <button class="pbtn" @click="panelOpen = false">Annuler</button>
            <button class="pbtn pbtn--primary" @click="submit">
              {{ form.autostart ? "Créer et démarrer" : "Créer" }}
            </button>
          </div>
        </aside>
      </Transition>
    </div>
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

.segmented {
  display: flex;
  padding: 3px;
  gap: 3px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  border-radius: 9px;
}

.segmented--full {
  border-radius: 10px;
}

.segmented__btn {
  height: 26px;
  padding: 0 11px;
  border: 0;
  border-radius: 6px;
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

.segmented--full .segmented__btn {
  flex: 1;
  height: 28px;
  border-radius: 7px;
  font-size: 11.5px;
}

.segmented__btn--on {
  background: var(--g-s0);
  color: var(--g-t1);
  font-weight: 600;
  box-shadow: var(--g-sh1);
}

.segmented--full .segmented__btn--on {
  background: var(--g-s1);
}

.newbtn {
  height: 34px;
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 0 14px;
  border: 0;
  border-radius: 10px;
  background: var(--g-accent);
  color: var(--g-accent-fg);
  font-family: inherit;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: var(--g-sh1);
  white-space: nowrap;
}

.newbtn:hover {
  background: var(--g-accent-h);
}

.body {
  flex: 1;
  display: flex;
  min-height: 0;
}

.list {
  flex: 1;
  min-width: 0;
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 9px;
  overflow-y: auto;
}

.cols {
  display: grid;
  grid-template-columns: 24px 34px minmax(140px, 1fr) minmax(0, 216px) 118px;
  gap: 10px;
  padding: 0 14px 2px;
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--g-t3);
}

.trow {
  display: grid;
  grid-template-columns: 24px 34px minmax(140px, 1fr) minmax(0, 216px) 118px;
  gap: 10px;
  align-items: center;
  padding: 12px 14px;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  border-radius: 12px;
  box-shadow: var(--g-sh1);
}

.trow--off {
  opacity: 0.78;
  box-shadow: none;
}

.trow--error {
  border-color: var(--g-danger);
}

.trow__dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: var(--g-t3);
  margin-left: 6px;
}

.trow__dot--on {
  background: var(--g-success);
  animation: g-pulse 2s ease-out infinite;
}

.trow__dot--error {
  background: var(--g-danger);
}

@keyframes g-pulse {
  0% {
    box-shadow: 0 0 0 0 var(--g-accent-ring);
  }
  70% {
    box-shadow: 0 0 0 6px rgba(0, 0, 0, 0);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(0, 0, 0, 0);
  }
}

.tb {
  font-family: var(--g-font-mono);
  font-size: 11px;
  font-weight: 700;
  padding: 3px 0;
  border-radius: 6px;
  text-align: center;
}

.tb--accent {
  color: var(--g-accent);
  background: var(--g-accent-soft);
}

.tb--info {
  color: var(--g-info);
  background: var(--g-s2);
  border: 1px solid var(--g-border);
}

.tb--muted {
  color: var(--g-t2);
  background: var(--g-s2);
  border: 1px solid var(--g-border);
}

.tb--danger {
  color: var(--g-danger);
  background: var(--g-danger-soft);
}

.trow__names {
  min-width: 0;
}

.trow__name {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--g-t1);
}

.trow__target {
  display: block;
  font-family: var(--g-font-mono);
  font-size: 11px;
  color: var(--g-t3);
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.trow__target--error {
  color: var(--g-danger);
}

.trow__map {
  font-family: var(--g-font-mono);
  font-size: 11px;
  color: var(--g-t2);
  display: flex;
  align-items: center;
  gap: 5px;
  flex-wrap: wrap;
  min-width: 0;
}

.trow__arrow {
  color: var(--g-accent);
}

.trow__arrow--error {
  color: var(--g-danger);
}

.trow__actions {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
}

.act {
  height: 28px;
  padding: 0 11px;
  border-radius: 8px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  font-family: inherit;
  font-size: 11.5px;
  font-weight: 600;
  color: var(--g-t1);
  cursor: pointer;
}

.act:hover {
  background: var(--g-s3);
}

.act--primary {
  border: 0;
  background: var(--g-accent);
  color: var(--g-accent-fg);
}

.act--primary:hover {
  background: var(--g-accent-h);
}

.act--retry {
  background: var(--g-danger-soft);
  border-color: var(--g-danger);
  color: var(--g-danger);
}

.act-icon {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  background: var(--g-s2);
  border: 1px solid var(--g-border);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--g-t2);
  cursor: pointer;
}

.act-icon:hover {
  background: var(--g-danger-soft);
  border-color: var(--g-danger);
  color: var(--g-danger);
}

.banner {
  margin-top: 6px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 11px 14px;
  background: var(--g-danger-soft);
  border: 1px solid var(--g-danger);
  border-radius: 11px;
}

.banner__mark {
  width: 24px;
  height: 24px;
  border-radius: 7px;
  background: var(--g-danger);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 700;
  flex-shrink: 0;
}

.banner__body {
  flex: 1;
  min-width: 0;
}

.banner__title {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--g-t1);
}

.banner__detail {
  font-family: var(--g-font-mono);
  font-size: 11px;
  color: var(--g-t2);
  margin-top: 2px;
}

.banner__dismiss {
  border: 0;
  background: transparent;
  font-family: inherit;
  font-size: 11.5px;
  font-weight: 600;
  color: var(--g-danger);
  cursor: pointer;
}

.empty {
  margin-top: 16px;
  padding: 40px;
  border: 1px dashed var(--g-border-2);
  border-radius: 12px;
  text-align: center;
  color: var(--g-t3);
  font-size: 13px;
}

.state--error {
  color: var(--g-danger);
  font-size: 12.5px;
}

/* ---------- Panneau latéral ---------- */
.panel {
  width: 300px;
  flex-shrink: 0;
  border-left: 1px solid var(--g-border);
  background: var(--g-sidebar);
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 13px;
  overflow-y: auto;
}

.panel-enter-active {
  transition:
    transform 0.24s cubic-bezier(0.2, 0.8, 0.3, 1),
    opacity 0.24s ease-out;
}

.panel-enter-from {
  transform: translateX(24px);
  opacity: 0;
}

.panel-leave-active {
  transition:
    transform 0.18s ease-in,
    opacity 0.18s ease-in;
}

.panel-leave-to {
  transform: translateX(24px);
  opacity: 0;
}

.panel__title {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--g-t1);
}

.panel__sub {
  font-size: 11.5px;
  color: var(--g-t3);
  margin-top: 2px;
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

.field input,
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
  transition:
    border-color 0.12s ease-out,
    box-shadow 0.12s ease-out;
}

.field input.mono {
  font-family: var(--g-font-mono);
}

.field input:focus,
.select:focus {
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

.pair {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.autostart {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: var(--g-s1);
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

.cmd {
  background: var(--g-s0);
  border: 1px solid var(--g-border);
  border-radius: 10px;
  padding: 10px 11px;
}

.cmd__label {
  font-size: 10.5px;
  font-weight: 600;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  color: var(--g-t3);
  margin-bottom: 5px;
}

.cmd__text {
  font-family: var(--g-font-mono);
  font-size: 11px;
  color: var(--g-t2);
  line-height: 1.5;
  word-break: break-all;
}

.panel__spacer {
  flex: 1;
}

.panel__cta {
  display: flex;
  gap: 9px;
}

.pbtn {
  flex: 1;
  height: 34px;
  border-radius: 9px;
  background: var(--g-s1);
  border: 1px solid var(--g-border);
  font-family: inherit;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--g-t2);
  cursor: pointer;
}

.pbtn:hover {
  color: var(--g-t1);
  background: var(--g-s2);
}

.pbtn--primary {
  border: 0;
  background: var(--g-accent);
  color: var(--g-accent-fg);
  font-weight: 600;
  box-shadow: var(--g-sh1);
}

.pbtn--primary:hover {
  background: var(--g-accent-h);
  color: var(--g-accent-fg);
}
</style>
