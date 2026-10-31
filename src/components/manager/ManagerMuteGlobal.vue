<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useToast } from "@nuxt/ui/composables";
import { getRoomSilent, setRoomSilent, type RoomSilentType } from "@/api/live";
import { useCountdown } from "@/composables/useCountdown";

type Status = "loading" | "error" | "success";
type Duration = "30min" | "60min" | "permanent";

interface GlobalMuteSettings {
  type: RoomSilentType;
  wealthLevel: number;
  medalLevel: number;
  duration: Duration;
}

const toast = useToast();

const state = ref<Status>("loading");
const enabled = ref(true);
const submitting = ref(false);
const globalMute = ref<GlobalMuteSettings>({
  type: "off",
  wealthLevel: 1,
  medalLevel: 1,
  duration: "30min",
});

const countdown = useCountdown(0, () => {
  enabled.value = true;
});

const durationItems = [
  { label: "30 分钟", value: "30min" },
  { label: "60 分钟", value: "60min" },
  { label: "永久", value: "permanent" },
];

function format(seconds: number) {
  const minutes = Math.floor(seconds / 60);
  const rest = seconds % 60;
  return `${minutes}:${rest.toString().padStart(2, "0")}`;
}

function showError(error: unknown) {
  toast.add({
    title: "操作失败",
    description: error instanceof Error ? error.message : String(error),
    color: "error",
    icon: "i-lucide-circle-alert",
  });
}

async function fetchConfig() {
  state.value = "loading";
  try {
    const config = await getRoomSilent();
    const type = config.type === "" ? "off" : config.type;
    globalMute.value = {
      type,
      wealthLevel: type === "wealth" ? config.level : 1,
      medalLevel: type === "medal" ? config.level : 1,
      duration: "30min",
    };
    enabled.value = type === "off";
    if (type !== "off") {
      countdown.reset();
      countdown.setTimeLeft(config.second);
      countdown.start();
    }
    state.value = "success";
  } catch (error) {
    state.value = "error";
    showError(error);
  }
}

async function applySilent() {
  submitting.value = true;
  try {
    let level = 1;
    if (globalMute.value.type === "wealth") {
      level = globalMute.value.wealthLevel;
    } else if (globalMute.value.type === "medal") {
      level = globalMute.value.medalLevel;
    }

    const duration =
      globalMute.value.duration === "30min"
        ? 30
        : globalMute.value.duration === "60min"
          ? 60
          : 0;

    await setRoomSilent(globalMute.value.type, level, duration);
    countdown.reset();
    countdown.setTimeLeft(duration * 60);
    countdown.start();
    enabled.value = false;
  } catch (error) {
    showError(error);
  } finally {
    submitting.value = false;
  }
}

async function cancelSilent() {
  submitting.value = true;
  try {
    await setRoomSilent("off");
    countdown.reset();
    enabled.value = true;
  } catch (error) {
    showError(error);
  } finally {
    submitting.value = false;
  }
}

onMounted(() => {
  void fetchConfig();
});
</script>

