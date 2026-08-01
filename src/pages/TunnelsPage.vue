<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { storeToRefs } from "pinia";
import { useTunnelsStore } from "@/stores/tunnels";
import type { TunnelInput, TunnelKind } from "@/types/ssh";

const store = useTunnelsStore();
const { tunnels, error } = storeToRefs(store);

const form = reactive<TunnelInput>({
  name: "",
  kind: "local",
  sshTarget: "",
  listenPort: 8080,
  targetHost: "localhost",
  targetPort: 80,
});
const submitted = ref(false);

function needsTarget(kind: TunnelKind): boolean {
  return kind !== "dynamic";
}

async function submit(): Promise<void> {
  submitted.value = true;
  if (!form.name.trim() || !form.sshTarget.trim() || !form.listenPort) return;
  await store.create({
    name: form.name.trim(),
    kind: form.kind,
    sshTarget: form.sshTarget.trim(),
    listenPort: Number(form.listenPort),
    targetHost: needsTarget(form.kind) ? form.targetHost?.trim() || null : null,
    targetPort: needsTarget(form.kind) ? Number(form.targetPort) || null : null,
  });
  submitted.value = false;
  form.name = "";
}

function describe(kind: TunnelKind): string {
  return { local: "Local -L", remote: "Remote -R", dynamic: "Dynamic -D (SOCKS)" }[kind];
}

onMounted(() => store.load());
</script>

<template>
  <section class="page">
    <header class="page__header">
      <h1><i class="pi pi-arrow-right-arrow-left" /> Tunnels</h1>
      <p>Redirections de port SSH (Local / Remote / Dynamic). Auth par clé ou agent.</p>
    </header>

    <form class="tform" @submit.prevent="submit">
      <input v-model="form.name" class="in" placeholder="Nom *" />
      <select v-model="form.kind" class="in">
        <option value="local">Local (-L)</option>
        <option value="remote">Remote (-R)</option>
        <option value="dynamic">Dynamic (-D / SOCKS)</option>
      </select>
      <input v-model="form.sshTarget" class="in" placeholder="Hôte SSH (alias ou user@host) *" />
      <input v-model.number="form.listenPort" class="in in--port" type="number" placeholder="Port local *" />
      <template v-if="needsTarget(form.kind)">
        <input v-model="form.targetHost" class="in" placeholder="Hôte cible" />
        <input v-model.number="form.targetPort" class="in in--port" type="number" placeholder="Port cible" />
      </template>
      <button class="btn btn--primary" type="submit"><i class="pi pi-plus" /> Ajouter</button>
    </form>

    <p v-if="error" class="state state--error"><i class="pi pi-exclamation-triangle" /> {{ error }}</p>

    <div v-if="tunnels.length === 0" class="state state--empty">
      <i class="pi pi-arrow-right-arrow-left" />
      <p>Aucun tunnel. Crée-en un ci-dessus.</p>
    </div>

    <ul v-else class="list">
      <li v-for="t in tunnels" :key="t.id" class="row">
        <span class="dot" :class="{ 'dot--on': store.isRunning(t.id) }" />
        <div class="row__body">
          <div class="row__name">{{ t.name }} <span class="chip">{{ describe(t.kind) }}</span></div>
          <div class="row__detail">
            <template v-if="t.kind === 'dynamic'">
              SOCKS sur <code>localhost:{{ t.listenPort }}</code> via {{ t.sshTarget }}
            </template>
            <template v-else>
              <code>localhost:{{ t.listenPort }}</code> → <code>{{ t.targetHost }}:{{ t.targetPort }}</code>
              via {{ t.sshTarget }}
            </template>
          </div>
        </div>
        <button
          v-if="!store.isRunning(t.id)"
          class="btn btn--primary"
          @click="store.start(t.id)"
        >
          <i class="pi pi-play" /> Démarrer
        </button>
        <button v-else class="btn" @click="store.stop(t.id)">
          <i class="pi pi-stop" /> Arrêter
        </button>
        <button class="btn btn--danger" @click="store.remove(t.id)">
          <i class="pi pi-trash" />
        </button>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.page {
  height: 100%;
  overflow-y: auto;
  padding: 18px 20px 24px;
  box-sizing: border-box;
}
.page__header h1 {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin: 0 0 0.35rem;
  font-size: 1.6rem;
}
.page__header p {
  margin: 0 0 1.5rem;
  color: var(--p-text-muted-color);
}

.tform {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: 1.5rem;
}
.in {
  padding: 0.55rem 0.7rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 9px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  font: inherit;
  outline: none;
}
.in:focus {
  border-color: var(--p-primary-color);
}
.in--port {
  width: 120px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.5rem 0.85rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 9px;
  background: var(--p-content-background);
  color: var(--p-text-color);
  font: inherit;
  cursor: pointer;
}
.btn:hover {
  background: var(--p-content-hover-background);
}
.btn--primary {
  border-color: var(--p-primary-color);
  background: var(--p-primary-color);
  color: var(--p-primary-contrast-color, #fff);
}
.btn--danger:hover {
  border-color: color-mix(in srgb, #ef4444 50%, transparent);
  color: #ef4444;
}

.list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.75rem 1rem;
  border: 1px solid var(--p-content-border-color);
  border-radius: 11px;
  background: var(--p-content-background);
}
.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--p-text-muted-color);
  opacity: 0.4;
  flex-shrink: 0;
}
.dot--on {
  background: #22c55e;
  opacity: 1;
  box-shadow: 0 0 0 3px color-mix(in srgb, #22c55e 25%, transparent);
}
.row__body {
  flex: 1;
  min-width: 0;
}
.row__name {
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.row__detail {
  font-size: 0.82rem;
  color: var(--p-text-muted-color);
}
.chip {
  padding: 0.05rem 0.4rem;
  border-radius: 6px;
  background: var(--p-content-hover-background);
  color: var(--p-text-muted-color);
  font-size: 0.72rem;
  font-weight: 500;
}
code {
  padding: 0.05rem 0.3rem;
  border-radius: 4px;
  background: var(--p-content-hover-background);
  font-size: 0.85em;
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
</style>
