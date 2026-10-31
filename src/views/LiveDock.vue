<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { emit, listen } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useToast } from "@nuxt/ui/composables";
import QrcodeVue from "qrcode.vue";
import AppRail from "@/components/AppRail.vue";
import LiveControl from "@/components/LiveControl.vue";
import DanmakuPanel from "@/components/DanmakuPanel.vue";
import StatusBar from "@/components/StatusBar.vue";
import StreamSettingsSlideover from "@/components/StreamSettingsSlideover.vue";
import SettingsSlideover from "@/components/SettingsSlideover.vue";
import AccountSlideover from "@/components/AccountSlideover.vue";
import ManagerSlideover from "@/components/ManagerSlideover.vue";
import { useLiveStore } from "@/stores/live";

type Panel = "account" | "manager" | "settings";

defineEmits<{
  logout: [];
}>();

const liveStore = useLiveStore();
const toast = useToast();

const activePanel = ref<Panel | null>(null);
const streamSettingsOpen = ref(false);
const commentsDetached = ref(false);

function panelModel(panel: Panel) {
  return computed({
    get: () => activePanel.value === panel,
    set: (value: boolean) => {
      if (value) {
        activePanel.value = panel;
      } else if (activePanel.value === panel) {
        activePanel.value = null;
      }
    },
  });
}

const settingsOpen = panelModel("settings");
const accountOpen = panelModel("account");
const managerOpen = panelModel("manager");

let disposeDocked: (() => void) | undefined;
let disposed = false;

async function getCommentsWindow() {
  return WebviewWindow.getByLabel("comments");
}

async function showCommentsWindow() {
  try {
    const window = await getCommentsWindow();
    if (!window) {
      toast.add({
        title: "找不到弹幕悬浮窗",
        color: "error",
        icon: "i-lucide-circle-alert",
      });
      return;
    }
    await window.show();
    await window.setFocus();
    await emit("comments-window-visibility", true);
    commentsDetached.value = true;
  } catch (error) {
    toast.add({
      title: "显示悬浮窗失败",
      description: error instanceof Error ? error.message : String(error),
      color: "error",
      icon: "i-lucide-circle-alert",
    });
  }
}

async function dockCommentsWindow() {
  try {
    await emit("comments-window-visibility", false);
    const window = await getCommentsWindow();
    await window?.hide();
    commentsDetached.value = false;
  } catch (error) {
    toast.add({
      title: "收回悬浮窗失败",
      description: error instanceof Error ? error.message : String(error),
      color: "error",
      icon: "i-lucide-circle-alert",
    });
  }
}

function openPanel(panel: Panel) {
  activePanel.value = activePanel.value === panel ? null : panel;
}

onMounted(async () => {
  const dispose = await listen("comments-window-docked", () => {
    commentsDetached.value = false;
  });
  if (disposed) {
    dispose();
  } else {
    disposeDocked = dispose;
  }

  try {
    const window = await getCommentsWindow();
    if (window) {
      commentsDetached.value = await window.isVisible();
    }
  } catch (error) {
    console.error("Failed to query comments window visibility:", error);
  }
});

onUnmounted(() => {
  disposed = true;
  disposeDocked?.();
});
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <div class="flex min-h-0 flex-1">
      <AppRail
        :active-panel="activePanel"
        :detached="commentsDetached"
        @dock="activePanel = null"
        @account="openPanel('account')"
        @manager="openPanel('manager')"
        @settings="openPanel('settings')"
        @show-floating="showCommentsWindow" />

      <main class="flex min-h-0 min-w-0 flex-1 flex-col">
        <LiveControl @open-settings="streamSettingsOpen = true" />
        <DanmakuPanel
          :detached="commentsDetached"
          @detach="showCommentsWindow"
          @show-floating="showCommentsWindow"
          @dock="dockCommentsWindow" />
      </main>
    </div>

    <StatusBar />

    <StreamSettingsSlideover v-model:open="streamSettingsOpen" />
    <SettingsSlideover v-model:open="settingsOpen" />
    <AccountSlideover v-model:open="accountOpen" @logout="$emit('logout')" />
    <ManagerSlideover v-model:open="managerOpen" />

    <UModal
      v-model:open="liveStore.qrOpen"
      title="开播验证"
      description="本次开播需要身份验证，请使用哔哩哔哩 App 扫码完成验证。扫码完成后，请关闭此对话框。"
      :ui="{ content: 'max-w-xs' }">
      <template #body>
        <div class="flex justify-center">
          <div class="rounded-xl border border-default bg-white p-2">
            <QrcodeVue
              v-if="liveStore.qrUrl"
              :value="liveStore.qrUrl"
              :size="220"
              level="M"
              render-as="svg" />
          </div>
        </div>
      </template>
      <template #footer>
        <UButton
          color="neutral"
          variant="outline"
          size="sm"
          label="关闭"
          @click="liveStore.qrOpen = false" />
      </template>
    </UModal>
  </div>
</template>
