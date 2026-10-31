<script setup lang="ts">
import { useWsStore } from "@/stores/ws";

const wsStore = useWsStore();

function formatNumber(value: number) {
  if (value >= 10000) return (value / 10000).toFixed(1) + "w";
  if (value >= 1000) return (value / 1000).toFixed(1) + "k";
  return value.toString();
}
</script>

<template>
  <footer
    class="flex h-6 shrink-0 items-center gap-2 border-t border-default bg-elevated/30 px-2 text-[10px] text-muted">
    <span class="flex items-center gap-1">
      <UIcon name="i-lucide-flame" class="size-3 text-warning" />
      <span class="tabular-nums">{{ formatNumber(wsStore.watchedUser) }}</span>
    </span>
    <span class="h-2.5 w-px bg-border" />
    <span class="flex items-center gap-1">
      <UIcon name="i-lucide-users" class="size-3" />
      <span class="tabular-nums">{{ formatNumber(wsStore.onlineCount) }}</span>
    </span>
    <span class="ml-auto flex items-center gap-1">
      <span
        class="size-1.5 rounded-full"
        :class="
          wsStore.connected
            ? 'bg-success shadow-[0_0_6px_var(--ui-success)]'
            : wsStore.connecting
              ? 'animate-pulse bg-warning'
              : 'bg-dimmed'
        " />
      {{
        wsStore.connected
          ? "弹幕已连接"
          : wsStore.connecting
            ? "连接中"
            : "未连接"
      }}
    </span>
  </footer>
</template>
