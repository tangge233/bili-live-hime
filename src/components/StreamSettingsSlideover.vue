<script setup lang="ts">
import { computed, ref } from "vue";
import { useToast } from "@nuxt/ui/composables";
import SlideoverHeader from "@/components/SlideoverHeader.vue";
import { useConfigStore } from "@/stores/config";
import { useLiveStore } from "@/stores/live";

const open = defineModel<boolean>("open", { default: false });

const configStore = useConfigStore();
const liveStore = useLiveStore();
const toast = useToast();

const savingTitle = ref(false);
const savingArea = ref(false);
const revealed = ref<Record<string, boolean>>({});

const config = computed(() => configStore.config);
const selectedParent = computed(() =>
  config.value.areaList.find((parent) => parent.id === config.value.categoryId),
);

const categoryItems = computed(() =>
  config.value.areaList.map((parent) => ({
    label: parent.name,
    value: parent.id,
  })),
);

const areaItems = computed(() =>
  (selectedParent.value?.list ?? []).map((area) => ({
    label: area.name,
    value: area.id,
  })),
);

const category = computed({
  get: () => config.value.categoryId,
  set: (value: string) => {
    configStore.updateConfig({ categoryId: value, areaId: "" });
  },
});

const area = computed({
  get: () => config.value.areaId,
  set: (value: string) => {
    configStore.updateConfig({ areaId: value });
  },
});

function setTitle(value: string) {
  configStore.updateConfig({ roomTitle: value });
}

async function saveTitle() {
  savingTitle.value = true;
  try {
    await liveStore.saveTitle();
  } catch (error) {
    toast.add({
      title: "更新标题失败",
      description: error instanceof Error ? error.message : String(error),
      color: "error",
      icon: "i-lucide-circle-alert",
    });
  } finally {
    savingTitle.value = false;
  }
}

async function saveArea() {
  savingArea.value = true;
  try {
    await liveStore.saveArea();
  } catch (error) {
    toast.add({
      title: "更新分区失败",
      description: error instanceof Error ? error.message : String(error),
      color: "error",
      icon: "i-lucide-circle-alert",
    });
  } finally {
    savingArea.value = false;
  }
}

function toggleKey(type: string) {
  revealed.value[type] = !revealed.value[type];
}
</script>

<template>
  <USlideover
    v-model:open="open"
    :close="false"
    title="开播设置"
    description="标题、分区与推流信息"
    :ui="{
      content: 'w-full max-w-none',
      body: 'space-y-4 p-3',
    }">
    <template #header="{ close }">
      <SlideoverHeader
        title="开播设置"
        description="标题、分区与推流信息"
        @back="close" />
    </template>

    <template #body>
      <div class="space-y-4">
        <section class="space-y-2">
          <UFormField
            label="直播间标题"
            :description="`${config.roomTitle.length} 字`">
            <div class="flex gap-1.5">
              <UInput
                :model-value="config.roomTitle"
                placeholder="请输入直播标题…"
                class="flex-1"
                @update:model-value="setTitle" />
              <UButton
                size="sm"
                square
                icon="i-lucide-check"
                aria-label="更新标题"
                :loading="savingTitle"
                :disabled="config.roomTitle.trim() === ''"
                @click="saveTitle" />
            </div>
          </UFormField>
        </section>

        <section class="space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-xs font-semibold text-toned">分区设置</h3>
            <UButton
              size="xs"
              variant="soft"
              icon="i-lucide-check"
              label="更新分区"
              :loading="savingArea"
              :disabled="config.areaId === ''"
              @click="saveArea" />
          </div>

          <UFormField label="分类">
            <USelect
              v-model="category"
              :items="categoryItems"
              placeholder="选择分类"
              class="w-full" />
          </UFormField>

          <UFormField label="子分区">
            <USelectMenu
              v-model="area"
              :items="areaItems"
              value-key="value"
              search-input
              placeholder="选择分区"
              :disabled="!config.categoryId"
              class="w-full" />
          </UFormField>
        </section>

        <div class="flex gap-1.5">
          <UButton
            v-if="!config.isOpenLive"
            block
            size="sm"
            icon="i-lucide-radio"
            label="开始直播"
            :loading="liveStore.starting"
            :disabled="!liveStore.canStart"
            @click="liveStore.startStream" />
          <UButton
            v-else
            block
            size="sm"
            color="error"
            icon="i-lucide-square"
            label="停止直播"
            :loading="liveStore.stopping"
            @click="liveStore.stopStream" />
        </div>

        <template v-if="config.isOpenLive && config.streams.length > 0">
          <USeparator label="推流信息" />

          <section
            v-for="stream in config.streams"
            :key="stream.type"
            class="space-y-2 rounded-lg border border-default bg-elevated/40 p-2.5">
            <div class="flex items-center justify-between">
              <UBadge color="primary" variant="soft" size="sm">
                {{ stream.type.toUpperCase() }}
              </UBadge>
              <UButton
                size="xs"
                variant="ghost"
                icon="i-lucide-copy"
                label="复制地址"
                @click="liveStore.copy(stream.address, '推流地址已复制')" />
            </div>

            <UFormField label="服务器地址">
              <UInput
                :model-value="stream.address"
                readonly
                size="sm"
                class="w-full font-mono text-[10px]" />
            </UFormField>

            <UFormField label="流密钥">
              <div class="flex gap-1.5">
                <UInput
                  :model-value="stream.key"
                  :type="revealed[stream.type] ? 'text' : 'password'"
                  readonly
                  size="sm"
                  class="flex-1 font-mono text-[10px]" />
                <UButton
                  size="sm"
                  square
                  color="neutral"
                  variant="outline"
                  :icon="
                    revealed[stream.type] ? 'i-lucide-eye-off' : 'i-lucide-eye'
                  "
                  :aria-label="
                    revealed[stream.type] ? '隐藏流密钥' : '显示流密钥'
                  "
                  @click="toggleKey(stream.type)" />
                <UButton
                  size="sm"
                  square
                  color="neutral"
                  variant="outline"
                  icon="i-lucide-copy"
                  aria-label="复制流密钥"
                  @click="liveStore.copy(stream.key, '流密钥已复制')" />
              </div>
            </UFormField>
          </section>
        </template>
      </div>
    </template>
  </USlideover>
</template>
