<script setup lang="ts">
import { ref } from "vue";
import QrLogin from "@/components/auth/QrLogin.vue";
import CookieLogin from "@/components/auth/CookieLogin.vue";

type LoginMethod = "qr" | "cookie";

const emit = defineEmits<{
  success: [];
}>();

const method = ref<LoginMethod>("qr");

const items = [
  { label: "二维码", icon: "i-lucide-qr-code", value: "qr" },
  { label: "Cookie", icon: "i-lucide-cookie", value: "cookie" },
];
</script>

<template>
  <div class="flex min-h-0 flex-1 items-center justify-center p-4">
    <div class="w-full space-y-4">
      <div class="space-y-1 text-center">
        <div
          class="mx-auto grid size-11 place-items-center rounded-2xl bg-primary/10 text-primary">
          <UIcon name="i-simple-icons:bilibili" class="size-6" />
        </div>
        <h1 class="text-base font-semibold text-highlighted">登录直播姬</h1>
        <p class="text-xs text-muted">扫码或粘贴 Cookie，开始你的直播控制台</p>
      </div>

      <div class="rounded-xl border border-default bg-elevated/40 p-3">
        <UTabs
          v-model="method"
          :items="items"
          :content="false"
          variant="pill"
          size="sm"
          class="mb-3 w-full" />

        <QrLogin v-if="method === 'qr'" @success="emit('success')" />
        <CookieLogin v-else @success="emit('success')" />
      </div>

      <p class="text-center text-[10px] text-dimmed">
        非官方实现 · 与哔哩哔哩官方无关联
      </p>
    </div>
  </div>
</template>
