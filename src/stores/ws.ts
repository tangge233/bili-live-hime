import { defineStore } from "pinia";
import { ref } from "vue";
import { type Comment } from "@/types/comment";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { InteractWordV2 } from "@/protos/InteractWordV2";
import { useToast } from "@nuxt/ui/composables";

interface MessageEvent {
  payload: string;
}

interface ConnectionEvent {
  status: string;
}

interface ErrorEvent {
  error: string;
}

const MAX_REGULAR_MESSAGES = 500;
const MAX_SUPER_CHATS = 200;
const MAX_PROTOBUF_BASE64_LENGTH = 1_000_000;

let nextMessageId = 0;

function createMessageId(): string {
  nextMessageId += 1;
  return `${Date.now().toString(36)}-${nextMessageId.toString(36)}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function readString(value: unknown, fallback = ""): string {
  return typeof value === "string" ? value : fallback;
}

function readNumber(value: unknown, fallback = 0): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function appendBounded<T>(items: T[], item: T, limit: number): T[] {
  if (items.length >= limit) {
    return [...items.slice(items.length - limit + 1), item];
  }
  return [...items, item];
}

function base64ToBytes(base64: string) {
  const binString = atob(base64);
  return Uint8Array.from(binString, (m) => m.codePointAt(0) || 0);
}

export const useWsStore = defineStore("ws", () => {
  const connected = ref(false);
  const connecting = ref(false);
  const regularMessages = ref<Comment[]>([]);
  const superChats = ref<Comment[]>([]);
  const watchedUser = ref(0);
  const popularity = ref(0);
  const onlineCount = ref(0);

  const toast = useToast();

  function setConnected(value: boolean) {
    connected.value = value;
  }

  function setWatchedUser(value: number) {
    watchedUser.value = value;
  }

  function setPopularity(value: number) {
    popularity.value = value;
  }

  function setOnlineCount(value: number) {
    onlineCount.value = value;
  }

  function pushRegular(comment: Comment) {
    regularMessages.value = appendBounded(
      regularMessages.value,
      comment,
      MAX_REGULAR_MESSAGES,
    );
  }

  function addMessage(message: Comment) {
    if (message.type === "superchat") {
      superChats.value = appendBounded(
        superChats.value,
        message,
        MAX_SUPER_CHATS,
      );
    } else {
      pushRegular(message);
    }
  }

  function reset() {
    connected.value = false;
    connecting.value = false;
    regularMessages.value = [];
    superChats.value = [];
    watchedUser.value = 0;
    popularity.value = 0;
    onlineCount.value = 0;
  }

  async function initListeners(): Promise<() => void> {
    try {
      const isConnected = await invoke<boolean>("comment_is_connected");
      connected.value = isConnected;
      connecting.value = false;
    } catch (error) {
      console.error("Failed to query comment connection status:", error);
      connected.value = false;
      connecting.value = false;
    }

    const unlistenSuccess = await listen<ConnectionEvent>(
      "connection-success",
      (event) => {
        console.log("Connected:", event.payload);
        connected.value = true;
        connecting.value = false;
      },
    );

    const unlistenClosed = await listen<ConnectionEvent>(
      "connection-closed",
      (event) => {
        console.log("Disconnected:", event.payload);
        connected.value = false;
        connecting.value = false;
      },
    );

    const unlistenMessage = await listen<MessageEvent>(
      "message-received",
      (event) => {
        const { payload } = event.payload;

        // The danmaku stream is remote (and possibly hostile) data. A frame
        // missing an expected field must only drop that frame, never throw
        // out of this listener.
        let parsed: unknown;
        try {
          parsed = JSON.parse(payload);
        } catch {
          console.warn("Received malformed danmaku payload");
          return;
        }

        const obj = isRecord(parsed) ? parsed : null;
        const cmd = obj?.cmd;
        if (!obj || typeof cmd !== "string") return;

        switch (cmd) {
          case "DANMU_MSG": {
            const info = obj.info;
            if (!Array.isArray(info)) {
              console.warn("DANMU_MSG missing info");
              break;
            }

            const infoHead = isRecord(info[0]) ? info[0] : null;
            const extra = infoHead ? infoHead[15] : null;
            const extraRecord = isRecord(extra) ? extra : null;
            const user = extraRecord ? extraRecord.user : null;
            const userRecord = isRecord(user) ? user : null;
            const base = userRecord ? userRecord.base : null;
            if (!isRecord(base)) {
              console.warn("DANMU_MSG missing user info");
              break;
            }

            pushRegular({
              id: createMessageId(),
              username: readString(base.name),
              message: readString(info[1]),
              timestamp: new Date(),
              type: "chat",
              avatar: readString(base.face),
            });
            break;
          }
          case "WATCHED_CHANGE": {
            const data = isRecord(obj.data) ? obj.data : null;
            watchedUser.value = readNumber(data?.num);
            break;
          }
          case "INTERACT_WORD_V2": {
            const data = isRecord(obj.data) ? obj.data : null;
            const pb = data?.pb;
            if (
              typeof pb !== "string" ||
              pb.length === 0 ||
              pb.length > MAX_PROTOBUF_BASE64_LENGTH
            ) {
              console.warn("INTERACT_WORD_V2 missing pb payload");
              break;
            }

            let username: string;
            try {
              username = InteractWordV2.fromBinary(base64ToBytes(pb)).uname;
            } catch {
              console.warn("INTERACT_WORD_V2 protobuf decode failed");
              break;
            }
            if (!username) break;

            pushRegular({
              id: createMessageId(),
              username,
              message: "进入直播间",
              timestamp: new Date(),
              type: "enter",
            });
            break;
          }
          case "ONLINE_RANK_COUNT": {
            const data = isRecord(obj.data) ? obj.data : null;
            onlineCount.value = readNumber(data?.count);
            break;
          }
          case "SEND_GIFT": {
            const data = isRecord(obj.data) ? obj.data : null;
            pushRegular({
              id: createMessageId(),
              username: readString(data?.uname),
              message: "",
              timestamp: new Date(),
              type: "gift",
              giftName: readString(data?.giftName),
              giftCount: readNumber(data?.num),
            });
            break;
          }
          case "SUPER_CHAT_MESSAGE": {
            const data = isRecord(obj.data) ? obj.data : null;
            const userInfo = isRecord(data?.user_info) ? data.user_info : null;
            if (!data || !userInfo) {
              console.warn("SUPER_CHAT_MESSAGE missing user_info");
              break;
            }

            superChats.value = appendBounded(
              superChats.value,
              {
                id: createMessageId(),
                username: readString(userInfo.uname),
                avatar: readString(userInfo.face),
                message: readString(data.message),
                timestamp: new Date(),
                amount: readNumber(data.price),
                type: "superchat",
              },
              MAX_SUPER_CHATS,
            );
            break;
          }
          default:
            // Unknown commands are expected; do not log the raw payload as it
            // may contain user content.
            break;
        }
      },
    );

    const unlistenError = await listen<ErrorEvent>(
      "connection-error",
      (event) => {
        console.error("Connection error:", event.payload.error);
        toast.add({
          title: "弹幕连接错误",
          description: event.payload.error,
          color: "error",
          icon: "i-lucide-circle-alert",
        });
        connecting.value = false;
      },
    );

    return () => {
      unlistenSuccess();
      unlistenClosed();
      unlistenMessage();
      unlistenError();
    };
  }

  async function connect(uid: number, room: number, token: string) {
    connecting.value = true;
    try {
      await invoke("comment_connect", {
        uid,
        room,
        token,
      });
    } catch (error) {
      const message = String(error);
      if (!message.includes("Already connected")) {
        toast.add({
          title: "连接弹幕失败",
          description: message,
          color: "error",
          icon: "i-lucide-circle-alert",
        });
      }
      try {
        const isConnected = await invoke<boolean>("comment_is_connected");
        connected.value = isConnected;
        connecting.value = false;
      } catch (statusError) {
        console.error(
          "Failed to query comment connection status:",
          statusError,
        );
        connected.value = false;
        connecting.value = false;
      }
    }
  }

  async function disconnect() {
    connecting.value = true;
    try {
      await invoke("comment_disconnect");
      connecting.value = false;
      connected.value = false;
    } catch (error) {
      toast.add({
        title: "断开弹幕失败",
        description: String(error),
        color: "error",
        icon: "i-lucide-circle-alert",
      });
      connecting.value = false;
    }
  }

  return {
    connected,
    connecting,
    regularMessages,
    superChats,
    watchedUser,
    popularity,
    onlineCount,
    initListeners,
    connect,
    disconnect,
    reset,
    setConnected,
    setWatchedUser,
    setPopularity,
    setOnlineCount,
    addMessage,
  };
});
