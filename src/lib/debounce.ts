export type Debounced<TArgs extends unknown[]> = ((...args: TArgs) => void) & {
  cancel: () => void;
};

/**
 * Small debounce with a maximum wait, used for config persistence. The last
 * arguments always win and `cancel` drops a pending write before logout.
 */
export function debounce<TArgs extends unknown[]>(
  fn: (...args: TArgs) => void,
  wait: number,
  maxWait = wait,
): Debounced<TArgs> {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let maxTimer: ReturnType<typeof setTimeout> | null = null;
  let pendingArgs: TArgs | null = null;

  const clear = () => {
    if (timer) clearTimeout(timer);
    if (maxTimer) clearTimeout(maxTimer);
    timer = null;
    maxTimer = null;
  };

  const invoke = () => {
    const args = pendingArgs;
    pendingArgs = null;
    clear();
    if (args) fn(...args);
  };

  const debounced = ((...args: TArgs) => {
    pendingArgs = args;
    if (timer) clearTimeout(timer);
    timer = setTimeout(invoke, wait);
    if (!maxTimer) maxTimer = setTimeout(invoke, maxWait);
  }) as Debounced<TArgs>;

  debounced.cancel = () => {
    pendingArgs = null;
    clear();
  };

  return debounced;
}
