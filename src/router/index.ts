import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";

/**
 * Routes de l'application.
 * Chaque page est chargée en lazy pour garder le bundle initial léger.
 */
const routes: RouteRecordRaw[] = [
  {
    path: "/",
    redirect: "/servers",
  },
  {
    path: "/servers",
    name: "servers",
    component: () => import("@/pages/ServersPage.vue"),
    meta: { title: "Serveurs", icon: "pi pi-server" },
  },
  {
    path: "/keys",
    name: "keys",
    component: () => import("@/pages/KeysPage.vue"),
    meta: { title: "Clés SSH", icon: "pi pi-key" },
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("@/pages/SettingsPage.vue"),
    meta: { title: "Paramètres", icon: "pi pi-cog" },
  },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
