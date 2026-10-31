<script setup lang="ts">
import { ref } from "vue";
import { useConfigStore } from "@/stores/config";
import { sanitizeCookies } from "@/utils/config-manager";
import type { AppCookie } from "@/types/config";

const emit = defineEmits<{
  success: [];
}>();

const configStore = useConfigStore();

const cookie = ref("");
const loading = ref(false);
const error = ref<string | null>(null);

function parseCookie(raw: string): AppCookie[] {
  const result = new Map<string, string>();

  raw
    .split(";")
    .map((item) => item.trim())
    .filter(Boolean)
    .forEach((pair) => {
      const eqIndex = pair.indexOf("=");
      if (eqIndex === -1) return;

      const key = pair.slice(0, eqIndex).trim();
      const value = pair.slice(eqIndex + 1).trim();

      if (key && value) {
        result.set(key, value);
      }
    });

  return sanitizeCookies([...result].map(([name, value]) => ({ name, value })));
}

function handleSubmit() {
  if (!cookie.value.trim()) {
    error.value = "请输入您的小饼干~";
    return;
  }

  loading.value = true;
  error.value = null;

  const parsed = parseCookie(cookie.value);
  const values = new Map(parsed.map(({ name, value }) => [name, value]));
  const missing = ["SESSDATA", "bili_jct"].filter((key) => !values.get(key));

  if (missing.length > 0) {
    error.value = "Cookie 无效。请检查并重试。";
    loading.value = false;
    return;
  }

  configStore.setCookies(parsed);
  emit("success");
}
</script>

<template>
  <form class="space-y-3" @submit.prevent="handleSubmit">
    <UFormField label="Session Cookie">
      <UTextarea
        v-model="cookie"
        :rows="4"
        placeholder="将您的 Cookie 粘贴到此处……"
        class="w-full font-mono text-xs"
        @update:model-value="error = null" />
    </UFormField>

    <UAlert
      v-if="error"
      color="error"
      variant="soft"
      icon="i-lucide-circle-alert"
      :title="error" />

    <UButton
      type="submit"
      block
      :loading="loading"
      :disabled="!cookie.trim()"
      icon="i-lucide-cookie"
      label="通过 Cookie 登入" />

    <div class="rounded-lg bg-muted/60 p-2.5">
      <p class="text-[11px] leading-relaxed text-muted">
        <strong class="text-toned">如何获得 Cookie：</strong><br />
        1. 在浏览器中登入哔哩哔哩网站<br />
        2. 打开 Developer Tools (F12)<br />
        3. 找到网络请求<br />
        4. 复制以 "api.bilibili.com" 域请求的 Cookie 字段内容
      </p>
    </div>
  </form>
</template>
