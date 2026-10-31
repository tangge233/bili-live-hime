<script setup lang="ts">
defineProps<{
  title: string;
  description?: string;
}>();

defineEmits<{
  back: [];
}>();
</script>

<template>
  <div class="flex min-w-0 flex-1 items-center gap-2">
    <UButton
      color="neutral"
      variant="soft"
      size="xs"
      icon="i-lucide-arrow-left"
      label="返回"
      aria-label="返回直播台"
      @click="$emit('back')" />
    <div class="min-w-0">
      <h2 class="truncate text-sm font-semibold text-highlighted">
        {{ title }}
      </h2>
      <p v-if="description" class="truncate text-[10px] text-muted">
        {{ description }}
      </p>
    </div>
  </div>
</template>
