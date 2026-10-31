<script setup lang="ts" generic="T">
import { computed, nextTick, ref, watch } from "vue";
import { useVirtualizer } from "@tanstack/vue-virtual";

const props = withDefaults(
  defineProps<{
    items: T[];
    autoScroll?: boolean;
    estimateSize?: number;
  }>(),
  {
    autoScroll: true,
    estimateSize: 48,
  },
);

const parentRef = ref<HTMLElement | null>(null);

const virtualizer = useVirtualizer(
  computed(() => ({
    count: props.items.length,
    getScrollElement: () => parentRef.value,
    estimateSize: () => props.estimateSize,
    overscan: 8,
  })),
);

function measureElement(element: Element | null) {
  if (element) {
    virtualizer.value.measureElement(element);
  }
}

watch(
  () => props.items.length,
  async (length) => {
    if (!props.autoScroll || length === 0) return;
    await nextTick();
    virtualizer.value.scrollToIndex(length - 1, { align: "end" });
  },
);
</script>

<template>
  <div ref="parentRef" class="h-full w-full overflow-x-hidden overflow-y-auto">
    <div
      class="relative w-full"
      :style="{ height: `${virtualizer.getTotalSize()}px` }">
      <div
        v-for="row in virtualizer.getVirtualItems()"
        :key="String(row.key)"
        :data-index="row.index"
        :ref="(element) => measureElement(element as Element | null)"
        class="absolute top-0 left-0 w-full"
        :style="{ transform: `translateY(${row.start}px)` }">
        <slot :item="props.items[row.index]" :index="row.index" />
      </div>
    </div>
  </div>
</template>
