import type {
  SyncProvider,
  SyncStatus,
  SyncableSnapshot,
} from "@/types/sync";

/**
 * Fournisseur par défaut : aucune synchronisation (mode 100 % hors ligne).
 * Sera remplacé en Phase 2 par une implémentation réseau, sans toucher au reste
 * du frontend.
 */
export class LocalOnlySyncProvider implements SyncProvider {
  readonly name = "local-only";

  isEnabled(): boolean {
    return false;
  }

  async status(): Promise<SyncStatus> {
    return "disabled";
  }

  async pull(): Promise<SyncableSnapshot> {
    throw new Error("Synchronisation non disponible (mode hors ligne).");
  }

  async push(): Promise<void> {
    throw new Error("Synchronisation non disponible (mode hors ligne).");
  }
}

/** Fournisseur de sync actif (injectable en Phase 2). */
export const syncProvider = new LocalOnlySyncProvider();
