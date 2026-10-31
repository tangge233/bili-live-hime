<script setup lang="ts">
import { computed, watch } from "vue";

export interface UserSelectItem {
  id: string;
  name: string;
  avatar: string;
}

const open = defineModel<boolean>("open", { default: false });

const props = defineProps<{
  users: UserSelectItem[];
  selectedId: string | null;
}>();

const emit = defineEmits<{
  "update:selectedId": [value: string | null];
  confirm: [user: UserSelectItem];
}>();

const selectedUser = computed(() =>
  props.users.find((user) => user.id === props.selectedId),
);

watch(open, (value) => {
  if (value) emit("update:selectedId", null);
});

function confirm() {
  if (!selectedUser.value) return;
  emit("confirm", selectedUser.value);
  open.value = false;
}
</script>

<template>
  <UModal
    v-model:open="open"
    title="选择用户"
    description="从搜索结果中选择要操作的用户"
    :ui="{ content: 'max-w-sm' }">
    <template #body>
      <div class="max-h-80 space-y-1.5 overflow-y-auto">
        <button
          v-for="user in users"
          :key="user.id"
          type="button"
          class="flex w-full items-center gap-2.5 rounded-lg border p-2 text-left transition"
          :class="
            user.id === selectedId
              ? 'border-primary bg-primary/5'
              : 'border-default hover:bg-muted/60'
          "
          @click="emit('update:selectedId', user.id)">
          <UAvatar
            :src="user.avatar || '/akarin.webp'"
            :alt="user.name"
            size="sm" />
          <span class="min-w-0 flex-1">
            <span class="block truncate text-xs font-medium text-highlighted">
              {{ user.name }}
            </span>
            <span class="block text-[10px] text-muted">UID: {{ user.id }}</span>
          </span>
          <span
            class="grid size-4 shrink-0 place-items-center rounded-full border"
            :class="
              user.id === selectedId
                ? 'border-primary bg-primary'
                : 'border-muted-foreground'
            ">
            <UIcon
              v-if="user.id === selectedId"
              name="i-lucide-check"
              class="size-2.5 text-inverted" />
          </span>
        </button>

        <p
          v-if="users.length === 0"
          class="py-8 text-center text-xs text-muted">
          未搜索到用户
        </p>
      </div>
    </template>

    <template #footer>
      <UButton
        color="neutral"
        variant="outline"
        size="sm"
        label="取消"
        @click="open = false" />
      <UButton
        size="sm"
        label="确定"
        :disabled="!selectedUser"
        @click="confirm" />
    </template>
  </UModal>
</template>
