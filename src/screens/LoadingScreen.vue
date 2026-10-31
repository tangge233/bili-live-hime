<script setup lang="ts">
import { onUnmounted, ref, watch } from "vue";
import { useConfigStore } from "@/stores/config";
import { getUserInfo } from "@/api/user";
import { getAreaList, getRoomId, getRoomToken } from "@/api/live";

const emit = defineEmits<{
  "validation-complete": [isValid: boolean];
}>();

const configStore = useConfigStore();
const status = ref("正在检查已保存的凭据...");
let started = false;
let cancelled = false;

async function validateToken() {
  if (started || cancelled) return;
  started = true;

  const cookieString = configStore.getCookieString();
  if (cookieString === "" || configStore.getCookie("bili_jct") === null) {
    emit("validation-complete", false);
    return;
  }

  try {
    status.value = "正在验证令牌...";
    const userRes = await getUserInfo();
    if (cancelled) return;

    const uid = userRes.mid;
    const imgUrl = userRes.wbi_img.img_url;
    const subUrl = userRes.wbi_img.sub_url;

    configStore.updateConfig({
      uid,
      username: userRes.uname,
      avatar: userRes.face,
      img_url: imgUrl.slice(
        imgUrl.lastIndexOf("/") + 1,
        imgUrl.lastIndexOf("."),
      ),
      sub_url: subUrl.slice(
        subUrl.lastIndexOf("/") + 1,
        subUrl.lastIndexOf("."),
      ),
    });

    status.value = "获取直播间信息...";
    const roomIdRes = await getRoomId(uid);
    if (cancelled) return;
    configStore.updateConfig({ roomId: roomIdRes.room_id });

    status.value = "获取分区信息...";
    const areaRes = await getAreaList();
    if (cancelled) return;
    configStore.updateConfig({ areaList: areaRes });

    status.value = "获取消息流...";
    const roomTokenRes = await getRoomToken(roomIdRes.room_id);
    if (cancelled) return;
    configStore.updateConfig({ roomToken: roomTokenRes.token });

    status.value = "验证成功！";
    emit("validation-complete", true);
  } catch (error) {
    if (cancelled) return;
    status.value = "初始化错误：" + error;
    await new Promise((resolve) => setTimeout(resolve, 1000));
    await configStore.clearAuth().catch((clearError) => {
      console.error("Failed to clear authentication:", clearError);
    });
    if (!cancelled) {
      emit("validation-complete", false);
    }
  }
}

watch(
  () => configStore.isInitialized,
  (ready) => {
    if (ready) void validateToken();
  },
  { immediate: true },
);

onUnmounted(() => {
  cancelled = true;
});
</script>

<template>
  <div class="flex min-h-0 flex-1 items-center justify-center p-6">
    <div class="w-full max-w-xs space-y-5 text-center">
      <div
        class="mx-auto grid size-14 place-items-center rounded-2xl bg-primary/10 text-primary">
        <UIcon name="i-simple-icons:bilibili" class="size-7" />
      </div>

      <div class="space-y-1">
        <h1 class="text-base font-semibold text-highlighted">
          哔哩哔哩直播姬（仮）
        </h1>
        <p class="text-xs text-muted">正在准备直播控制台</p>
      </div>

      <div class="flex items-center justify-center gap-2 text-xs text-muted">
        <UIcon
          name="i-lucide-loader-circle"
          class="size-4 animate-spin text-primary" />
        <span>{{ status }}</span>
      </div>
    </div>
  </div>
</template>
