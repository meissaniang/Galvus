import type { Server } from "@/types/ssh";

/**
 * Contrat de synchronisation (Phase 2). Défini dès maintenant pour que le
 * frontend n'ait AUCUNE modification à subir quand la couche réseau (NestJS +
 * JWT + PostgreSQL + Redis) sera branchée : il suffira de fournir une autre
 * implémentation de `SyncProvider`.
 */

/** Un snippet de commande réutilisable (préparé pour la sync). */
export interface Snippet {
  id: string;
  name: string;
  command: string;
  description?: string;
}

/** Instantané synchronisable de l'état local. */
export interface SyncableSnapshot {
  servers: Server[];
  snippets: Snippet[];
  preferences: Record<string, unknown>;
}

export type SyncStatus = "disabled" | "idle" | "syncing" | "error";

/** Fournisseur de synchronisation. */
export interface SyncProvider {
  readonly name: string;
  isEnabled(): boolean;
  status(): Promise<SyncStatus>;
  /** Récupère l'état distant. */
  pull(): Promise<SyncableSnapshot>;
  /** Pousse l'état local. */
  push(snapshot: SyncableSnapshot): Promise<void>;
}
