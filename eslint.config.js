import js from "@eslint/js";
import pluginVue from "eslint-plugin-vue";
import vueTsConfig from "@vue/eslint-config-typescript";
import skipFormatting from "@vue/eslint-config-prettier/skip-formatting";

/**
 * Configuration ESLint (format plat).
 * Le formatage est délégué à Prettier : ESLint ne traite que la correction.
 */
export default [
  {
    ignores: ["dist/**", "src-tauri/**", "node_modules/**", "design/**"],
  },
  js.configs.recommended,
  ...pluginVue.configs["flat/recommended"],
  ...vueTsConfig(),
  skipFormatting,
  {
    rules: {
      // Les composants d'écran sont volontairement nommés d'un seul mot
      // (ServersPage, KeysPage…) : la règle multi-mots n'apporte rien ici.
      "vue/multi-word-component-names": "off",
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
    },
  },
];
