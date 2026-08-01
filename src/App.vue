<script setup lang="ts">
import AppLayout from "@/layouts/AppLayout.vue";
</script>

<template>
  <AppLayout />
</template>

<style>
:root {
  font-family:
    Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica,
    Arial, sans-serif;
  font-size: 15px;
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
  background: var(--p-content-background);
  color: var(--p-text-color);
}
</style>
