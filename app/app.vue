<script setup lang="ts">
onMounted(() => {
  if (typeof window === 'undefined') return
  if (import.meta.env.DEV) return
  if (!('serviceWorker' in navigator)) return
  void navigator.serviceWorker.register('/sw.js')
})
</script>

<template>
  <NuxtRouteAnnouncer />
  <NuxtPage />
</template>
