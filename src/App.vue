<script setup lang="ts">
import { zh_cn } from "@nuxt/ui/locale";
import AppContent from "@/components/AppContent.vue";
</script>

<template>
  <UApp :locale="zh_cn" :toaster="{ position: 'top-center', duration: 3200 }">
    <AppContent />
  </UApp>
</template>
