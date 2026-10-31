import { invoke } from "@tauri-apps/api/core";
import {
  type AppConfig,
  type AppCookie,
  type Area,
  type ParentArea,
  type Stream,
  DEFAULT_CONFIG,
} from "@/types/config";

const MAX_STRING_LENGTH = 2_048;
const MAX_TOKEN_LENGTH = 8_192;
const MAX_COOKIES = 512;
const MAX_STREAMS = 64;
const MAX_PARENT_AREAS = 200;
const MAX_CHILD_AREAS = 200;

const COOKIE_NAME_PATTERN = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/;
const CONTROL_CHARACTER_PATTERN = /[\r\n\0]/;

type SecretFields = Pick<
  AppConfig,
  "cookies" | "csrf" | "roomToken" | "streams"
>;
type PublicConfig = Omit<AppConfig, keyof SecretFields>;

const SECRET_KEYS = ["cookies", "csrf", "roomToken", "streams"] as const;
const EMPTY_SECRETS_JSON = JSON.stringify({
  cookies: [],
  csrf: null,
  roomToken: "",
  streams: [],
});

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function readString(
  value: unknown,
  fallback = "",
  maxLength = MAX_STRING_LENGTH,
): string {
  return typeof value === "string" ? value.slice(0, maxLength) : fallback;
}

function readNullableString(
  value: unknown,
  maxLength = MAX_STRING_LENGTH,
): string | null {
  return typeof value === "string" ? value.slice(0, maxLength) : null;
}

function readNumber(
  value: unknown,
  fallback: number,
  { min = 0, max = Number.MAX_SAFE_INTEGER } = {},
): number {
  return typeof value === "number" &&
    Number.isFinite(value) &&
    value >= min &&
    value <= max
    ? value
    : fallback;
}

function readBoolean(value: unknown, fallback = false): boolean {
  return typeof value === "boolean" ? value : fallback;
}

