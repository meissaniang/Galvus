import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** Fragment de sortie d'une session (octets bruts). */
export interface TerminalOutput {
  sessionId: string;
  data: number[];
}

/** Fin d'une session. */
export interface TerminalExit {
  sessionId: string;
}

/**
 * Accès au terminal SSH côté Rust. Unique point d'appel des commandes Tauri et
 * d'écoute des events terminal (aucun composant n'appelle `invoke`/`listen`).
 */
export const terminalRepository = {
  open(sessionId: string, host: string, cols: number, rows: number): Promise<void> {
    return invoke("terminal_open", { sessionId, host, cols, rows });
  },

  write(sessionId: string, data: string): Promise<void> {
    return invoke("terminal_write", { sessionId, data });
  },

  resize(sessionId: string, cols: number, rows: number): Promise<void> {
    return invoke("terminal_resize", { sessionId, cols, rows });
  },

  close(sessionId: string): Promise<void> {
    return invoke("terminal_close", { sessionId });
  },

  onOutput(callback: (payload: TerminalOutput) => void): Promise<UnlistenFn> {
    return listen<TerminalOutput>("terminal://output", (event) =>
      callback(event.payload),
    );
  },

  onExit(callback: (payload: TerminalExit) => void): Promise<UnlistenFn> {
    return listen<TerminalExit>("terminal://exit", (event) => callback(event.payload));
  },
};
