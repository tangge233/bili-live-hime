<script setup lang="ts">
import { computed, ref } from "vue";
import { useToast } from "@nuxt/ui/composables";
import { useConfigStore } from "@/stores/config";
import { useWsStore } from "@/stores/ws";
import { sendComment } from "@/api/live";
import CommentItem from "@/components/comments/CommentItem.vue";
import SuperChatItem from "@/components/comments/SuperChatItem.vue";
import VirtualCommentList from "@/components/comments/VirtualCommentList.vue";
import RankList from "@/components/comments/RankList.vue";

type Feed = "all" | "superchat" | "audience";

const props = defineProps<{
  detached: boolean;
}>();

const emit = defineEmits<{
  detach: [];
  "show-floating": [];
  dock: [];
}>();

const configStore = useConfigStore();
const wsStore = useWsStore();
const toast = useToast();

const feed = ref<Feed>("all");
const autoScroll = ref(true);
const newMessage = ref("");
const sending = ref(false);

const tabs = computed(() => [
  {
    label: "弹幕",
    icon: "i-lucide-message-square",
    value: "all",
    badge: String(wsStore.regularMessages.length),
  },
  {
    label: "SC",
    icon: "i-lucide-sparkles",
    value: "superchat",
    badge: String(wsStore.superChats.length),
  },
  {
    label: "榜单",
    icon: "i-lucide-users",
    value: "audience",
  },
]);

const emptyText = computed(() =>
  wsStore.connected ? "等待新弹幕" : "连接后开始接收实时弹幕",
);

async function toggleConnection() {
  if (wsStore.connected) {
    await wsStore.disconnect();
    return;
  }

  const config = configStore.config;
  if (!config.uid || !config.roomId || !config.roomToken) {
    toast.add({
      title: "弹幕凭证尚未就绪",
      description: "请稍候或重新登录后再试",
      color: "warning",
      icon: "i-lucide-clock",
    });
    return;
  }

  await wsStore.connect(config.uid, config.roomId, config.roomToken);
}

