import { onUnmounted, ref, watch, type Ref } from "vue";

interface UseCountdownReturn {
  timeLeft: Ref<number>;
  isRunning: Ref<boolean>;
  start: () => void;
  pause: () => void;
  reset: () => void;
  setTimeLeft: (time: number) => void;
}

export function useCountdown(
  initialTime: number,
  onComplete?: () => void,
): UseCountdownReturn {
  const timeLeft = ref(initialTime);
  const isRunning = ref(false);

  let timer: ReturnType<typeof setInterval> | null = null;

  const stopTimer = () => {
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
  };

  const tick = () => {
    if (timeLeft.value <= 1) {
      timeLeft.value = 0;
      isRunning.value = false;
      stopTimer();
      onComplete?.();
      return;
    }
    timeLeft.value -= 1;
  };

  watch(isRunning, (running) => {
    stopTimer();
    if (running && timeLeft.value > 0) {
      timer = setInterval(tick, 1000);
    }
  });

  onUnmounted(stopTimer);

  return {
    timeLeft,
    isRunning,
    start: () => {
      isRunning.value = true;
    },
    pause: () => {
      isRunning.value = false;
    },
    reset: () => {
      isRunning.value = false;
      stopTimer();
      timeLeft.value = initialTime;
    },
    setTimeLeft: (time: number) => {
      timeLeft.value = Math.max(0, time);
    },
  };
}
