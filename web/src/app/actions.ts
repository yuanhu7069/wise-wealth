"use server";

/**
 * P01 的 Server Action:页面 Server 组件与 Client 卡片之间的取数桥梁。
 * 服务端 fetch 经 lib/api.ts(BFF 唯一出口,RULE-004)。
 */
import { apiGet, type HealthData } from "@/lib/api";

export interface HealthActionResult {
  ok: boolean;
  /** 成功时 code=OK;失败时:ERR-001 / ERR-002 / 其他 errorCode */
  code: string;
  version?: string;
}

export async function fetchHealthAction(): Promise<HealthActionResult> {
  try {
    // 泛型来自 openapi 生成物(RULE-005):后端字段变更会在此处编译期暴露
    const { envelope } = await apiGet<HealthData>("/api/v1/health");
    if (envelope.success && envelope.data) {
      if (envelope.data.db === "ok") {
        return { ok: true, code: "OK", version: envelope.data.version };
      }
      // ADR-A-003:进程存活但库不可达 → ERR-002
      return { ok: false, code: "ERR_002" };
    }
    // 信封层失败(理论上 health 恒 200,此处防御)
    return { ok: false, code: envelope.errorCode ?? "INTERNAL_ERROR" };
  } catch {
    // fetch 层失败(后端未启动/超时/非 JSON)→ ERR-001
    return { ok: false, code: "ERR_001" };
  }
}
