import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { serversRepository } from "@/repositories/serversRepository";
import type { Server, ServerInput } from "@/types/ssh";

/**
 * Serveurs créés par l'utilisateur (base chiffrée), distincts des hôtes lus
 * depuis ~/.ssh/config (voir le store `servers`).
 */
export const useMyServersStore = defineStore("myServers", () => {
  const servers = ref<Server[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const query = ref("");

  const filtered = computed<Server[]>(() => {
    const q = query.value.trim().toLowerCase();
    if (!q) return servers.value;
    return servers.value.filter((s) =>
      [s.name, s.hostname, s.username]
        .filter((v): v is string => Boolean(v))
        .some((v) => v.toLowerCase().includes(q)),
    );
  });

  async function load(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      servers.value = await serversRepository.list();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  async function create(input: ServerInput): Promise<void> {
    await serversRepository.create(input);
    await load();
  }

  async function update(id: number, input: ServerInput): Promise<void> {
    await serversRepository.update(id, input);
    await load();
  }

  async function remove(id: number): Promise<void> {
    await serversRepository.remove(id);
    await load();
  }

  return { servers, loading, error, query, filtered, load, create, update, remove };
});
