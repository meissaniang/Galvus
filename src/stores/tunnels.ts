import { ref } from "vue";
import { defineStore } from "pinia";
import { tunnelsRepository } from "@/repositories/tunnelsRepository";
import type { Tunnel, TunnelInput } from "@/types/ssh";

/** Dernière erreur de démarrage d'un tunnel (affichée en ligne + bannière). */
export interface TunnelError {
  id: number;
  message: string;
}

export const useTunnelsStore = defineStore("tunnels", () => {
  const tunnels = ref<Tunnel[]>([]);
  const runningIds = ref<number[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const lastError = ref<TunnelError | null>(null);

  function isRunning(id: number): boolean {
    return runningIds.value.includes(id);
  }

  async function refreshRunning(): Promise<void> {
    try {
      runningIds.value = await tunnelsRepository.running();
    } catch {
      /* ignore */
    }
  }

  async function load(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      tunnels.value = await tunnelsRepository.list();
      await refreshRunning();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  async function create(input: TunnelInput): Promise<void> {
    await tunnelsRepository.create(input);
    await load();
  }

  async function remove(id: number): Promise<void> {
    await tunnelsRepository.remove(id);
    await load();
  }

  async function start(id: number): Promise<void> {
    error.value = null;
    lastError.value = null;
    try {
      await tunnelsRepository.start(id);
    } catch (e) {
      lastError.value = { id, message: e instanceof Error ? e.message : String(e) };
    }
    await refreshRunning();
    // Un tunnel qui meurt aussitôt (port occupé, auth refusée…) est signalé.
    setTimeout(async () => {
      await refreshRunning();
      if (!runningIds.value.includes(id) && !lastError.value) {
        lastError.value = {
          id,
          message: "le tunnel s'est arrêté immédiatement — port local déjà utilisé ?",
        };
      }
    }, 900);
  }

  async function stop(id: number): Promise<void> {
    if (lastError.value?.id === id) lastError.value = null;
    await tunnelsRepository.stop(id);
    await refreshRunning();
  }

  function dismissError(): void {
    lastError.value = null;
  }

  return {
    tunnels,
    runningIds,
    loading,
    error,
    lastError,
    isRunning,
    load,
    create,
    remove,
    start,
    stop,
    refreshRunning,
    dismissError,
  };
});
