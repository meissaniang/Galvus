import type { ITheme } from "@xterm/xterm";

/**
 * Palette du terminal — Material Dark.
 *
 * Les seize couleurs ANSI doivent être définies explicitement : sans elles,
 * xterm retombe sur ses valeurs héritées du VGA (rouge `#cd0000`, bleu
 * `#0000ee`…), qui rendent la sortie de `ls`, `git` ou `htop` agressive.
 *
 * Le schéma est repris tel quel, sans réharmonisation avec l'accent de
 * l'application : c'est une palette connue, et l'intérêt d'en choisir une
 * connue est justement qu'elle soit reconnaissable. Conformément à l'original,
 * les teintes vives sont identiques aux normales — seuls le noir et le blanc
 * ont une variante claire.
 */
export const MATERIAL_DARK: ITheme = {
  background: "#212121",
  foreground: "#EEFFFF",
  cursor: "#FFCC00",
  cursorAccent: "#212121",
  selectionBackground: "rgba(97, 97, 97, 0.5)",
  selectionInactiveBackground: "rgba(97, 97, 97, 0.28)",

  black: "#000000",
  red: "#FF5370",
  green: "#C3E88D",
  yellow: "#FFCB6B",
  blue: "#82AAFF",
  magenta: "#C792EA",
  cyan: "#89DDFF",
  white: "#EEFFFF",

  brightBlack: "#545454",
  brightRed: "#FF5370",
  brightGreen: "#C3E88D",
  brightYellow: "#FFCB6B",
  brightBlue: "#82AAFF",
  brightMagenta: "#C792EA",
  brightCyan: "#89DDFF",
  brightWhite: "#FFFFFF",
};
