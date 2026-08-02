/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  // Shim des composants monofichiers : les props et l'état réels sont inférés
  // par vue-tsc à partir du `<script setup>`, ce type n'est qu'un repli.
  const component: DefineComponent<
    Record<string, unknown>,
    Record<string, unknown>,
    unknown
  >;
  export default component;
}
