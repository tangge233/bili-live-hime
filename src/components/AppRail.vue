<script setup lang="ts">
import { computed } from "vue";
import { useConfigStore } from "@/stores/config";

type Panel = "account" | "manager" | "settings";

const props = defineProps<{
  activePanel: Panel | null;
  detached: boolean;
}>();

const emit = defineEmits<{
  dock: [];
  account: [];
  manager: [];
  settings: [];
  "show-floating": [];
}>();

const configStore = useConfigStore();
const dark = computed(() => configStore.config.theme === "dark");

function toggleTheme() {
  configStore.updateConfig({ theme: dark.value ? "light" : "dark" });
}

const isActive = (panel: Panel) => props.activePanel === panel;
</script>

<template>
  <nav
    class="flex w-12 shrink-0 flex-col items-center gap-1 border-e border-default bg-elevated/25 py-2">
    <UTooltip text="直播台">
      <UButton
        color="neutral"
        :variant="activePanel === null ? 'soft' : 'ghost'"
        size="sm"
        square
        icon="i-lucide-radio-tower"
        aria-label="直播台"
        :class="
          activePanel === null
            ? 'bg-primary/15 text-primary'
            : 'text-muted hover:text-default'
        "
        @click="emit('dock')" />
    </UTooltip>

    <UTooltip text="房间管理">
      <UButton
        color="neutral"
        :variant="isActive('manager') ? 'soft' : 'ghost'"
        size="sm"
        square
        icon="i-lucide-shield-check"
        aria-label="房间管理"
        :class="
          isActive('manager')
            ? 'bg-primary/15 text-primary'
            : 'text-muted hover:text-default'
        "
        @click="emit('manager')" />
    </UTooltip>

    <UTooltip text="账号">
      <UButton
        color="neutral"
        :variant="isActive('account') ? 'soft' : 'ghost'"
        size="sm"
        square
        icon="i-lucide-circle-user-round"
        aria-label="账号"
        :class="
          isActive('account')
            ? 'bg-primary/15 text-primary'
            : 'text-muted hover:text-default'
        "
        @click="emit('account')" />
    </UTooltip>

    <div class="mt-auto flex flex-col items-center gap-1">
      <UTooltip v-if="detached" text="弹幕在悬浮窗中">
        <UButton
          color="warning"
          variant="soft"
          size="sm"
          square
          icon="i-lucide-picture-in-picture-2"
          aria-label="弹幕在悬浮窗中"
          @click="emit('show-floating')" />
      </UTooltip>

      <UTooltip text="设置">
        <UButton
          color="neutral"
          :variant="isActive('settings') ? 'soft' : 'ghost'"
          size="sm"
          square
          icon="i-lucide-settings-2"
          aria-label="设置"
          :class="
            isActive('settings')
              ? 'bg-primary/15 text-primary'
              : 'text-muted hover:text-default'
          "
          @click="emit('settings')" />
      </UTooltip>

      <UTooltip :text="dark ? '切换到浅色' : '切换到深色'">
        <UButton
          color="neutral"
          variant="ghost"
          size="sm"
          square
          :icon="dark ? 'i-lucide-sun' : 'i-lucide-moon'"
          :aria-label="dark ? '切换到浅色' : '切换到深色'"
          class="text-muted hover:text-default"
          @click="toggleTheme" />
      </UTooltip>
    </div>
  </nav>
</template>
