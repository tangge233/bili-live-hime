<script setup lang="ts">
import { computed } from "vue";
import type { Comment } from "@/types/comment";

const props = withDefaults(
  defineProps<{
    comment: Comment;
    dense?: boolean;
  }>(),
  { dense: false },
);

const time = computed(() =>
  props.comment.timestamp.toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  }),
);
</script>

<template>
  <div
    v-if="comment.type === 'enter'"
    class="px-2 py-0.5 text-[11px] text-muted">
    <span class="font-medium text-primary">{{ comment.username }}</span>
    {{ comment.message }}
  </div>

  <div
    v-else-if="comment.type === 'gift'"
    class="my-0.5 flex items-center gap-1.5 rounded-md bg-primary/10 px-2 py-1 text-[11px]">
    <UIcon name="i-lucide-gift" class="size-3.5 shrink-0 text-primary" />
    <span class="min-w-0 truncate">
      <span class="font-medium text-primary">{{ comment.username }}</span>
      <span class="text-muted"> 送出 </span>
      <span class="font-medium text-toned">
        {{ comment.giftName }} ×{{ comment.giftCount }}
      </span>
    </span>
  </div>

  <div
    v-else
    class="flex items-start gap-1.5 rounded-md px-2 py-1 hover:bg-muted/40">
    <UAvatar
      :src="comment.avatar || '/akarin.webp'"
      :alt="comment.username"
      size="2xs"
      class="mt-0.5 shrink-0" />
    <div class="min-w-0 flex-1">
      <div class="flex items-baseline gap-1.5">
        <span class="truncate text-[11px] font-medium text-primary">
          {{ comment.username }}
        </span>
        <UBadge
          v-if="comment.badge"
          color="neutral"
          variant="soft"
          size="sm"
          class="h-3.5 px-1 text-[9px]">
          {{ comment.badge }}
        </UBadge>
        <time class="ml-auto shrink-0 text-[9px] text-dimmed tabular-nums">
          {{ time }}
        </time>
      </div>
      <p
        class="text-xs leading-4 break-words text-default"
        :class="dense ? 'line-clamp-3' : ''">
        {{ comment.message }}
      </p>
    </div>
  </div>
</template>
