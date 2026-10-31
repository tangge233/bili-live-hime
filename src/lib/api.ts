import { useConfigStore } from "@/stores/config";
import type { AppCookie } from "@/types/config";
import { fetch } from "@tauri-apps/plugin-http";
import { appSign } from "@/lib/app-sign";

const USER_AGENT =
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0";

const COMMON_HEADERS = {
  accept: "*/*",
  "user-agent": USER_AGENT,
  "sec-ch-ua": `"Microsoft Edge";v="143", "Chromium";v="143", "Not A(Brand";v="24"`,
  "sec-ch-ua-mobile": "?0",
  "sec-ch-ua-platform": `"Windows"`,
  "sec-fetch-dest": "empty",
  "sec-fetch-mode": "cors",
  "sec-fetch-site": "same-site",
};

const COOKIE_SPLIT_PATTERN = /,(?=\s*[!#$%&'*+\-.^_`|~0-9A-Za-z]+=)/;

/**
 * Parses one or more `Set-Cookie` header values. A naive `split(",")` breaks
 * on the comma inside `Expires=Wed, 21 Oct ...`, so only split where the next
 * segment starts a new `name=value` pair.
 */
function parseSetCookies(headers: string[]): AppCookie[] {
  const cookies: AppCookie[] = [];

  for (const header of headers) {
    for (const segment of header.split(COOKIE_SPLIT_PATTERN)) {
      const pair = segment.split(";", 1)[0]?.trim();
      if (!pair) continue;

      const separator = pair.indexOf("=");
      if (separator <= 0) continue;

      const name = pair.slice(0, separator).trim();
      const value = pair.slice(separator + 1).trim();
      if (name && value) {
        cookies.push({ name, value });
      }
    }
  }

  return cookies;
}

function getSetCookieHeaders(response: Response): string[] {
  const headers = response.headers as Headers & {
    getSetCookie?: () => string[];
  };

  if (typeof headers.getSetCookie === "function") {
    const values = headers.getSetCookie();
    if (values.length > 0) return values;
  }

  const combined = response.headers.get("set-cookie");
  return combined ? [combined] : [];
}

function applyCookies(cookies: AppCookie[]) {
  if (cookies.length === 0) return;

  // Only session-changing responses matter here. Merging every tracking
  // cookie would both bloat the secret store and risk overwriting unrelated
  // values.
  const hasSessionCookie = cookies.some(
    (cookie) => cookie.name === "SESSDATA" || cookie.name === "bili_jct",
  );
  if (!hasSessionCookie) return;

  const current = useConfigStore().config.cookies;
  const merged = new Map(current.map((cookie) => [cookie.name, cookie.value]));
  for (const cookie of cookies) {
    merged.set(cookie.name, cookie.value);
  }

  useConfigStore().setCookies(
    [...merged].map(([name, value]) => ({ name, value })),
  );
}

export async function request<T>(
  base_url: string,
  endpoint: string,
  options: RequestInit & {
    data?: Record<string, string>;
  } = {},
  requireSign: boolean = false,
  raw: boolean = false,
): Promise<T> {
  const state = useConfigStore();
  const url = `${base_url}${endpoint}`;
  const method = options.method?.toUpperCase() || "GET";

  const headers: HeadersInit = {
    ...COMMON_HEADERS,
    ...options.headers,
    cookie: state.getCookieString(),
  };

  const fetchOptions: RequestInit = {
    ...options,
    method,
    headers,
  };

  if (options.data && method !== "GET") {
    (headers as Record<string, string>)["Content-Type"] =
      "application/x-www-form-urlencoded; charset=UTF-8";
    if (requireSign) {
      fetchOptions.body = appSign(options.data);
    } else {
      const urlSP = new URLSearchParams(options.data);
      fetchOptions.body = urlSP.toString();
    }
  }

  const response = await fetch(url, fetchOptions);
  const contentType = response.headers.get("content-type") ?? "";
  const body = await response.text();

  if (!contentType.toLowerCase().includes("json")) {
    throw new Error(
      `API returned an unexpected response (HTTP ${response.status})`,
    );
  }

  let data: unknown;
  try {
    data = body ? JSON.parse(body) : null;
  } catch {
    throw new Error(`API returned malformed JSON (HTTP ${response.status})`);
  }

  if (!response.ok) {
    const message =
      typeof data === "object" &&
      data !== null &&
      "message" in data &&
      typeof data.message === "string"
        ? data.message
        : response.statusText || "Unknown Error";
    throw new Error(`HTTP ${response.status}: ${message}`);
  }

  if (!raw) {
    if (
      typeof data !== "object" ||
      data === null ||
      Array.isArray(data) ||
      !("code" in data)
    ) {
      throw new Error("API returned an unexpected response body");
    }

    const result = data as { code: unknown; message?: unknown; data?: unknown };
    if (
      result.code !== 0 ||
      result.data === null ||
      result.data === undefined
    ) {
      const message =
        typeof result.message === "string" ? result.message : "Unknown Error";
      throw new Error(`API Request Error: ${message} (${String(result.code)})`);
    }

    // Only trust cookies from a fully successful API response. A failed
    // login/poll must not be able to replace the stored session.
    applyCookies(parseSetCookies(getSetCookieHeaders(response)));
    return result.data as T;
  }

  // Raw responses have no shared envelope, so HTTP success is the only
  // success signal available.
  applyCookies(parseSetCookies(getSetCookieHeaders(response)));
  return data as T;
}
