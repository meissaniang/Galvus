import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { keysRepository, type GenerateKeyInput } from "@/repositories/keysRepository";
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

  async function generate(input: GenerateKeyInput): Promise<void> {
    error.value = null;
    try {
      await keysRepository.generate(input);
      await load();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function importFrom(source: string, name: string): Promise<void> {
    error.value = null;
    try {
      await keysRepository.import(source, name);
      await load();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function remove(name: string): Promise<void> {
    await keysRepository.remove(name);
    await load();
  }

  /** Copie la clé publique dans le presse-papiers. */
  async function copyPublic(name: string): Promise<void> {
    const content = await keysRepository.readPublic(name);
    await navigator.clipboard.writeText(content);
  }

  return {
    keys,
    loading,
    error,
    query,
    filteredKeys,
    load,
    generate,
    importFrom,
    remove,
    copyPublic,
  };
});
