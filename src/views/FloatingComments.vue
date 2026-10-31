<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit, listen } from "@tauri-apps/api/event";
import { useToast } from "@nuxt/ui/composables";
import { useConfigStore } from "@/stores/config";
import { useWsStore } from "@/stores/ws";
import { sendComment } from "@/api/live";
import CommentItem from "@/components/comments/CommentItem.vue";
import SuperChatItem from "@/components/comments/SuperChatItem.vue";
import VirtualCommentList from "@/components/comments/VirtualCommentList.vue";
import RankList from "@/components/comments/RankList.vue";

type Feed = "comments" | "superchats" | "audience";

const OPACITY_STORAGE_KEY = "comments-window-opacity";

const configStore = useConfigStore();
const wsStore = useWsStore();
const toast = useToast();

const feed = ref<Feed>("comments");
const autoScroll = ref(true);
const showOpacity = ref(false);
const opacity = ref(readOpacity());
const newMessage = ref("");
const sending = ref(false);
const visible = ref(false);

const attemptedCredentials = ref<string | null>(null);
const ownsConnection = ref(false);

const cleanupFns: Array<() => void> = [];
let disposed = false;
let credentialTimer: ReturnType<typeof setInterval> | null = null;

const rankActive = computed(() => feed.value === "audience");

const tabs = computed(() => [
  {
    label: `弹幕 ${wsStore.regularMessages.length}`,
    icon: "i-lucide-message-square",
    value: "comments",
  },
  {
    label: `SC ${wsStore.superChats.length}`,
    icon: "i-lucide-sparkles",
    value: "superchats",
  },
  {
    label: "在线榜",
    icon: "i-lucide-users",
    value: "audience",
  },
]);

const backgroundStyle = computed(() => ({
  backgroundColor: `color-mix(in oklab, var(--ui-bg, #ffffff) ${opacity.value}%, transparent)`,
}));

function readOpacity() {
  const stored = localStorage.getItem(OPACITY_STORAGE_KEY);
  if (stored === null) return 80;
  const value = Number(stored);
  return Number.isFinite(value) && value >= 0 && value <= 100 ? value : 80;
}

function addCleanup(fn: () => void) {
  if (disposed) {
    fn();
  } else {
    cleanupFns.push(fn);
  }
}

async function dockCommentsWindow() {
  await emit("comments-window-visibility", false);
  await emit("comments-window-docked");
  await getCurrentWindow().hide();
}

async function closeWindow() {
  visible.value = false;
  try {
    await dockCommentsWindow();
  } catch (error) {
    console.error("Failed to dock comments window:", error);
  }
}

function connectIfPossible() {
  if (!visible.value) {
    attemptedCredentials.value = null;
    return;
  }

  const { uid, roomId, roomToken } = configStore.config;
  if (wsStore.connected || wsStore.connecting) return;
  if (!uid || !roomId || !roomToken) return;

  const credentials = `${uid}:${roomId}:${roomToken}`;
  if (attemptedCredentials.value === credentials) return;

  attemptedCredentials.value = credentials;
  ownsConnection.value = true;
  void wsStore.connect(uid, roomId, roomToken);
}

watch(
  () => [
    visible.value,
    configStore.config.uid,
    configStore.config.roomId,
    configStore.config.roomToken,
    wsStore.connected,
    wsStore.connecting,
  ],
  () => {
    connectIfPossible();

    if (visible.value && wsStore.connected) {
      ownsConnection.value = true;
    }

    if (!visible.value && ownsConnection.value) {
      attemptedCredentials.value = null;
      if (wsStore.connected || wsStore.connecting) {
        void wsStore.disconnect();
      }
      ownsConnection.value = false;
    }
  },
  { immediate: true },
);

watch(opacity, (value) => {
  localStorage.setItem(OPACITY_STORAGE_KEY, String(value));
});

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

function onInputEnter(event: KeyboardEvent) {
  if (event.isComposing) return;
  void send();
}

onMounted(async () => {
  document.body.classList.add("floating-window");

  try {
    const isVisible = await getCurrentWindow().isVisible();
    if (!disposed) visible.value = isVisible;
  } catch (error) {
    console.error("Failed to query comments window visibility:", error);
  }

  addCleanup(
    await listen<boolean>("comments-window-visibility", (event) => {
      visible.value = event.payload;
    }),
  );

  addCleanup(
    await getCurrentWindow().onCloseRequested((event) => {
      event.preventDefault();
      void closeWindow();
    }),
  );

  addCleanup(
    await listen("auth-cleared", () => {
      configStore.resetAuth();
      wsStore.reset();
      void wsStore.disconnect().catch((error) => {
        console.error("Failed to stop comments after logout:", error);
      });
    }),
  );

  credentialTimer = setInterval(() => {
    const { uid, roomId, roomToken } = configStore.config;
    if (uid && roomId && roomToken) return;
    void configStore.init();
  }, 3000);
});

