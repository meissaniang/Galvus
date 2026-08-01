import { open } from "@tauri-apps/plugin-dialog";

/**
 * Ouvre le sélecteur de fichier natif et retourne le chemin choisi (ou null).
 * Seul point d'accès au plugin dialog pour les composants de formulaire.
 */
export async function pickFile(title: string): Promise<string | null> {
  const selected = await open({ multiple: false, directory: false, title });
  return typeof selected === "string" ? selected : null;
}
