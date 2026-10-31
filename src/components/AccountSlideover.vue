<script setup lang="ts">
import { computed, ref } from "vue";
import { useToast } from "@nuxt/ui/composables";
import SlideoverHeader from "@/components/SlideoverHeader.vue";
import { useConfigStore } from "@/stores/config";

const open = defineModel<boolean>("open", { default: false });

const emit = defineEmits<{
  logout: [];
}>();

const configStore = useConfigStore();
const toast = useToast();

const config = computed(() => configStore.config);
const loggingOut = ref(false);

async function copy(value: string, label: string) {
  try {
    await navigator.clipboard.writeText(value);
    toast.add({
      title: label,
      color: "success",
      icon: "i-lucide-copy-check",
    });
  } catch (error) {
    toast.add({
      title: "复制失败",
      description: error instanceof Error ? error.message : String(error),
      color: "error",
      icon: "i-lucide-circle-alert",
    });
  }
}

async function logout() {
  loggingOut.value = true;
  try {
    emit("logout");
  } finally {
    loggingOut.value = false;
  }
}
</script>

<template>
  <USlideover
    v-model:open="open"
    :close="false"
    title="账号"
    description="当前登录信息"
    :ui="{
      content: 'w-full max-w-none',
      body: 'space-y-4 p-3',
      footer: 'p-3',
    }">
    <template #header="{ close }">
      <SlideoverHeader title="账号" description="当前登录信息" @back="close" />
    </template>

    <template #body>
      <section
        class="flex flex-col items-center gap-2 rounded-xl border border-default bg-elevated/30 px-4 py-6">
        <UAvatar
          :src="config.avatar || '/akarin.webp'"
          :alt="config.username || 'user'"
          size="3xl"
          class="border border-default" />
        <div class="text-center">
          <p class="text-sm font-semibold text-highlighted">
            {{ config.username || "user" }}
          </p>
          <p class="text-[10px] text-muted">已登录哔哩哔哩</p>
        </div>
      </section>

      <section class="grid grid-cols-2 gap-2">
        <div class="rounded-lg bg-muted/60 p-2.5">
          <p class="text-[9px] tracking-wider text-dimmed uppercase">User ID</p>
          <div class="mt-1 flex items-center gap-1">
            <span class="min-w-0 flex-1 truncate font-mono text-xs text-toned">
              {{ config.uid }}
            </span>
            <UButton
              color="neutral"
              variant="ghost"
              size="xs"
              square
              icon="i-lucide-copy"
              aria-label="复制用户 ID"
              class="size-5"
              @click="copy(String(config.uid), '用户 ID 已复制')" />
          </div>
        </div>

        <div class="rounded-lg bg-muted/60 p-2.5">
          <p class="text-[9px] tracking-wider text-dimmed uppercase">直播间</p>
          <div class="mt-1 flex items-center gap-1">
            <span class="min-w-0 flex-1 truncate font-mono text-xs text-toned">
              {{ config.roomId }}
            </span>
            <UButton
              color="neutral"
              variant="ghost"
              size="xs"
              square
              icon="i-lucide-copy"
              aria-label="复制直播间 ID"
              class="size-5"
              @click="copy(String(config.roomId), '直播间 ID 已复制')" />
          </div>
        </div>
      </section>

      <section class="rounded-lg border border-default p-3">
        <div class="flex items-center justify-between gap-2">
          <div class="flex items-center gap-2">
            <UIcon name="i-lucide-link" class="size-3.5 text-muted" />
            <span class="text-[11px] text-muted">直播状态</span>
          </div>
          <UBadge
            :color="config.isOpenLive ? 'error' : 'neutral'"
            variant="soft"
            size="sm">
            {{ config.isOpenLive ? "直播中" : "未开播" }}
          </UBadge>
        </div>
        <div class="mt-2 flex items-center justify-between gap-2">
          <div class="flex items-center gap-2">
            <UIcon name="i-lucide-message-square" class="size-3.5 text-muted" />
            <span class="text-[11px] text-muted">弹幕凭据</span>
          </div>
          <UBadge
            :color="config.roomToken ? 'success' : 'warning'"
            variant="soft"
            size="sm">
            {{ config.roomToken ? "已就绪" : "未就绪" }}
          </UBadge>
        </div>
      </section>
    </template>

    <template #footer>
      <UButton
        block
        color="error"
        variant="soft"
        size="sm"
        icon="i-lucide-log-out"
        label="退出登录"
        :loading="loggingOut"
        @click="logout" />
    </template>
  </USlideover>
</template>
