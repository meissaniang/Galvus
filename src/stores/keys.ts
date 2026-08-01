import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { keysRepository } from "@/repositories/keysRepository";
import type { SshKey } from "@/types/ssh";

export const useKeysStore = defineStore("keys", () => {
  const keys = ref<SshKey[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const query = ref("");

  const filteredKeys = computed<SshKey[]>(() => {
    const q = query.value.trim().toLowerCase();
    if (!q) {
      return keys.value;
    }
    return keys.value.filter((k) =>
      [k.name, k.keyType, k.comment, k.fingerprint]
        .filter((v): v is string => Boolean(v))
        .some((v) => v.toLowerCase().includes(q)),
    );
  });

  async function load(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      keys.value = await keysRepository.list();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  return { keys, loading, error, query, filteredKeys, load };
});
