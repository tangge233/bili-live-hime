import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { useToast } from "@nuxt/ui/composables";
import { useConfigStore } from "@/stores/config";
import type { Stream } from "@/types/config";
import {
  getLiveVersion,
  startLive,
  stopLive,
  updateRoomArea,
  updateRoomTitle,
} from "@/api/live";

export const useLiveStore = defineStore("live", () => {
  const configStore = useConfigStore();
  const toast = useToast();

  const starting = ref(false);
  const stopping = ref(false);
  const qrOpen = ref(false);
  const qrUrl = ref("");

  const canStart = computed(() => {
    const config = configStore.config;
    return (
      !config.isOpenLive &&
      config.roomTitle.trim() !== "" &&
      config.categoryId !== "" &&
      config.areaId !== ""
    );
  });

  const activeStream = computed<Stream | undefined>(
    () => configStore.config.streams[0],
  );

  function showError(error: unknown) {
    const message = error instanceof Error ? error.message : String(error);
    toast.add({
      title: "操作失败",
      description: message,
      color: "error",
      icon: "i-lucide-circle-alert",
    });
  }

  async function saveTitle() {
    if (configStore.config.roomTitle.trim() === "") return;
    await updateRoomTitle(configStore.config.roomTitle);
    toast.add({
      title: "标题已更新",
      color: "success",
      icon: "i-lucide-check",
    });
  }

  async function saveArea() {
    if (configStore.config.areaId === "") return;
    await updateRoomArea(configStore.config.areaId);
    toast.add({
      title: "分区已更新",
      color: "success",
      icon: "i-lucide-check",
    });
  }

  async function startStream() {
    if (!canStart.value || starting.value) return;

    starting.value = true;
    try {
      const version = await getLiveVersion();
      await updateRoomTitle(configStore.config.roomTitle);

      const result = await startLive(
        version.curr_version,
        String(version.build),
      );

      if (result.code === 60024) {
        qrUrl.value = result.data.qr;
        qrOpen.value = true;
        return;
      }

      if (result.code === 60043) {
        qrUrl.value = `https://www.bilibili.com/blackboard/live/face-auth-middle.html?source_event=400&mid=${configStore.config.uid}`;
        qrOpen.value = true;
        return;
      }

      if (result.code !== 0) {
        throw new Error("开始直播失败：" + result.message);
      }

      let rtmp = 1;
      let srt = 0;
      const streams: Stream[] = [
        {
          type: "rtmp-1",
          address: result.data.rtmp.addr,
          key: result.data.rtmp.code,
        },
      ];

      for (const protocol of result.data.protocols) {
        if (protocol.protocol === "rtmp" && protocol.addr && protocol.code) {
          rtmp += 1;
          streams.push({
            type: `rtmp-${rtmp}`,
            address: protocol.addr,
            key: protocol.code,
          });
        }
        if (protocol.protocol === "srt" && protocol.addr && protocol.code) {
          srt += 1;
          streams.push({
            type: `srt-${srt}`,
            address: protocol.addr,
            key: protocol.code,
          });
        }
      }

      streams.sort((a, b) => a.type.localeCompare(b.type));
      configStore.updateConfig({ streams, isOpenLive: true });
      toast.add({
        title: "直播已开始",
        description: "推流地址和密钥已准备好",
        color: "success",
        icon: "i-lucide-radio",
      });
    } catch (error) {
      showError(error);
    } finally {
      starting.value = false;
    }
  }

  async function stopStream() {
    if (stopping.value) return;

    stopping.value = true;
    try {
      await stopLive();
      configStore.updateConfig({ isOpenLive: false, streams: [] });
      toast.add({
        title: "直播已停止",
        color: "success",
        icon: "i-lucide-square",
      });
    } catch (error) {
      showError(error);
    } finally {
      stopping.value = false;
    }
  }

  async function copy(text: string, label = "已复制") {
    try {
      await navigator.clipboard.writeText(text);
      toast.add({
        title: label,
        color: "success",
        icon: "i-lucide-copy-check",
      });
    } catch (error) {
      showError(error);
    }
  }

  return {
    starting,
    stopping,
    qrOpen,
    qrUrl,
    canStart,
    activeStream,
    saveTitle,
    saveArea,
    startStream,
    stopStream,
    copy,
  };
});