function parseJsonObject(value: string | null): Record<string, unknown> | null {
  if (!value) return null;

  try {
    const parsed: unknown = JSON.parse(value);
    return isRecord(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

export function sanitizeCookies(value: unknown): AppCookie[] {
  if (!Array.isArray(value)) return [];

  const cookies: AppCookie[] = [];
  const seen = new Set<string>();

  for (const item of value.slice(0, MAX_COOKIES)) {
    if (!isRecord(item)) continue;

    const name = readString(item.name, "", 256).trim();
    const cookieValue = readString(item.value, "", MAX_TOKEN_LENGTH);
    if (
      !name ||
      !cookieValue ||
      seen.has(name) ||
      !COOKIE_NAME_PATTERN.test(name) ||
      CONTROL_CHARACTER_PATTERN.test(cookieValue)
    ) {
      continue;
    }

    seen.add(name);
    cookies.push({ name, value: cookieValue });
  }

  return cookies;
}

function sanitizeStreams(value: unknown): Stream[] {
  if (!Array.isArray(value)) return [];

  const streams: Stream[] = [];
  for (const item of value.slice(0, MAX_STREAMS)) {
    if (!isRecord(item)) continue;

    const type = readString(item.type, "", 64);
    const address = readString(item.address, "", MAX_STRING_LENGTH);
    const key = readString(item.key, "", MAX_TOKEN_LENGTH);
    if (!type || !address || !key) continue;

    streams.push({ type, address, key });
  }

  return streams;
}

function sanitizeArea(value: unknown): Area | null {
  if (!isRecord(value)) return null;

  const id = readString(value.id, "", 64);
  const name = readString(value.name, "", 512);
  if (!id || !name) return null;

  const stringFields = [
    "parent_id",
    "old_area_id",
    "act_id",
    "pk_status",
    "hot_status",
    "lock_status",
    "complex_area_name",
    "pinyin",
    "parent_name",
    "area_type",
  ] as const;

  const area: Area = {
    id,
    name,
    parent_id: "",
    old_area_id: "",
    act_id: "",
    pk_status: "",
    hot_status: "",
    lock_status: "",
    pic: readString(value.pic, "", MAX_STRING_LENGTH),
    complex_area_name: "",
    pinyin: "",
    parent_name: "",
    area_type: "",
  };

  for (const field of stringFields) {
    area[field] = readString(value[field], "", 256);
  }

  return area;
}

function sanitizeAreaList(value: unknown): ParentArea[] {
  if (!Array.isArray(value)) return [];

  const parents: ParentArea[] = [];
  for (const item of value.slice(0, MAX_PARENT_AREAS)) {
    if (!isRecord(item)) continue;

    const id = readString(item.id, "", 64);
    const name = readString(item.name, "", 512);
    if (!id || !name) continue;

    const children = Array.isArray(item.list)
      ? item.list
          .slice(0, MAX_CHILD_AREAS)
          .map(sanitizeArea)
          .filter((area): area is Area => area !== null)
      : [];

    parents.push({ id, name, list: children });
  }

  return parents;
}

function sanitizePublicConfig(value: unknown): PublicConfig {
  const source = isRecord(value) ? value : {};

  return {
    areaList: sanitizeAreaList(source.areaList),
    theme: source.theme === "dark" ? "dark" : "light",
    uid: Math.trunc(readNumber(source.uid, DEFAULT_CONFIG.uid)),
    avatar: readNullableString(source.avatar),
    username: readNullableString(source.username),
    roomId: Math.trunc(readNumber(source.roomId, DEFAULT_CONFIG.roomId)),
    roomTitle: readString(source.roomTitle),
    categoryId: readString(source.categoryId, "", 64),
    areaId: readString(source.areaId, "", 64),
    img_url: readString(source.img_url, "", 512),
    sub_url: readString(source.sub_url, "", 512),
    isOpenLive: readBoolean(source.isOpenLive),
  };
}

function sanitizeSecretFields(value: unknown): SecretFields {
  const source = isRecord(value) ? value : {};

  return {
    cookies: sanitizeCookies(source.cookies),
    csrf: readNullableString(source.csrf, 512),
    roomToken: readString(source.roomToken, "", MAX_TOKEN_LENGTH),
    streams: sanitizeStreams(source.streams),
  };
}

function splitConfig(config: AppConfig): {
  publicConfig: PublicConfig;
  secrets: SecretFields;
} {
  return {
    publicConfig: {
      areaList: config.areaList,
      theme: config.theme,
      uid: config.uid,
      avatar: config.avatar,
      username: config.username,
      roomId: config.roomId,
      roomTitle: config.roomTitle,
      categoryId: config.categoryId,
      areaId: config.areaId,
      img_url: config.img_url,
      sub_url: config.sub_url,
      isOpenLive: config.isOpenLive,
    },
    secrets: {
      cookies: config.cookies,
      csrf: config.csrf,
      roomToken: config.roomToken,
      streams: config.streams,
    },
  };
}

function pickSecretFields(
  source: Record<string, unknown> | null,
): Record<string, unknown> {
  if (!source) return {};

  return Object.fromEntries(
    SECRET_KEYS.filter((key) => key in source).map((key) => [key, source[key]]),
  );
}

export class ConfigManager {
  private static lastPublicJson: string | null = null;
  private static lastSecretsJson: string | null = null;
  private static writeQueue: Promise<void> = Promise.resolve();

  private static enqueue<T>(task: () => Promise<T>): Promise<T> {
    const result = this.writeQueue.then(task, task);
    this.writeQueue = result.then(
      () => undefined,
      () => undefined,
    );
    return result;
  }

  public static async load(): Promise<AppConfig> {
    // A queued write may still be in flight; read after it finishes so this
    // window cannot resurrect credentials that were just cleared.
    await this.writeQueue;

    let publicSource: Record<string, unknown> | null = null;
    let secretSource: Record<string, unknown> | null = null;

    const [publicResult, secretResult] = await Promise.allSettled([
      invoke<string | null>("load_app_config"),
      invoke<string | null>("load_app_secrets"),
    ]);

    if (publicResult.status === "fulfilled") {
      publicSource = parseJsonObject(publicResult.value);
      if (publicResult.value && !publicSource) {
        console.warn("Ignoring malformed app config");
      }
    } else {
      console.error("Failed to load app config:", publicResult.reason);
    }

    if (secretResult.status === "fulfilled") {
      secretSource = parseJsonObject(secretResult.value);
      if (secretResult.value && !secretSource) {
        console.warn("Ignoring malformed app secrets");
      }
    } else {
      console.error("Failed to load app secrets:", secretResult.reason);
    }

    // Older versions kept cookies, CSRF tokens and stream keys in the public
    // config file. Migrate them into the secret store and rewrite the public
    // file without any secret fields.
    const legacySecrets = pickSecretFields(publicSource);
    const storedSecrets = pickSecretFields(secretSource);
    const secrets = sanitizeSecretFields({
      ...legacySecrets,
      ...storedSecrets,
    });
    const publicConfig = sanitizePublicConfig(publicSource);

    const needsMigration = Object.keys(legacySecrets).length > 0;

    if (needsMigration) {
      try {
        await this.save({ ...publicConfig, ...secrets });
      } catch (error) {
        console.error("Failed to migrate app config:", error);
      }
    } else {
      this.lastPublicJson = JSON.stringify(publicConfig, null, 2);
      this.lastSecretsJson = JSON.stringify(secrets);
    }

    return { ...publicConfig, ...secrets };
  }

  public static save(config: AppConfig): Promise<void> {
    return this.enqueue(() => this.saveNow(config));
  }

  private static async saveNow(config: AppConfig): Promise<void> {
    const { publicConfig, secrets } = splitConfig(config);
    const publicJson = JSON.stringify(publicConfig, null, 2);
    const secretsJson = JSON.stringify(secrets);

    // Persist secrets before the public file. If the credential store fails,
    // the previous public file is still intact and no saved secret is lost.
    if (secretsJson !== this.lastSecretsJson) {
      await invoke("save_app_secrets", { contents: secretsJson });
      this.lastSecretsJson = secretsJson;
    }

    if (publicJson !== this.lastPublicJson) {
      await invoke("save_app_config", { contents: publicJson });
      this.lastPublicJson = publicJson;
    }
  }

  public static clearSecrets(): Promise<void> {
    return this.enqueue(async () => {
      await invoke("clear_app_secrets");
      this.lastSecretsJson = EMPTY_SECRETS_JSON;
    });
  }
}
