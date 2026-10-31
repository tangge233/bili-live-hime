<script setup lang="ts">
import { ref } from "vue";
import ManagerMuteUser from "@/components/manager/ManagerMuteUser.vue";
import ManagerMuteGlobal from "@/components/manager/ManagerMuteGlobal.vue";

type MuteTab = "list" | "global";

const tab = ref<MuteTab>("list");

const items = [
  { label: "禁言名单", value: "list" },
  { label: "全局禁言", value: "global" },
];
</script>

<template>
  <div class="space-y-3">
    <UTabs
      v-model="tab"
      :items="items"
      :content="false"
      variant="pill"
      size="sm"
      class="w-full"
      :ui="{ trigger: 'flex-1', label: 'text-[11px]' }" />

    <ManagerMuteUser v-if="tab === 'list'" />
    <ManagerMuteGlobal v-else />
  </div>
</template>
