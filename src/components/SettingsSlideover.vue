<script setup lang="ts">
import { computed } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import SlideoverHeader from "@/components/SlideoverHeader.vue";
import { useConfigStore } from "@/stores/config";

const open = defineModel<boolean>("open", { default: false });

const configStore = useConfigStore();
const dark = computed(() => configStore.config.theme === "dark");

function setDark(value: boolean) {
  configStore.updateConfig({ theme: value ? "dark" : "light" });
}

function openExternal(url: string) {
  void openUrl(url);
}
</script>

<template>
  <USlideover
    v-model:open="open"
    :close="false"
    title="设置"
    description="外观与关于"
    :ui="{
      content: 'w-full max-w-none',
      body: 'space-y-4 p-3',
    }">
    <template #header="{ close }">
      <SlideoverHeader title="设置" description="外观与关于" @back="close" />
    </template>

    <template #body>
      <section class="rounded-xl border border-default bg-elevated/30 p-3">
        <div class="flex items-center justify-between gap-3">
          <div class="flex items-center gap-2.5">
            <div
              class="grid size-8 place-items-center rounded-lg bg-primary/10 text-primary">
              <UIcon
                :name="dark ? 'i-lucide-moon' : 'i-lucide-sun'"
                class="size-4" />
            </div>
            <div>
              <p class="text-xs font-medium text-highlighted">深色模式</p>
              <p class="text-[10px] text-muted">跟随你的习惯切换主题</p>
            </div>
          </div>
          <USwitch :model-value="dark" @update:model-value="setDark" />
        </div>
      </section>

      <section class="space-y-1">
        <h3
          class="px-1 text-[10px] font-semibold tracking-wider text-dimmed uppercase">
          链接
        </h3>
        <UButton
          color="neutral"
          variant="ghost"
          block
          class="justify-between px-2 py-2.5"
          @click="
            openExternal('https://github.com/Rsplwe/bili-live-hime/issues')
          ">
          <span class="flex items-center gap-2.5">
            <UIcon name="i-lucide-bug" class="size-4 text-muted" />
            <span class="text-xs">报告错误</span>
          </span>
          <UIcon name="i-lucide-external-link" class="size-3.5 text-dimmed" />
        </UButton>
        <UButton
          color="neutral"
          variant="ghost"
          block
          class="justify-between px-2 py-2.5"
          @click="openExternal('https://github.com/Rsplwe/bili-live-hime')">
          <span class="flex items-center gap-2.5">
            <UIcon name="i-lucide-folder-git-2" class="size-4 text-muted" />
            <span class="text-xs">项目主页</span>
          </span>
          <UIcon name="i-lucide-external-link" class="size-3.5 text-dimmed" />
        </UButton>
      </section>

      <USeparator />

      <section class="space-y-1 px-1 text-center">
        <p class="text-[10px] leading-relaxed text-muted">
          本项目为
          <span class="font-semibold text-toned">非官方实现</span>
          ，与哔哩哔哩官方无任何关联
        </p>
        <p class="text-[10px] text-dimmed">哔哩哔哩直播姬（仮）</p>
        <p class="text-[10px] text-dimmed">Created by Rsplwe</p>
      </section>
    </template>
  </USlideover>
</template>
