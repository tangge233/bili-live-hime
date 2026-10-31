<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useToast } from "@nuxt/ui/composables";
import { addRoomAdmin, deleteRoomAdmin, getRoomAdmins } from "@/api/live";

interface Admin {
  uid: number;
  uname: string;
  face: string;
  ctime: string;
}

type Status = "loading" | "error" | "empty" | "success";

const toast = useToast();

const state = ref<Status>("loading");
const admins = ref<Admin[]>([]);
const maxCount = ref(0);
const adminInput = ref("");

const confirmOpen = ref(false);
const revokeOpen = ref(false);
const pendingAdd = ref("");
const pendingRevoke = ref<Admin | null>(null);
const submitting = ref(false);

function showError(error: unknown) {
  toast.add({
    title: "操作失败",
    description: error instanceof Error ? error.message : String(error),
    color: "error",
    icon: "i-lucide-circle-alert",
  });
}

async function fetchAllAdmins() {
  state.value = "loading";
  try {
    const firstPage = await getRoomAdmins(1);
    maxCount.value = firstPage.max_room_anchors_number;

    const pages = [];
    for (let page = 2; page <= firstPage.page.total_page; page++) {
      pages.push(getRoomAdmins(page));
    }
    const remaining = await Promise.all(pages);
    admins.value = [
      ...(firstPage.data ?? []),
      ...remaining.flatMap((page) => page.data ?? []),
    ];
    state.value = admins.value.length > 0 ? "success" : "empty";
  } catch (error) {
    state.value = "error";
    showError(error);
  }
}

function requestAppoint() {
  const value = adminInput.value.trim();
  if (!value) return;
  pendingAdd.value = value;
  confirmOpen.value = true;
}

async function confirmAppoint() {
  if (!pendingAdd.value) return;
  submitting.value = true;
  try {
    await addRoomAdmin(pendingAdd.value);
    adminInput.value = "";
    pendingAdd.value = "";
    confirmOpen.value = false;
    await fetchAllAdmins();
  } catch (error) {
    showError(error);
  } finally {
    submitting.value = false;
  }
}

function requestRevoke(admin: Admin) {
  pendingRevoke.value = admin;
  revokeOpen.value = true;
}

async function confirmRevoke() {
  if (!pendingRevoke.value) return;
  submitting.value = true;
  try {
    await deleteRoomAdmin(String(pendingRevoke.value.uid));
    pendingRevoke.value = null;
    revokeOpen.value = false;
    await fetchAllAdmins();
  } catch (error) {
    showError(error);
  } finally {
    submitting.value = false;
  }
}

onMounted(() => {
  void fetchAllAdmins();
});
</script>

<template>
  <div class="space-y-4">
    <section class="space-y-2">
      <label class="text-xs font-medium text-toned">任命房管</label>
      <div class="flex gap-1.5">
        <UInput
          v-model="adminInput"
          size="sm"
          placeholder="输入 UID 或用户名"
          class="flex-1"
          @keydown.enter="requestAppoint" />
        <UButton
          size="sm"
          label="任命"
          :disabled="!adminInput.trim()"
          @click="requestAppoint" />
      </div>
    </section>

    <section class="space-y-2">
      <div class="flex items-center justify-between">
        <span class="text-xs font-medium text-toned">当前房管</span>
        <UBadge color="neutral" variant="soft" size="sm">
          {{ admins.length }} / {{ maxCount }}
        </UBadge>
      </div>

      <div
        class="max-h-[46vh] overflow-y-auto rounded-lg border border-default">
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
            @click="fetchAllAdmins" />
        </div>

        <div
          v-else-if="state === 'empty'"
          class="p-6 text-center text-xs text-muted">
          尚未任命房管
        </div>

        <div v-else class="divide-y divide-default">
          <div
            v-for="admin in admins"
            :key="admin.uid"
            class="flex items-center justify-between gap-2 p-2.5 hover:bg-muted/30">
            <div class="flex min-w-0 items-center gap-2">
              <UAvatar
                :src="admin.face || '/akarin.webp'"
                :alt="admin.uname"
                size="sm" />
              <div class="min-w-0">
                <p class="truncate text-[11px] font-medium text-highlighted">
                  {{ admin.uname }}
                  <span class="font-normal text-muted"
                    >(UID: {{ admin.uid }})</span
                  >
                </p>
                <p class="flex items-center gap-1 text-[9px] text-dimmed">
                  <UIcon name="i-lucide-clock" class="size-2.5" />
                  任命时间：{{ admin.ctime }}
                </p>
              </div>
            </div>
            <UButton
              color="error"
              variant="ghost"
              size="xs"
              label="撤销"
              @click="requestRevoke(admin)" />
          </div>
        </div>
      </div>
    </section>

    <UModal v-model:open="confirmOpen" title="任命房管">
      <template #body>
        <p class="text-xs leading-relaxed text-toned">
          确定任命
          <strong class="text-highlighted">{{ pendingAdd }}</strong>
          为你的房管吗？房管可禁言用户，设置直播间屏蔽词。
        </p>
      </template>
      <template #footer>
        <UButton
          color="neutral"
          variant="outline"
          size="sm"
          label="取消"
          @click="confirmOpen = false" />
        <UButton
          size="sm"
          label="确定"
          :loading="submitting"
          @click="confirmAppoint" />
      </template>
    </UModal>

    <UModal v-model:open="revokeOpen" title="撤销房管">
      <template #body>
        <p class="text-xs leading-relaxed text-toned">
          确定撤销
          <strong class="text-highlighted">
            {{ pendingRevoke?.uname }} ({{ pendingRevoke?.uid }})
          </strong>
          ？
        </p>
      </template>
      <template #footer>
        <UButton
          color="neutral"
          variant="outline"
          size="sm"
          label="取消"
          @click="revokeOpen = false" />
        <UButton
          color="error"
          size="sm"
          label="撤销"
          :loading="submitting"
          @click="confirmRevoke" />
      </template>
    </UModal>
  </div>
</template>
