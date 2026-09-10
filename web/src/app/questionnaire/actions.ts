"use server";

/**
 * 问卷向导的 Server Action(RULE-003/004/005)。
 *
 * 浏览器 → 本层 → 后端:会话令牌是 HttpOnly Cookie,只有服务端读得到,
 * 因此每次保存都从请求里取出并转发给后端。
 */
import { cookies } from "next/headers";

import { apiGet, apiPut, BackendUnreachableError } from "@/lib/api";
import { SESSION_COOKIE } from "@/lib/session";

import type { Answers, ModesData, SaveResult } from "./state";

interface ProfileView {
  draft_step: number;
  questionnaire_completed: boolean;
}

export async function saveStepAction(step: number, answers: Answers): Promise<SaveResult> {
  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value;
  if (!token) {
    return { ok: false, error: "登录已过期,请重新登录" };
  }

  const body = { step, ...answers };
  try {
    const { status, envelope } = await apiPut<ProfileView>(
      "/api/v1/profiles/me/step",
      body,
      `${SESSION_COOKIE}=${token}`,
    );

    if (status === 200 && envelope.success && envelope.data) {
      return { ok: true, draftStep: envelope.data.draft_step };
    }
    if (status === 401) {
      return { ok: false, error: "登录已过期,请重新登录" };
    }
    // 后端给的是面向用户的校验文案(如「金额需大于 0」),直接透出
    return { ok: false, error: envelope.message ?? "保存失败,请稍后重试" };
  } catch (e) {
    if (e instanceof BackendUnreachableError) {
      return { ok: false, error: "服务暂时不可用,请确认后端已启动后重试" };
    }
    throw e;
  }
}

/** 步 6 的模式列表与推荐。按需拉取:可能是用户本次刚答完问卷,页面初次渲染时还没有档案。 */
export async function loadModesAction(): Promise<
  { ok: true; data: ModesData } | { ok: false; error: string }
> {
  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value;
  if (!token) {
    return { ok: false, error: "登录已过期,请重新登录" };
  }

  try {
    const { status, envelope } = await apiGet<ModesData>("/api/v1/modes", {
      cookie: `${SESSION_COOKIE}=${token}`,
    });
    if (status === 200 && envelope.success && envelope.data) {
      return { ok: true, data: envelope.data };
    }
    if (status === 401) {
      return { ok: false, error: "登录已过期,请重新登录" };
    }
    return { ok: false, error: envelope.message ?? "推荐加载失败,请稍后重试" };
  } catch (e) {
    if (e instanceof BackendUnreachableError) {
      return { ok: false, error: "服务暂时不可用,请确认后端已启动后重试" };
    }
    throw e;
  }
}
