<script setup lang="ts">
import { computed } from "vue";
import { useConfigStore } from "@/stores/config";
import { useLiveStore } from "@/stores/live";

const emit = defineEmits<{
  "open-settings": [];
}>();

const configStore = useConfigStore();
const liveStore = useLiveStore();

const config = computed(() => configStore.config);
const isLive = computed(() => config.value.isOpenLive);

const areaName = computed(() => {
  for (const parent of config.value.areaList) {
    const area = parent.list.find((item) => item.id === config.value.areaId);
    if (area) return `${parent.name} · ${area.name}`;
  }
  return "未选择分区";
});

const needsSetup = computed(
  () =>
    !isLive.value &&
    (config.value.roomTitle.trim() === "" ||
      config.value.categoryId === "" ||
      config.value.areaId === ""),
);
</script>

<template>
  <section class="shrink-0 border-b border-default bg-elevated/25 px-2.5 py-2">
    <div class="flex items-start gap-2">
      <UAvatar
        :src="config.avatar || '/akarin.webp'"
        :alt="config.username || 'user'"
        size="sm"
        class="mt-0.5 shrink-0" />

      <div class="min-w-0 flex-1">
        <div class="flex min-w-0 items-center gap-1.5">
          <span class="truncate text-xs font-semibold text-highlighted">
            {{ config.roomTitle || "未设置直播标题" }}
          </span>
          <span
            v-if="isLive"
            class="flex shrink-0 items-center gap-1 rounded-full bg-error/10 px-1.5 py-0.5 text-[8px] font-bold tracking-wide text-error">
            <span class="size-1 animate-pulse rounded-full bg-error" />
            LIVE
          </span>
        </div>
        <div class="mt-0.5 flex items-center gap-1 text-[10px] text-muted">
          <span class="truncate">{{ config.username || "未登录" }}</span>
          <span class="text-dimmed">·</span>
          <span class="truncate">{{ areaName }}</span>
        </div>
      </div>

      <UTooltip text="开播设置">
        <UButton
          color="neutral"
          variant="ghost"
          size="xs"
          square
          icon="i-lucide-sliders-horizontal"
          aria-label="开播设置"
          class="shrink-0 text-muted"
          @click="emit('open-settings')" />
      </UTooltip>
    </div>

    <div class="mt-2 flex gap-1.5">
      <UButton
        v-if="!isLive"
        block
        size="sm"
        icon="i-lucide-radio"
        label="开始直播"
        :loading="liveStore.starting"
        :disabled="!liveStore.canStart"
        @click="liveStore.startStream" />
      <UButton
        v-else
        block
        size="sm"
        color="error"
        icon="i-lucide-square"
        label="停止直播"
        :loading="liveStore.stopping"
        @click="liveStore.stopStream" />

      <UButton
        color="neutral"
        variant="outline"
        size="sm"
        square
        icon="i-lucide-settings-2"
        aria-label="开播设置"
        @click="emit('open-settings')" />
    </div>

    <div
      v-if="isLive && liveStore.activeStream"
      class="mt-1.5 flex items-center gap-1.5 rounded-md bg-muted/70 px-2 py-1">
      <UIcon name="i-lucide-key-round" class="size-3 shrink-0 text-warning" />
      <span class="min-w-0 flex-1 truncate font-mono text-[9px] text-muted">
        {{ liveStore.activeStream.address }}
      </span>
      <UButton
        color="neutral"
        variant="ghost"
        size="xs"
        square
        icon="i-lucide-copy"
        aria-label="复制推流地址"
        class="size-5"
        @click="
          liveStore.copy(liveStore.activeStream.address, '推流地址已复制')
        " />
    </div>

    <p v-if="needsSetup" class="mt-1.5 text-[10px] text-dimmed">
      先在开播设置中填写标题与分区。
    </p>
  </section>
</template>
