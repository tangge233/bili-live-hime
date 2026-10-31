import { defineStore } from "pinia";
import { ref } from "vue";
import { emit } from "@tauri-apps/api/event";
import { ConfigManager } from "@/utils/config-manager";
import { debounce } from "@/lib/debounce";
import { type AppConfig, DEFAULT_CONFIG, type AppCookie } from "@/types/config";
import { useWsStore } from "@/stores/ws";

function createEmptyAuthConfig(config: AppConfig): AppConfig {
  return {
    ...DEFAULT_CONFIG,
    // Theme and cached area metadata are not account credentials, so keep
    // them across logout instead of resetting the user's preferences.
    areaList: config.areaList,
    theme: config.theme,
  };
}

export const useConfigStore = defineStore("config", () => {
  const config = ref<AppConfig>({ ...DEFAULT_CONFIG });
  const isInitialized = ref(false);

  let pendingInit: Promise<void> | null = null;

  const persist = debounce(
    (next: AppConfig) => {
      void ConfigManager.save(next).catch((error) => {
        console.error("保存配置失败:", error);
      });
    },
    1000,
    5000,
  );

  async function init() {
    if (pendingInit) return pendingInit;

    pendingInit = (async () => {
      const savedConfig = await ConfigManager.load();
      config.value = savedConfig;
      isInitialized.value = true;
    })().finally(() => {
      pendingInit = null;
    });

    return pendingInit;
  }

  function updateConfig(patch: Partial<AppConfig>) {
    const newConfig = { ...config.value, ...patch };
    config.value = newConfig;
    persist(newConfig);
  }

  function setCookies(cookies: AppCookie[]) {
    updateConfig({ cookies });
  }

  function addCookie(newCookie: AppCookie) {
    const currentCookies = config.value.cookies;
    const index = currentCookies.findIndex((c) => c.name === newCookie.name);

    const newCookies = [...currentCookies];
    if (index > -1) {
      newCookies[index] = newCookie;
    } else {
      newCookies.push(newCookie);
    }

    updateConfig({ cookies: newCookies });
  }

  function resetAuth() {
    config.value = createEmptyAuthConfig(config.value);
  }

  async function clearAuth() {
    // A pending debounced save would immediately write the old credentials
    // back after logout, so cancel it before clearing local state.
    persist.cancel();

    const emptyConfig = createEmptyAuthConfig(config.value);
    config.value = emptyConfig;

    const ws = useWsStore();
    // Always tell the backend to stop the danmaku task; local state may be
    // stale if the connection was established by the other window.
    try {
      await ws.disconnect();
    } catch (error) {
      console.error("Failed to disconnect comments:", error);
    }
    ws.reset();

    try {
      await ConfigManager.save(emptyConfig);
      // Delete the credential-store entry entirely instead of leaving an
      // empty secret object behind.
      await ConfigManager.clearSecrets();
    } catch (error) {
      console.error("清除登录信息失败:", error);
      throw error;
    } finally {
      // The floating-comments window has its own store. Tell it to drop the
      // credentials and any danmaku connection it may have reopened.
      try {
        await emit("auth-cleared");
      } catch (error) {
        console.error("Failed to notify other windows about logout:", error);
      }
    }
  }

  function getCookieString(): string {
    const cookies = config.value.cookies;
    if (!cookies || cookies.length === 0) return "";
    return cookies.map((c) => `${c.name}=${c.value}`).join("; ");
  }

  function getCookie(name: string): string | null {
    const cookie = config.value.cookies.find((c) => c.name === name);
    return cookie ? cookie.value : null;
  }

  return {
    config,
    isInitialized,
    init,
    updateConfig,
    setCookies,
    addCookie,
    resetAuth,
    clearAuth,
    getCookieString,
    getCookie,
  };
});
