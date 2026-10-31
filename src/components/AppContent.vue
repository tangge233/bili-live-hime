<script setup lang="ts">
import { onMounted, onUnmounted, ref, watchEffect } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useConfigStore } from "@/stores/config";
import { useWsStore } from "@/stores/ws";
import AppTitlebar from "@/components/AppTitlebar.vue";
import LoadingScreen from "@/screens/LoadingScreen.vue";
import LoginScreen from "@/screens/LoginScreen.vue";
import LiveDock from "@/views/LiveDock.vue";
import FloatingComments from "@/views/FloatingComments.vue";

type AuthState = "loading" | "login" | "authenticated";

const isCommentsWindow = (() => {
  try {
    return getCurrentWindow().label === "comments";
  } catch {
    return false;
  }
})();

const configStore = useConfigStore();
const wsStore = useWsStore();

const authState = ref<AuthState>("loading");
let disposeListeners: (() => void) | undefined;
let disposed = false;

onMounted(async () => {
  const dispose = await wsStore.initListeners();
  if (disposed) {
    dispose();
  } else {
    disposeListeners = dispose;
  }

  if (!configStore.isInitialized) {
    await configStore.init();
  }
});

onUnmounted(() => {
  disposed = true;
  disposeListeners?.();
});

watchEffect(() => {
  if (!configStore.isInitialized) return;

  const dark = configStore.config.theme === "dark";
  document.documentElement.classList.toggle("dark", dark);
  document.documentElement.style.colorScheme = dark ? "dark" : "light";
});

function handleValidationComplete(isValid: boolean) {
  authState.value = isValid ? "authenticated" : "login";
}

function handleLoginSuccess() {
  authState.value = "loading";
}

async function handleLogout() {
  try {
    await configStore.clearAuth();
  } catch (error) {
    console.error("Failed to clear authentication:", error);
  } finally {
    authState.value = "login";
  }
}
</script>

<template>
  <div v-if="isCommentsWindow" class="h-full">
    <FloatingComments v-if="configStore.isInitialized" />
    <div
      v-else
      class="flex h-full items-center justify-center bg-transparent text-xs text-white/60">
      正在载入弹幕...
    </div>
  </div>

  <div
    v-else
    class="flex h-full min-h-0 flex-col overflow-hidden border border-default bg-default text-default shadow-2xl">
    <AppTitlebar />

    <div
      v-if="!configStore.isInitialized"
      class="flex flex-1 items-center justify-center gap-2 text-sm text-muted">
      <UIcon
        name="i-lucide-loader-circle"
        class="size-4 animate-spin text-primary" />
      初始化中...
    </div>

    <LoadingScreen
      v-else-if="authState === 'loading'"
      @validation-complete="handleValidationComplete" />

    <LoginScreen
      v-else-if="authState === 'login'"
      @login-success="handleLoginSuccess" />

    <LiveDock v-else @logout="handleLogout" />
  </div>
</template>
