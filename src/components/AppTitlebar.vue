<script setup lang="ts">
import { computed } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useConfigStore } from "@/stores/config";

const configStore = useConfigStore();
const isLive = computed(() => configStore.config.isOpenLive);

function minimize() {
  void getCurrentWindow().minimize();
}

function close() {
  void getCurrentWindow().close();
}
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex h-10 shrink-0 items-center gap-2 border-b border-default bg-elevated/60 px-2.5">
    <div
      class="grid size-6 place-items-center rounded-md bg-primary text-inverted shadow-sm">
      <UIcon name="i-simple-icons:bilibili" class="size-3.5" />
    </div>

    <div data-tauri-drag-region class="flex min-w-0 items-center gap-1.5">
      <span class="text-xs font-semibold tracking-wide">直播姬</span>
      <span class="text-[9px] font-medium tracking-[0.18em] text-dimmed">
        DECK
      </span>
    </div>

    <UBadge
      v-if="isLive"
      color="error"
      variant="soft"
      size="sm"
      class="ml-1 h-5 gap-1 px-1.5 text-[9px] font-bold tracking-wider">
      <span class="size-1.5 animate-pulse rounded-full bg-error" />
      LIVE
    </UBadge>

    <div class="ml-auto flex items-center gap-0.5">
      <UButton
        color="neutral"
        variant="ghost"
        size="xs"
        square
        icon="i-lucide-minus"
        aria-label="最小化"
        @click="minimize" />
      <UButton
        color="neutral"
        variant="ghost"
        size="xs"
        square
        icon="i-lucide-x"
        aria-label="关闭"
        class="hover:bg-error hover:text-inverted"
        @click="close" />
    </div>
  </header>
</template>
