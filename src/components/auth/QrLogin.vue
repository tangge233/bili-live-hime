<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import QrcodeVue from "qrcode.vue";
import { getLoginQrcode, pollQrCodeStatus } from "@/api/passport";

type QRStatus =
  "loading" | "pending" | "scanned" | "confirmed" | "expired" | "error";

const emit = defineEmits<{
  success: [];
}>();

const status = ref<QRStatus>("loading");
const qrCodeUrl = ref("");
const qrCodeKey = ref("");

let pollTimer: ReturnType<typeof setInterval> | null = null;
let expireTimer: ReturnType<typeof setTimeout> | null = null;
let polling = false;

function stopTimers() {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
  if (expireTimer) {
    clearTimeout(expireTimer);
    expireTimer = null;
  }
}

function startPolling() {
  stopTimers();

  expireTimer = setTimeout(() => {
    if (status.value === "pending") {
      status.value = "expired";
      stopTimers();
    }
  }, 180_000);

  pollTimer = setInterval(async () => {
    if (polling || !qrCodeKey.value) return;
    polling = true;

    try {
      const data = await pollQrCodeStatus(qrCodeKey.value);
      if (data.code === 0) {
        status.value = "confirmed";
        stopTimers();
        emit("success");
      } else if (data.code === 86038) {
        status.value = "expired";
        stopTimers();
      } else if (data.code === 86090) {
        status.value = "scanned";
      }
    } catch (error) {
      console.error("Polling error:", error);
    } finally {
      polling = false;
    }
  }, 1000);
}

async function generateQRCode() {
  stopTimers();
  status.value = "loading";
  qrCodeUrl.value = "";
  qrCodeKey.value = "";

  try {
    const qrcode = await getLoginQrcode();
    qrCodeUrl.value = qrcode.url;
    qrCodeKey.value = qrcode.qrcode_key;
    status.value = "pending";
    startPolling();
  } catch (error) {
    console.error("Failed to generate QR code:", error);
    status.value = "error";
  }
}

onMounted(() => {
  void generateQRCode();
});

onUnmounted(stopTimers);

const statusText = () => {
  switch (status.value) {
    case "loading":
      return "生成二维码中…";
    case "pending":
      return "请使用哔哩哔哩 App 扫码";
    case "scanned":
      return "已扫码，请在手机上确认";
    case "confirmed":
      return "登入成功！";
    case "expired":
      return "二维码已失效";
    case "error":
      return "二维码生成失败";
  }
};
</script>

<template>
  <div class="space-y-3">
    <div
      class="relative mx-auto aspect-square w-full max-w-[190px] overflow-hidden rounded-xl border border-default bg-white p-2">
      <QrcodeVue
        v-if="qrCodeUrl && status !== 'expired' && status !== 'error'"
        :value="qrCodeUrl"
        :size="176"
        level="M"
        render-as="svg"
        class="h-full w-full" />

      <div
        v-if="status === 'loading'"
        class="absolute inset-0 flex items-center justify-center bg-elevated/90">
        <UIcon
          name="i-lucide-loader-circle"
          class="size-6 animate-spin text-primary" />
      </div>

      <div
        v-else-if="status === 'expired' || status === 'error'"
        class="absolute inset-0 flex flex-col items-center justify-center gap-2 bg-elevated/95 text-center">
        <UIcon name="i-lucide-circle-x" class="size-7 text-error" />
        <UButton
          size="xs"
          variant="soft"
          icon="i-lucide-refresh-cw"
          label="刷新"
          @click="generateQRCode" />
      </div>

      <div
        v-else-if="status === 'scanned'"
        class="absolute inset-0 flex items-center justify-center bg-elevated/80">
        <UIcon name="i-lucide-circle-check-big" class="size-10 text-success" />
      </div>

      <div
        v-else-if="status === 'confirmed'"
        class="absolute inset-0 flex items-center justify-center bg-elevated/90">
        <UIcon name="i-lucide-circle-check-big" class="size-10 text-success" />
      </div>
    </div>

    <div class="flex items-center justify-center gap-1.5 text-xs text-muted">
      <UIcon
        v-if="status === 'loading' || status === 'scanned'"
        name="i-lucide-loader-circle"
        class="size-3.5 animate-spin text-primary" />
      <UIcon
        v-else-if="status === 'confirmed'"
        name="i-lucide-circle-check"
        class="size-3.5 text-success" />
      <UIcon
        v-else-if="status === 'expired' || status === 'error'"
        name="i-lucide-circle-x"
        class="size-3.5 text-error" />
      <span>{{ statusText() }}</span>
    </div>

    <UButton
      v-if="status === 'pending' || status === 'scanned'"
      color="neutral"
      variant="soft"
      size="sm"
      block
      icon="i-lucide-refresh-cw"
      label="刷新二维码"
      @click="generateQRCode" />

    <p class="text-center text-[11px] leading-relaxed text-dimmed">
      打开哔哩哔哩 App，扫描二维码并确认登入。
    </p>
  </div>
</template>
