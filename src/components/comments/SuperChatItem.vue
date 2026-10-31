<script setup lang="ts">
import { computed } from "vue";
import type { Comment } from "@/types/comment";

const props = defineProps<{ comment: Comment }>();

const time = computed(() =>
  props.comment.timestamp.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  }),
);

const accent = computed(() => {
  const amount = props.comment.amount ?? 0;
  if (amount <= 30) return "border-info/40 bg-info/10";
  if (amount <= 50) return "border-cyan-500/40 bg-cyan-500/10";
  if (amount <= 100) return "border-warning/40 bg-warning/10";
  if (amount <= 500) return "border-orange-400/40 bg-orange-400/10";
  if (amount <= 1000) return "border-error/40 bg-error/10";
  return "border-error bg-error/15";
});

const amountColor = computed(() => {
  const amount = props.comment.amount ?? 0;
  if (amount <= 30) return "bg-info";
  if (amount <= 50) return "bg-cyan-600";
  if (amount <= 100) return "bg-warning";
  if (amount <= 500) return "bg-orange-500";
  return "bg-error";
});
</script>

<template>
  <div class="my-1 overflow-hidden rounded-lg border" :class="accent">
    <div class="flex items-center gap-1.5 border-b border-current/10 px-2 py-1">
      <UAvatar
        :src="comment.avatar || '/akarin.webp'"
        :alt="comment.username"
        size="2xs"
        class="shrink-0" />
      <span class="min-w-0 flex-1 truncate text-[11px] font-semibold">
        {{ comment.username }}
      </span>
      <UBadge
        color="neutral"
        variant="solid"
        size="sm"
        class="h-4 border-0 px-1.5 text-[9px] text-white"
        :class="amountColor">
        ¥{{ comment.amount || 0 }}
      </UBadge>
      <time class="shrink-0 text-[9px] text-dimmed tabular-nums">
        {{ time }}
      </time>
    </div>
    <p class="px-2 py-1.5 text-xs leading-4 break-words text-toned">
      {{ comment.message }}
    </p>
  </div>
</template>
