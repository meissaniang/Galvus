import { createApp } from "vue";
import PrimeVue from "primevue/config";
import Aura from "@primevue/themes/aura";

import "primeicons/primeicons.css";
import "primeflex/primeflex.css";

import App from "./App.vue";

const app = createApp(App);

app.use(PrimeVue, {
  theme: {
    preset: Aura,
    options: {
      // La bascule dark/light sera pilotée à l'étape 4 via la classe `.app-dark`.
      darkModeSelector: ".app-dark",
    },
  },
});

app.mount("#app");
