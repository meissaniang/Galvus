<script setup lang="ts">
import AppLayout from "@/layouts/AppLayout.vue";
</script>

<template>
  <AppLayout />
</template>

<style>
:root {
  font-family: var(--g-font-ui);
  font-size: 14px;
  line-height: 1.5;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

* {
  box-sizing: border-box;
}

html,
body,
#app {
  margin: 0;
  padding: 0;
  height: 100%;
}

body {
  background: var(--g-app);
  color: var(--g-t1);
}
</style>
