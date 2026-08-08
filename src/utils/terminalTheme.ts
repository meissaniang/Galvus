import type { ITheme } from "@xterm/xterm";

/**
 * Palette du terminal.
 *
 * Les seize couleurs ANSI doivent être définies explicitement : sans elles,
 * xterm retombe sur ses valeurs héritées du VGA (rouge `#cd0000`, bleu
 * `#0000ee`…), qui jurent avec le reste de l'interface et rendent la sortie de
 * `ls`, `git` ou `htop` agressive. Celles-ci sont désaturées et alignées sur
 * l'accent de l'application.
 */
export const TERMINAL_THEME: ITheme = {
  background: "#0B1017",
  foreground: "#D6E1EC",
  cursor: "#23C48A",
  cursorAccent: "#0B1017",
  selectionBackground: "rgba(35, 196, 138, 0.26)",
  selectionForeground: "#EEF4FA",
  selectionInactiveBackground: "rgba(154, 167, 184, 0.16)",

  black: "#1B2430",
  red: "#FF6B7F",
  green: "#23C48A",
  yellow: "#FFC46B",
  blue: "#5AA9FF",
  magenta: "#C98BFF",
  cyan: "#1FC7E8",
  white: "#C8D4E2",

  brightBlack: "#4A5A6E",
  brightRed: "#FF8A9B",
  brightGreen: "#4EE0A8",
  brightYellow: "#FFD591",
  brightBlue: "#85C2FF",
  brightMagenta: "#DCAAFF",
  brightCyan: "#6FE0F5",
  brightWhite: "#EEF4FA",
};
