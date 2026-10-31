import { onUnmounted, ref, watch, type Ref } from "vue";
import { getContributionRank, type ContributionRankItem } from "@/api/live";

const REFRESH_INTERVAL = 5000;

export function useContributionRank(enabled: Ref<boolean>) {
  const items = ref<ContributionRankItem[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  let timer: ReturnType<typeof setInterval> | null = null;
  let requestId = 0;

  const stopTimer = () => {
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
  };

  async function refresh() {
    const currentRequest = ++requestId;
    loading.value = true;
    error.value = null;

    try {
      const data = await getContributionRank();
      if (currentRequest === requestId) {
        items.value = data.item ?? [];
      }
    } catch (cause) {
      if (currentRequest === requestId) {
        error.value = cause instanceof Error ? cause.message : "在线榜加载失败";
      }
    } finally {
      if (currentRequest === requestId) {
        loading.value = false;
      }
    }
  }

  watch(
    enabled,
    (isEnabled) => {
      stopTimer();
      if (isEnabled) {
        void refresh();
        timer = setInterval(() => void refresh(), REFRESH_INTERVAL);
      }
    },
    { immediate: true },
  );

  onUnmounted(() => {
    requestId += 1;
    stopTimer();
  });

  return { items, loading, error, refresh };
}
