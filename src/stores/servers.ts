import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { hostsRepository } from "@/repositories/hostsRepository";
import type { ConfigHostInput, Host } from "@/types/ssh";

export const useServersStore = defineStore("servers", () => {
  const hosts = ref<Host[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  /** Filtre de recherche (alias, hostname, user). */
  const query = ref("");

  const filteredHosts = computed<Host[]>(() => {
    const q = query.value.trim().toLowerCase();
    if (!q) {
      return hosts.value;
    }
    return hosts.value.filter((h) =>
      [h.alias, h.hostname, h.user]
        .filter((v): v is string => Boolean(v))
        .some((v) => v.toLowerCase().includes(q)),
    );
  });

  async function load(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      hosts.value = await hostsRepository.list();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  /** Met à jour une entrée du ~/.ssh/config puis recharge la liste. */
  async function update(alias: string, input: ConfigHostInput): Promise<void> {
    error.value = null;
    try {
      await hostsRepository.update(alias, input);
      await load();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  /** Supprime une entrée du ~/.ssh/config puis recharge la liste. */
  async function remove(alias: string): Promise<void> {
    error.value = null;
    try {
      await hostsRepository.remove(alias);
      await load();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  return { hosts, loading, error, query, filteredHosts, load, update, remove };
});
