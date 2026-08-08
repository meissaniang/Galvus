import type { OsId } from "./osIcons";

/**
 * Reconnaissance du système distant à partir de la bannière de connexion.
 *
 * OpenSSH affiche le contenu de `/etc/motd` et de `/etc/issue.net` juste après
 * l'authentification : « Welcome to Ubuntu 22.04.5 LTS », « Debian GNU/Linux
 * 12 »… C'est gratuit, et cela évite d'exécuter une commande à l'insu de
 * l'utilisateur sur sa machine. En contrepartie ce n'est qu'une heuristique :
 * une bannière peut être désactivée ou personnalisée, d'où la possibilité de
 * corriger le système à la main dans la fiche du serveur.
 */

/** L'ordre compte : du plus spécifique au plus générique. */
const SIGNATURES: [OsId, RegExp][] = [
  ["mint", /\blinux mint\b/i],
  ["kali", /\bkali (gnu\/)?linux\b/i],
  ["raspbian", /\braspbian\b|\braspberry pi os\b/i],
  ["proxmox", /\bproxmox\b/i],
  ["ubuntu", /\bubuntu\b/i],
  ["debian", /\bdebian\b/i],
  ["alma", /\balmalinux\b/i],
  ["rocky", /\brocky linux\b/i],
  ["centos", /\bcentos\b/i],
  ["rhel", /\bred hat enterprise linux\b|\brhel\b/i],
  ["fedora", /\bfedora\b/i],
  ["opensuse", /\bopensuse\b|\bsuse linux\b/i],
  ["arch", /\barch linux\b/i],
  ["manjaro", /\bmanjaro\b/i],
  ["gentoo", /\bgentoo\b/i],
  ["alpine", /\balpine linux\b/i],
  ["freebsd", /\bfreebsd\b/i],
  ["macos", /\bmacos\b|\bdarwin\b/i],
  // Dernier recours : on sait que c'est Linux, sans savoir laquelle.
  ["linux", /\bgnu\/linux\b|\blinux\b/i],
];

/** Retire les séquences ANSI, qui hachent les mots des bannières colorées. */
// eslint-disable-next-line no-control-regex
const ANSI = /\x1b\[[0-9;?]*[a-zA-Z]|\x1b\][^\x07\x1b]*(\x07|\x1b\\)/g;

export function detectOs(banner: string): OsId | null {
  const text = banner.replace(ANSI, " ");
  for (const [id, pattern] of SIGNATURES) {
    if (pattern.test(text)) return id;
  }
  return null;
}

/**
 * Accumulateur de bannière.
 *
 * Ne conserve que le début de la session : au-delà, ce n'est plus la bannière
 * mais la sortie des commandes de l'utilisateur, où « ubuntu » peut apparaître
 * pour mille raisons sans rien dire du système hôte.
 */
export class BannerCollector {
  private buffer = "";
  private readonly decoder = new TextDecoder();
  private done = false;

  constructor(private readonly limit = 8192) {}

  /** Retourne le système dès qu'il est reconnu, puis `null` définitivement. */
  push(chunk: Uint8Array): OsId | null {
    if (this.done) return null;

    this.buffer += this.decoder.decode(chunk, { stream: true });
    const found = detectOs(this.buffer);
    if (found) {
      this.stop();
      return found;
    }
    if (this.buffer.length >= this.limit) this.stop();
    return null;
  }

  /** Libère le tampon : plus rien ne sera analysé. */
  stop(): void {
    this.done = true;
    this.buffer = "";
  }
}
