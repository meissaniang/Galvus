import { createApp } from "vue";
import { createPinia } from "pinia";
import PrimeVue from "primevue/config";
import Aura from "@primevue/themes/aura";

import "primeicons/primeicons.css";
import "primeflex/primeflex.css";
import "@/assets/theme.css";

import App from "./App.vue";
import router from "@/router";
import { useThemeStore } from "@/stores/theme";

const app = createApp(App);

app.use(createPinia());
app.use(router);

app.use(PrimeVue, {
  theme: {
    preset: Aura,
    options: {
      // Bascule dark/light pilotée par la classe `.app-dark` (voir store theme).
      darkModeSelector: ".app-dark",
    },
  },
});

// Applique le thème persisté et écoute la préférence système avant le montage.
useThemeStore().init();

app.mount("#app");
