<script setup lang="ts">
import { ref } from "vue";
import SlideoverHeader from "@/components/SlideoverHeader.vue";
import ManagerAdmins from "@/components/manager/ManagerAdmins.vue";
import ManagerMute from "@/components/manager/ManagerMute.vue";
import ManagerBlockedWords from "@/components/manager/ManagerBlockedWords.vue";

type Section = "admin" | "mute" | "blocked";

const open = defineModel<boolean>("open", { default: false });

const section = ref<Section>("admin");

const items = [
  { label: "房管", icon: "i-lucide-crown", value: "admin" },
  { label: "禁言", icon: "i-lucide-user-x", value: "mute" },
  { label: "屏蔽词", icon: "i-lucide-message-square-off", value: "blocked" },
];
</script>

<template>
  <USlideover
    v-model:open="open"
    :close="false"
    title="房间管理"
    description="房管、禁言与屏蔽词"
    :ui="{
      content: 'w-full max-w-none',
      body: 'space-y-3 p-3',
    }">
    <template #header="{ close }">
      <SlideoverHeader
        title="房间管理"
        description="房管、禁言与屏蔽词"
        @back="close" />
    </template>

    <template #body>
      <UTabs
        v-model="section"
        :items="items"
        :content="false"
        variant="pill"
        size="sm"
        class="w-full"
        :ui="{ trigger: 'flex-1 px-1', label: 'text-[10px]' }" />

      <ManagerAdmins v-if="section === 'admin'" />
      <ManagerMute v-else-if="section === 'mute'" />
      <ManagerBlockedWords v-else />
    </template>
  </USlideover>
</template>
