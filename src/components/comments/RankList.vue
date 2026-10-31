<script setup lang="ts">
import { computed } from "vue";
import type { ContributionRankItem } from "@/api/live";
import { useContributionRank } from "@/composables/useContributionRank";

const props = withDefaults(
  defineProps<{
    active: boolean;
    dense?: boolean;
  }>(),
  { dense: false },
);

const activeRef = computed(() => props.active);
const { items, loading, error, refresh } = useContributionRank(activeRef);

function intToHexColor(color: number) {
  return `#${color.toString(16).padStart(6, "0")}`;
}

function medalStyle(user: ContributionRankItem) {
  const medal = user.medal_info;
  if (!medal) return undefined;
  return {
    background: `linear-gradient(90deg, ${intToHexColor(medal.medal_color_start)}, ${intToHexColor(medal.medal_color_end)})`,
    borderColor: intToHexColor(medal.medal_color_border),
    opacity: medal.is_light === 1 ? 1 : 0.35,
  };
}
</script>

<template>
  <div class="h-full overflow-y-auto">
    <div
      v-if="loading && items.length === 0"
      class="flex h-full items-center justify-center">
      <UIcon
        name="i-lucide-loader-circle"
        class="size-5 animate-spin text-primary" />
    </div>

    <div
      v-else-if="error && items.length === 0"
      class="flex h-full flex-col items-center justify-center gap-2 px-6 text-center">
      <UIcon name="i-lucide-circle-alert" class="size-5 text-error" />
      <p class="text-[11px] text-muted">在线榜加载失败</p>
      <UButton
        size="xs"
        variant="soft"
        icon="i-lucide-refresh-cw"
        label="重试"
        @click="refresh" />
    </div>

    <div
      v-else-if="items.length === 0"
      class="flex h-full items-center justify-center px-6 text-center text-[11px] text-muted">
      暂无在线榜数据
    </div>

    <div v-else class="divide-y divide-default/60">
      <div
        v-for="(user, index) in items"
        :key="user.uid"
        class="flex items-center gap-2 px-2 py-1.5 hover:bg-muted/40">
        <div class="flex w-4 shrink-0 items-center justify-center">
          <UIcon
            v-if="index === 0"
            name="i-lucide-trophy"
            class="size-3.5 text-warning" />
          <UIcon
            v-else-if="index === 1"
            name="i-lucide-medal"
            class="size-3.5 text-slate-400" />
          <UIcon
            v-else-if="index === 2"
            name="i-lucide-award"
            class="size-3.5 text-amber-600" />
          <span v-else class="text-[10px] text-dimmed tabular-nums">
            {{ index + 1 }}
          </span>
        </div>

        <UAvatar
          :src="user.face || '/akarin.webp'"
          :alt="user.name"
          size="2xs"
          class="shrink-0" />

        <div class="min-w-0 flex-1">
          <div class="truncate text-[11px] font-medium">{{ user.name }}</div>
          <div v-if="!dense" class="flex items-center gap-1">
            <span
              v-if="user.wealth_level > 0"
              class="rounded bg-muted px-1 text-[8px] text-muted">
              荣耀 {{ user.wealth_level }}
            </span>
            <span
              v-if="user.medal_info"
              class="rounded border px-1 text-[8px] text-white"
              :style="medalStyle(user)">
              {{ user.medal_info.medal_name }} {{ user.medal_info.level }}
            </span>
          </div>
        </div>

        <span
          class="shrink-0 text-[10px] font-semibold text-primary tabular-nums">
          {{ user.score }}
        </span>
      </div>
    </div>
  </div>
</template>
