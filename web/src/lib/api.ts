/**
 * BFF 唯一出口(RULE-004):所有后端调用必须经本模块,禁止组件直接 fetch 后端。
 * 服务端(Server Component / Route Handler)通过 API_BASE_URL 访问后端,
 * 该环境变量仅服务端可见,禁止 NEXT_PUBLIC_ 前缀暴露到浏览器。
 */
import "server-only";
import type { components } from "./api-types";

/** 后端 HealthData(RULE-001:仅 status/db/version 三字段),来自生成类型。 */
export type HealthData = components["schemas"]["HealthData"];

/** 错误码 wire 枚举(SCREAMING_SNAKE),来自生成类型。 */
export type ErrorCode = components["schemas"]["ErrorCode"];

/**
 * 统一信封助手(RULE-003,泛型)。openapi 生成的是具体信封(Envelope_HealthData),
 * 这里基于生成类型派生泛型版供各接口复用;不在生成物内手改(RULE-005)。
 */
export interface ApiEnvelope<T = unknown> {
  success: boolean;
  data?: T;
  errorCode?: ErrorCode | null;
  message?: string | null;
}

/** 后端基础地址(仅服务端)。默认指向本地 dev 后端。 */
const API_BASE_URL = process.env.API_BASE_URL ?? "http://127.0.0.1:8080";

/** 后端不可达(连接失败/超时/非 JSON)时抛出——前端映射 ERR-001。 */
export class BackendUnreachableError extends Error {
  constructor(cause: unknown) {
    super("backend unreachable", { cause });
    this.name = "BackendUnreachableError";
  }
}

/** 统一信封解包:成功返回 data;业务失败返回 null + 错误信息(RULE-003 信封)。 */
export type UnwrapResult<T> =
  | { ok: true; data: T }
  | { ok: false; code: string; message: string }
  | { ok: false; unreachable: true };

/**
 * GET 请求后端并解统一信封。
 * - 网络层失败(后端未启动/DNS/超时)→ BackendUnreachableError(非 5xx)
 * - HTTP 层失败 → 按信封 errorCode 透传
 */
export async function apiGet<T>(
  path: string,
): Promise<{ status: number; envelope: ApiEnvelope<T> }> {
  let res: Response;
  try {
    res = await fetch(`${API_BASE_URL}${path}`, {
      cache: "no-store", // health 类探测必须实时
      signal: AbortSignal.timeout(5000),
    });
  } catch (e) {
    throw new BackendUnreachableError(e);
  }

  let json: ApiEnvelope<T>;
  try {
    json = (await res.json()) as ApiEnvelope<T>;
  } catch (e) {
    throw new BackendUnreachableError(e);
  }
  return { status: res.status, envelope: json };
}
