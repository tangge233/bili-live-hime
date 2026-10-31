<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useToast } from "@nuxt/ui/composables";
import {
  addBlockedWords,
  deleteBlockedWords,
  getBlockedWords,
} from "@/api/live";

type Status = "loading" | "error" | "empty" | "success";

const toast = useToast();

const state = ref<Status>("loading");
const maxLimit = ref(0);
const blockedWords = ref<string[]>([]);
const blockedWordInput = ref("");
const adding = ref(false);

function showError(error: unknown) {
  toast.add({
    title: "操作失败",
    description: error instanceof Error ? error.message : String(error),
    color: "error",
    icon: "i-lucide-circle-alert",
  });
}

async function fetchKeywords() {
  state.value = "loading";
  try {
    const { keyword_list, max_limit } = await getBlockedWords();
    blockedWords.value = keyword_list.map((item) => item.keyword);
    maxLimit.value = max_limit;
    state.value = blockedWords.value.length > 0 ? "success" : "empty";
  } catch (error) {
    state.value = "error";
    showError(error);
  }
}

async function addWord() {
  const raw = blockedWordInput.value.trim();
  if (!raw || adding.value) return;

  const word = raw.toLowerCase();
  if (blockedWords.value.includes(word)) {
    blockedWordInput.value = "";
    return;
  }

  adding.value = true;
  try {
    await addBlockedWords(word);
    blockedWords.value = [...blockedWords.value, word];
    blockedWordInput.value = "";
    state.value = "success";
  } catch (error) {
    showError(error);
  } finally {
    adding.value = false;
  }
}

async function removeWord(word: string) {
  try {
    await deleteBlockedWords(word);
    blockedWords.value = blockedWords.value.filter((item) => item !== word);
    if (blockedWords.value.length === 0) state.value = "empty";
  } catch (error) {
    showError(error);
  }
}

onMounted(() => {
  void fetchKeywords();
});
</script>

<template>
  <div class="space-y-4">
    <section class="space-y-2">
      <div>
        <p class="text-xs font-medium text-toned">屏蔽词</p>
        <p class="text-[10px] text-dimmed">
          包含屏蔽词的弹幕不会出现在直播中，屏蔽词不得超过 15 个字符。
        </p>
      </div>

      <div class="flex gap-1.5">
        <UInput
          v-model="blockedWordInput"
          size="sm"
          maxlength="15"
          placeholder="请输入要屏蔽的内容"
          class="flex-1"
          @keydown.enter="addWord" />
        <UButton
          size="sm"
          label="添加"
          :loading="adding"
          :disabled="!blockedWordInput.trim()"
          @click="addWord" />
      </div>
    </section>

    <section class="space-y-2">
      <div class="flex items-center justify-between">
        <span class="text-xs font-medium text-toned">屏蔽列表</span>
        <UBadge color="neutral" variant="soft" size="sm">
          {{ blockedWords.length }} / {{ maxLimit }}
        </UBadge>
      </div>

      <div class="min-h-36 rounded-lg border border-default p-3">
        <div
          v-if="state === 'loading'"
          class="flex items-center justify-center gap-2 p-4 text-xs text-muted">
          <UIcon
            name="i-lucide-loader-circle"
            class="size-4 animate-spin text-primary" />
          加载中…
        </div>

        <div
          v-else-if="state === 'error'"
          class="flex flex-col items-center justify-center gap-3 p-4 text-center">
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
            @click="fetchKeywords" />
        </div>

        <p
          v-else-if="state === 'empty'"
          class="py-6 text-center text-xs text-muted">
          屏蔽词列表为空
        </p>

        <div v-else class="flex flex-wrap gap-1.5">
          <UBadge
            v-for="word in blockedWords"
            :key="word"
            color="neutral"
            variant="soft"
            size="md"
            class="gap-1 py-1 pr-0.5 pl-2.5">
            {{ word }}
            <UButton
              color="neutral"
              variant="ghost"
              size="xs"
              square
              icon="i-lucide-x"
              :aria-label="`移除屏蔽词 ${word}`"
              class="size-4"
              @click="removeWord(word)" />
          </UBadge>
        </div>
      </div>
    </section>
  </div>
</template>