onUnmounted(() => {
  disposed = true;
  if (credentialTimer) clearInterval(credentialTimer);
  cleanupFns.forEach((fn) => fn());
  document.body.classList.remove("floating-window");
});
</script>

<template>
  <div
    class="relative flex h-screen flex-col overflow-hidden rounded-lg border border-default text-default shadow-xl backdrop-blur-sm"
    :style="backgroundStyle">
    <header
      data-tauri-drag-region
      class="flex h-11 shrink-0 items-center gap-1.5 border-b border-default/70 bg-elevated/35 px-2">
      <span
        class="size-1.5 shrink-0 rounded-full"
        :class="
          wsStore.connected
            ? 'bg-success shadow-[0_0_6px_var(--ui-success)]'
            : wsStore.connecting
              ? 'animate-pulse bg-warning'
              : 'bg-dimmed'
        " />
      <strong data-tauri-drag-region class="shrink-0 text-xs font-semibold">
        直播弹幕
      </strong>
      <span data-tauri-drag-region class="truncate text-[10px] text-muted">
        {{
          wsStore.connected
            ? "实时"
            : wsStore.connecting
              ? "连接中"
              : "等待直播间"
        }}
      </span>

      <div class="ml-auto flex items-center gap-0.5">
        <UTooltip text="调整透明度">
          <UButton
            color="neutral"
            :variant="showOpacity ? 'soft' : 'ghost'"
            size="xs"
            square
            icon="i-lucide-blend"
            aria-label="调整透明度"
            :class="showOpacity ? 'text-primary' : 'text-muted'"
            @click="showOpacity = !showOpacity" />
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
        <UTooltip text="隐藏悬浮窗">
          <UButton
            color="neutral"
            variant="ghost"
            size="xs"
            square
            icon="i-lucide-x"
            aria-label="隐藏悬浮窗"
            class="text-muted hover:bg-error hover:text-inverted"
            @click="closeWindow" />
        </UTooltip>
      </div>
    </header>

    <div
      v-if="showOpacity"
      class="absolute top-10 right-9 z-20 w-44 rounded-lg border border-default bg-default p-3 text-default shadow-lg">
      <div class="mb-2 flex items-center justify-between text-[11px]">
        <span>透明度</span>
        <span class="text-muted tabular-nums">{{ opacity }}%</span>
      </div>
      <USlider
        v-model="opacity"
        :min="0"
        :max="100"
        :step="5"
        aria-label="悬浮窗透明度" />
    </div>

    <div class="shrink-0 border-b border-default/60 bg-elevated/20 p-1">
      <UTabs
        v-model="feed"
        :items="tabs"
        :content="false"
        variant="pill"
        size="xs"
        class="w-full"
        :ui="{
          trigger: 'flex-1 px-1',
          label: 'text-[10px]',
        }" />
    </div>

    <div class="min-h-0 flex-1 py-1">
      <template v-if="feed === 'audience'">
        <RankList :active="rankActive" dense />
      </template>

      <template v-else-if="feed === 'superchats'">
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
          class="flex h-full flex-col items-center justify-center gap-1 px-6 text-center">
          <UIcon name="i-lucide-sparkles" class="size-6 text-dimmed" />
          <p class="text-[11px] text-muted">
            {{ wsStore.connected ? "暂无醒目留言" : "登录后将自动连接直播间" }}
          </p>
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
          class="flex h-full flex-col items-center justify-center gap-1 px-6 text-center">
          <UIcon
            name="i-lucide-message-square-dashed"
            class="size-6 text-dimmed" />
          <p class="text-[11px] text-muted">
            {{ wsStore.connected ? "等待新弹幕" : "登录后将自动连接直播间" }}
          </p>
        </div>
      </template>
    </div>

    <div
      class="flex shrink-0 items-center gap-1.5 border-t border-default/60 bg-elevated/25 p-2">
      <UInput
        v-model="newMessage"
        size="sm"
        placeholder="发送弹幕…"
        aria-label="发送弹幕"
        class="flex-1"
        :disabled="sending"
        @keydown.enter="onInputEnter" />
      <UButton
        size="sm"
        square
        icon="i-lucide-send"
        aria-label="发送弹幕"
        :loading="sending"
        :disabled="!newMessage.trim()"
        @click="send" />
    </div>
  </div>
</template>
