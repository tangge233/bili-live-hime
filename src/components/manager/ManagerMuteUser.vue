<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useToast } from "@nuxt/ui/composables";
import {
  addSilentUser,
  deleteSilentUser,
  getSilentUserList,
  searchUsers,
} from "@/api/live";
import UserSelectModal, {
  type UserSelectItem,
} from "@/components/manager/UserSelectModal.vue";

interface MutedUser {
  tuid: number;
  tname: string;
  uid: number;
  name: string;
  ctime: string;
  face: string;
  block_end_time: string;
}

type Status = "loading" | "error" | "empty" | "success";

const toast = useToast();

const state = ref<Status>("loading");
const mutedUsers = ref<MutedUser[]>([]);
const muteInput = ref("");
const searching = ref(false);

const userSelectOpen = ref(false);
const userList = ref<UserSelectItem[]>([]);
const selectedId = ref<string | null>(null);

const pendingMute = ref<{ id: string; username: string } | null>(null);
const confirmMuteOpen = ref(false);
const revokeTarget = ref<MutedUser | null>(null);
const revokeOpen = ref(false);
const submitting = ref(false);

function showError(error: unknown) {
  toast.add({
    title: "操作失败",
    description: error instanceof Error ? error.message : String(error),
    color: "error",
    icon: "i-lucide-circle-alert",
  });
}

async function fetchAllMutedUsers() {
  state.value = "loading";
  try {
    const firstPage = await getSilentUserList(1);
    const pages = [];
    for (let page = 2; page <= firstPage.total_page; page++) {
      pages.push(getSilentUserList(page));
    }
    const remaining = await Promise.all(pages);
    mutedUsers.value = [
      ...(firstPage.data ?? []),
      ...remaining.flatMap((page) => page.data ?? []),
    ];
    state.value = mutedUsers.value.length > 0 ? "success" : "empty";
  } catch (error) {
    state.value = "error";
    showError(error);
  }
}

async function search() {
  const keyword = muteInput.value.trim();
  if (!keyword || searching.value) return;

  searching.value = true;
  try {
    const { items } = await searchUsers(keyword);
    userList.value = items.map((user) => ({
      name: user.uname,
      id: String(user.uid),
      avatar: user.face,
    }));
    userSelectOpen.value = true;
  } catch (error) {
    showError(error);
  } finally {
    searching.value = false;
  }
}

function selectUser(user: UserSelectItem) {
  pendingMute.value = { id: user.id, username: user.name };
  confirmMuteOpen.value = true;
}

async function confirmMute() {
  if (!pendingMute.value) return;
  submitting.value = true;
  try {
    await addSilentUser(pendingMute.value.id, String(-1));
    muteInput.value = "";
    pendingMute.value = null;
    confirmMuteOpen.value = false;
    await fetchAllMutedUsers();
  } catch (error) {
    showError(error);
  } finally {
    submitting.value = false;
  }
}

function requestRevoke(user: MutedUser) {
  revokeTarget.value = user;
  revokeOpen.value = true;
}

async function confirmRevoke() {
  if (!revokeTarget.value) return;
  submitting.value = true;
  try {
    await deleteSilentUser(String(revokeTarget.value.tuid));
    revokeTarget.value = null;
    revokeOpen.value = false;
    await fetchAllMutedUsers();
  } catch (error) {
    showError(error);
  } finally {
    submitting.value = false;
  }
}

onMounted(() => {
  void fetchAllMutedUsers();
});
</script>

<template>
  <div class="space-y-4">
    <section class="space-y-2">
      <label class="text-xs font-medium text-toned">禁言用户</label>
      <div class="flex gap-1.5">
        <UInput
          v-model="muteInput"
          size="sm"
          placeholder="输入 UID 或用户名"
          class="flex-1"
          @keydown.enter="search" />
        <UButton
          size="sm"
          label="禁言"
          :loading="searching"
          :disabled="!muteInput.trim()"
          @click="search" />
      </div>
      <p class="text-[10px] text-dimmed">当前禁言时长固定为永久。</p>
    </section>

    <section class="space-y-2">
      <span class="text-xs font-medium text-toned">已禁言用户</span>

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
            @click="fetchAllMutedUsers" />
        </div>

        <div
          v-else-if="state === 'empty'"
          class="p-6 text-center text-xs text-muted">
          尚未有已禁言的用户
        </div>

        <div v-else class="divide-y divide-default">
          <div
            v-for="user in mutedUsers"
            :key="user.tuid"
            class="flex items-start justify-between gap-2 p-2.5 hover:bg-muted/30">
            <div class="flex min-w-0 items-start gap-2">
              <UAvatar
                :src="user.face || '/akarin.webp'"
                :alt="user.tname"
                size="sm" />
              <div class="min-w-0 space-y-0.5">
                <p class="truncate text-[11px] font-medium text-highlighted">
                  {{ user.tname }}
                  <span class="font-normal text-muted"
                    >(UID: {{ user.tuid }})</span
                  >
                </p>
                <p class="flex items-center gap-1 text-[9px] text-muted">
                  <UIcon name="i-lucide-clock" class="size-2.5" />
                  禁言至 {{ user.block_end_time }}
                </p>
                <p class="flex items-center gap-1 text-[9px] text-dimmed">
                  <UIcon name="i-lucide-user-x" class="size-2.5" />
                  {{ user.name }}
                  <span>·</span>
                  <UIcon name="i-lucide-calendar" class="size-2.5" />
                  {{ user.ctime }}
                </p>
              </div>
            </div>
            <UButton
              color="error"
              variant="ghost"
              size="xs"
              label="撤销"
              @click="requestRevoke(user)" />
          </div>
        </div>
      </div>
    </section>

    <UserSelectModal
      v-model:open="userSelectOpen"
      v-model:selected-id="selectedId"
      :users="userList"
      @confirm="selectUser" />

    <UModal v-model:open="confirmMuteOpen" title="确认禁言">
      <template #body>
        <p class="text-xs leading-relaxed text-toned">
          确认要永久禁言用户
          <strong class="text-highlighted">
            {{ pendingMute?.username }} (UID: {{ pendingMute?.id }})
          </strong>
          吗？
        </p>
      </template>
      <template #footer>
        <UButton
          color="neutral"
          variant="outline"
          size="sm"
          label="取消"
          @click="confirmMuteOpen = false" />
        <UButton
          color="error"
          size="sm"
          label="确认"
          :loading="submitting"
          @click="confirmMute" />
      </template>
    </UModal>

    <UModal v-model:open="revokeOpen" title="撤销禁言">
      <template #body>
        <p class="text-xs leading-relaxed text-toned">
          是否确认撤销
          <strong class="text-highlighted">
            {{ revokeTarget?.tname }} (UID: {{ revokeTarget?.tuid }})
          </strong>
          的禁言？
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
          size="sm"
          label="确认"
          :loading="submitting"
          @click="confirmRevoke" />
      </template>
    </UModal>
  </div>
</template>
