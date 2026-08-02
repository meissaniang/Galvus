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

  /** Enregistre le contenu modifié d'une clé (privée ou publique). */
  async function writeContent(
    name: string,
    kind: "private" | "public",
    content: string,
  ): Promise<void> {
    error.value = null;
    try {
      if (kind === "private") {
        await keysRepository.writePrivate(name, content);
      } else {
        await keysRepository.writePublic(name, content);
      }
      await load();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  /** Restaure les permissions 600 sur une clé privée. */
  async function fixPermissions(name: string): Promise<void> {
    error.value = null;
    try {
      await keysRepository.fixPermissions(name);
      await load();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  /** Ajoute, change ou retire la passphrase d'une clé. */
  async function changePassphrase(
    name: string,
    oldPassphrase: string,
    newPassphrase: string,
  ): Promise<void> {
    error.value = null;
    try {
      await keysRepository.changePassphrase(name, oldPassphrase, newPassphrase);
      await load();
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
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
    writeContent,
    changePassphrase,
    fixPermissions,
  };
});
