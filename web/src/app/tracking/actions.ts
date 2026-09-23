"use server";

/**
 * P05 追踪页的 Server Action(RULE-021/024/029 的前端入口)。
 *
 * 与问卷同一纪律:会话令牌是 HttpOnly Cookie,只有服务端读得到,
 * 每次提交都从请求里取出并转发给后端;校验的**权威**在后端,
 * 这一层只负责把信封错误翻译成面向用户的文案。
 */
import { cookies } from "next/headers";

import { trackSnapshotSkip } from "@/lib/analytics";
import { apiDelete, apiPut, BackendUnreachableError } from "@/lib/api";
import { SESSION_COOKIE } from "@/lib/session";

import { SNAPSHOT_EXPIRED_ERROR, SNAPSHOT_NETWORK_ERROR } from "./state";

/** 余额以**元字符串**过线(用户输入形态):「3200.50」;分的换算在后端 DTO(禁 float)。 */
export type BalanceInputs = Record<string, string>;

export type SubmitResult = { ok: true } | { ok: false; error: string };
export type SimpleResult = { ok: true } | { ok: false; error: string };

/**
 * 录入 / 覆盖某月快照(RULE-021:全桶必填由后端按桶集校验;覆盖即同月更新)。
 * 是否属于覆盖由调用方(录入卡)的 `recorded` 状态掌握 —— 提交前的覆盖确认在那里。
 */
export async function submitSnapshotAction(
  month: string,
  balances: BalanceInputs,
  specialMonth: boolean,
): Promise<SubmitResult> {
  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value;
  if (!token) return { ok: false, error: SNAPSHOT_EXPIRED_ERROR };

  try {
    const { status, envelope } = await apiPut<{ snapshot: unknown }>(
      `/api/v1/snapshots/${month}`,
      { balances, special_month: specialMonth },
      `${SESSION_COOKIE}=${token}`,
    );
    if (status === 200 && envelope.success) {
      // 页面数据经 router.refresh() 重取 summary;快照与偏离结论由服务端算好回填
      return { ok: true };
    }
    if (status === 401) return { ok: false, error: SNAPSHOT_EXPIRED_ERROR };
    return { ok: false, error: envelope.message ?? "提交失败,请稍后重试" };
  } catch (e) {
    if (e instanceof BackendUnreachableError) {
      return { ok: false, error: SNAPSHOT_NETWORK_ERROR };
    }
    throw e;
  }
}

/** 删除快照(RULE-029:仅最新月;后端判边界,历史月返回 422 文案)。 */
export async function deleteSnapshotAction(month: string): Promise<SimpleResult> {
  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value;
  if (!token) return { ok: false, error: SNAPSHOT_EXPIRED_ERROR };

  try {
    const { status, envelope } = await apiDelete<{ deleted: boolean }>(
      `/api/v1/snapshots/${month}`,
      `${SESSION_COOKIE}=${token}`,
    );
    if (status === 200 && envelope.success) return { ok: true };
    if (status === 401) return { ok: false, error: SNAPSHOT_EXPIRED_ERROR };
    return { ok: false, error: envelope.message ?? "删除失败,请稍后重试" };
  } catch (e) {
    if (e instanceof BackendUnreachableError) {
      return { ok: false, error: SNAPSHOT_NETWORK_ERROR };
    }
    throw e;
  }
}

/** 跳过本月(RULE-023:不落任何数据;上报是唯一观测口,失败静默)。 */
export async function skipMonthAction(): Promise<void> {
  await trackSnapshotSkip();
}