async function send() {
  const message = newMessage.value.trim();
  if (!message || sending.value) return;

  sending.value = true;
  try {
    await sendComment(message);
    newMessage.value = "";
  } catch (error) {
    toast.add({
      title: "发送失败",
      description: error instanceof Error ? error.message : String(error),
      color: "error",
      icon: "i-lucide-circle-alert",
    });
  } finally {
    sending.value = false;
  }
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col">
    <div
      class="flex h-8 shrink-0 items-center gap-1.5 border-b border-default px-2">
      <span
        class="size-1.5 rounded-full"
        :class="
          wsStore.connected
            ? 'bg-success'
            : wsStore.connecting
              ? 'animate-pulse bg-warning'
              : 'bg-muted'
        " />
      <span class="text-[10px] font-medium text-toned">
        {{ wsStore.connected ? "实时弹幕" : "弹幕未连接" }}
      </span>
      <span class="text-[9px] text-dimmed tabular-nums">
        {{ wsStore.regularMessages.length }}
      </span>

      <div class="ml-auto flex items-center gap-0.5">
        <UTooltip :text="wsStore.connected ? '断开弹幕' : '连接弹幕'">
          <UButton
            color="neutral"
            :variant="wsStore.connected ? 'soft' : 'ghost'"
            size="xs"
            square
            :icon="wsStore.connected ? 'i-lucide-unplug' : 'i-lucide-plug'"
            :loading="wsStore.connecting"
            :aria-label="wsStore.connected ? '断开弹幕' : '连接弹幕'"
            :class="wsStore.connected ? 'text-success' : 'text-muted'"
            @click="toggleConnection" />
        </UTooltip>

        <UTooltip :text="autoScroll ? '暂停自动滚动' : '恢复自动滚动'">
          <UButton
            color="neutral"
            variant="ghost"
            size="xs"
            square
            icon="i-lucide-arrow-down-to-line"
            :aria-label="autoScroll ? '暂停自动滚动' : '恢复自动滚动'"
            :class="autoScroll ? 'text-primary' : 'text-muted'"
            @click="autoScroll = !autoScroll" />
        </UTooltip>

        <UTooltip text="拆分到悬浮窗">
          <UButton
            color="neutral"
            variant="ghost"
            size="xs"
            square
            icon="i-lucide-picture-in-picture-2"
            aria-label="拆分到悬浮窗"
            class="text-muted"
            @click="emit('detach')" />
        </UTooltip>
      </div>
    </div>

    <div
      v-if="props.detached"
      class="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 p-5 text-center">
      <div
        class="grid size-12 place-items-center rounded-2xl bg-primary/10 text-primary">
        <UIcon name="i-lucide-picture-in-picture-2" class="size-6" />
      </div>
      <div class="space-y-1">
        <p class="text-sm font-medium text-highlighted">弹幕已拆分为悬浮窗</p>
        <p class="text-[11px] text-muted">悬浮窗会接管实时连接</p>
      </div>
      <div class="flex items-center gap-1.5">
        <UBadge
          :color="wsStore.connected ? 'success' : 'neutral'"
          variant="soft"
          size="sm">
          {{ wsStore.connected ? "已连接" : "等待连接" }}
        </UBadge>
        <UBadge color="neutral" variant="soft" size="sm">
          {{ wsStore.regularMessages.length + wsStore.superChats.length }} 条
        </UBadge>
      </div>
      <div class="flex w-full max-w-[220px] gap-1.5">
        <UButton
          block
          size="sm"
          icon="i-lucide-eye"
          label="显示"
          @click="emit('show-floating')" />
        <UButton
          block
          size="sm"
          color="neutral"
          variant="outline"
          icon="i-lucide-x"
          label="收回"
          @click="emit('dock')" />
      </div>
    </div>

    <template v-else>
      <div class="shrink-0 px-2 pt-1.5">
        <UTabs
          v-model="feed"
          :items="tabs"
          :content="false"
          variant="pill"
          size="xs"
          class="w-full"
          :ui="{
            trigger: 'flex-1 px-1.5',
            label: 'text-[10px]',
            trailingBadge: 'text-[8px] h-3.5 px-1',
          }" />
      </div>

      <div class="min-h-0 flex-1 px-1 py-1">
        <template v-if="feed === 'audience'">
          <RankList :active="feed === 'audience'" />
        </template>

        <template v-else-if="feed === 'superchat'">
          <VirtualCommentList
            v-if="wsStore.superChats.length > 0"
            :items="wsStore.superChats"
            :auto-scroll="autoScroll"
            :estimate-size="72">
            <template #default="{ item }">
              <SuperChatItem :comment="item" />
            </template>
          </VirtualCommentList>
          <div
            v-else
            class="flex h-full items-center justify-center px-6 text-center text-[11px] text-muted">
            暂无醒目留言
          </div>
        </template>

        <template v-else>
          <VirtualCommentList
            v-if="wsStore.regularMessages.length > 0"
            :items="wsStore.regularMessages"
            :auto-scroll="autoScroll"
            :estimate-size="44">
            <template #default="{ item }">
              <CommentItem :comment="item" dense />
            </template>
          </VirtualCommentList>
          <div
            v-else
            class="flex h-full flex-col items-center justify-center gap-2 px-6 text-center text-[11px] text-muted">
            <UIcon
              name="i-lucide-message-square-dashed"
              class="size-6 text-dimmed" />
            <span>{{ emptyText }}</span>
            <UButton
              v-if="!wsStore.connected"
              size="xs"
              variant="soft"
              icon="i-lucide-plug"
              label="连接弹幕"
              :loading="wsStore.connecting"
              @click="toggleConnection" />
          </div>
        </template>
      </div>

      <div
        class="flex shrink-0 items-center gap-1.5 border-t border-default p-2">
        <UInput
          v-model="newMessage"
          size="sm"
          placeholder="发送弹幕…"
          class="flex-1"
          :disabled="sending"
          @keydown.enter="send" />
        <UButton
          size="sm"
          square
          icon="i-lucide-send"
          aria-label="发送弹幕"
          :loading="sending"
          :disabled="!newMessage.trim()"
          @click="send" />
      </div>
    </template>
  </section>
</template>
