"use server";

/**
 * P04 方案生成入口(Server Action)。
 *
 * 为什么放在 plan 而不是 questionnaire:方案是这张页面的产物,生成的失败态也归它解释;
 * P03 步 6 只是触发点。动作与归属一致,重新生成(工单 06)回来复用同一个函数。
 *
 * 返回结构化结果而不是抛错:422(问卷未完成/模式不存在)是**用户能自己修**的情况,
 * 要在步 6 原地给出文案与重试入口,不能变成一整页错误边界。
 */
import { cookies } from "next/headers";

import { apiPost, BackendUnreachableError } from "@/lib/api";
import { errorCopyOf } from "@/lib/errors";
import { SESSION_COOKIE } from "@/lib/session";

import type { PlanView } from "./state";

export type GeneratePlanResult =
  | { ok: true; plan: PlanView }
  | {
      ok: false;
      /** 面向用户的文案(三要素:发生了什么 + 为什么 + 怎么办) */
      error: string;
      /** 是否值得原地重试:401 重试没有意义(要先登录),5xx 与网络失败则可以 */
      retryable: boolean;
      /** true 表示会话已失效,页面应引导重新登录 */
      expired?: boolean;
    };

/** 生成入口(F 期 prd-f §9.5):区分问卷路径与模式库手动路径,仅进埋点口径。 */
export type PlanEntry = "questionnaire" | "mode_lib";

export async function generatePlanAction(
  l1Mode: string,
  entry: PlanEntry = "questionnaire",
): Promise<GeneratePlanResult> {
  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value;
  if (!token) {
    return { ok: false, error: "登录已过期,请重新登录", retryable: false, expired: true };
  }

  try {
    const { status, envelope } = await apiPost<PlanView>(
      "/api/v1/plans",
      { l1_mode: l1Mode, entry },
      `${SESSION_COOKIE}=${token}`,
    );

    if (status === 200 && envelope.success && envelope.data) {
      return { ok: true, plan: envelope.data };
    }
    if (status === 401) {
      return { ok: false, error: "登录已过期,请重新登录", retryable: false, expired: true };
    }
    // 422 的 message 由后端给出且面向用户(「请先完成问卷」等),可直接展示
    const copy = errorCopyOf(envelope.errorCode);
    return {
      ok: false,
      error: envelope.message ?? `${copy.title}。${copy.description}`,
      retryable: status >= 500,
    };
  } catch (e) {
    if (e instanceof BackendUnreachableError) {
      const copy = errorCopyOf("ERR_001");
      return { ok: false, error: `${copy.title}。${copy.description}`, retryable: true };
    }
    throw e;
  }
}