<template>
  <div
    v-if="state === 'loading'"
    class="flex items-center justify-center gap-2 p-6 text-xs text-muted">
    <UIcon
      name="i-lucide-loader-circle"
      class="size-4 animate-spin text-primary" />
    加载中…
  </div>

  <div
    v-else-if="state === 'error'"
    class="flex flex-col items-center justify-center gap-3 p-6 text-center">
    <UIcon name="i-lucide-circle-x" class="size-5 text-error" />
    <div>
      <p class="text-xs font-medium">加载失败</p>
      <p class="text-[10px] text-muted">网络异常或服务器错误</p>
    </div>
    <UButton
      color="neutral"
      variant="outline"
      size="xs"
      icon="i-lucide-refresh-cw"
      label="刷新"
      @click="fetchConfig" />
  </div>

  <div v-else class="space-y-4">
    <section class="space-y-2">
      <span class="text-xs font-medium text-toned">全局禁言规则</span>

      <div class="space-y-2 rounded-lg border border-default p-2.5">
        <button
          type="button"
          class="flex w-full items-center gap-2.5 rounded-md p-2.5 text-left transition-colors"
          :class="
            globalMute.type === 'wealth'
              ? 'border border-primary bg-primary/10'
              : 'hover:bg-muted/60'
          "
          @click="globalMute.type = 'wealth'">
          <span
            class="grid size-4 place-items-center rounded-full border-2"
            :class="
              globalMute.type === 'wealth'
                ? 'border-primary'
                : 'border-muted-foreground'
            ">
            <span
              v-if="globalMute.type === 'wealth'"
              class="size-2 rounded-full bg-primary" />
          </span>
          <span class="text-xs">荣耀等级</span>
        </button>

        <div v-if="globalMute.type === 'wealth'" class="space-y-2 px-2 pb-1">
          <div class="flex items-center justify-between text-[11px]">
            <span class="text-muted">阈值</span>
            <span class="font-medium text-toned">
              {{ globalMute.wealthLevel }} 以下（不包含该等级）
            </span>
          </div>
          <USlider
            v-model="globalMute.wealthLevel"
            :min="1"
            :max="80"
            :step="1" />
          <div class="flex justify-between text-[9px] text-dimmed">
            <span>1</span>
            <span>40</span>
            <span>80</span>
          </div>
        </div>

        <button
          type="button"
          class="flex w-full items-center gap-2.5 rounded-md p-2.5 text-left transition-colors"
          :class="
            globalMute.type === 'medal'
              ? 'border border-primary bg-primary/10'
              : 'hover:bg-muted/60'
          "
          @click="globalMute.type = 'medal'">
          <span
            class="grid size-4 place-items-center rounded-full border-2"
            :class="
              globalMute.type === 'medal'
                ? 'border-primary'
                : 'border-muted-foreground'
            ">
            <span
              v-if="globalMute.type === 'medal'"
              class="size-2 rounded-full bg-primary" />
          </span>
          <span class="text-xs">粉丝勋章</span>
        </button>

        <div v-if="globalMute.type === 'medal'" class="space-y-2 px-2 pb-1">
          <div class="flex items-center justify-between text-[11px]">
            <span class="text-muted">阈值</span>
            <span class="font-medium text-toned">
              {{ globalMute.medalLevel }} 以下
            </span>
          </div>
          <USlider
            v-model="globalMute.medalLevel"
            :min="1"
            :max="120"
            :step="1" />
          <div class="flex justify-between text-[9px] text-dimmed">
            <span>1</span>
            <span>60</span>
            <span>120</span>
          </div>
        </div>

        <button
          type="button"
          class="flex w-full items-center gap-2.5 rounded-md p-2.5 text-left transition-colors"
          :class="
            globalMute.type === 'member'
              ? 'border border-error bg-error/10'
              : 'hover:bg-muted/60'
          "
          @click="globalMute.type = 'member'">
          <span
            class="grid size-4 place-items-center rounded-full border-2"
            :class="
              globalMute.type === 'member'
                ? 'border-error'
                : 'border-muted-foreground'
            ">
            <span
              v-if="globalMute.type === 'member'"
              class="size-2 rounded-full bg-error" />
          </span>
          <span class="text-xs">全员</span>
        </button>
      </div>
    </section>

    <section class="space-y-1.5">
      <label class="text-xs font-medium text-toned">禁言时间</label>
      <USelect
        v-model="globalMute.duration"
        :items="durationItems"
        value-key="value"
        class="w-full" />
    </section>

    <section v-if="!enabled" class="space-y-2">
      <div
        class="flex items-center justify-between rounded-lg border border-error bg-error/10 p-2.5">
        <span class="text-[11px] font-medium text-error"
          >禁言已开启，剩余时长</span
        >
        <span
          class="flex items-center gap-1.5 text-[11px] font-bold text-error tabular-nums">
          <UIcon name="i-lucide-clock" class="size-3.5" />
          {{
            globalMute.duration === "permanent"
              ? "永久"
              : format(countdown.timeLeft.value)
          }}
        </span>
      </div>
      <UButton
        block
        color="error"
        size="sm"
        label="取消禁言"
        :loading="submitting"
        @click="cancelSilent" />
    </section>

    <UButton
      v-else
      block
      size="sm"
      label="开始禁言"
      :disabled="globalMute.type === 'off'"
      :loading="submitting"
      @click="applySilent" />
  </div>
</template>
