import { invoke } from "@tauri-apps/api/core";
import type { Tunnel, TunnelInput } from "@/types/ssh";

/** Accès aux tunnels SSH. Unique point d'appel des commandes Tauri `tunnel_*`. */
export const tunnelsRepository = {
  list(): Promise<Tunnel[]> {
    return invoke<Tunnel[]>("tunnel_list");
  },
  create(input: TunnelInput): Promise<Tunnel> {
    return invoke<Tunnel>("tunnel_create", { input });
  },
  remove(id: number): Promise<void> {
    return invoke<void>("tunnel_delete", { id });
  },
  start(id: number): Promise<void> {
    return invoke<void>("tunnel_start", { id });
  },
  stop(id: number): Promise<void> {
    return invoke<void>("tunnel_stop", { id });
  },
  running(): Promise<number[]> {
    return invoke<number[]>("tunnel_running");
  },
};
