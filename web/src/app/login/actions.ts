"use server";

/**
 * 登录相关的 Server Action(RULE-001 / RULE-002)。
 *
 * 浏览器 → Next(本层)→ 后端:后端下发的 HttpOnly Cookie 由这里转发给浏览器,
 * 前端代码任何地方都不接触令牌原文(基线 §5.3 禁止 localStorage 存凭证)。
 */
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { apiPost } from "@/lib/api";
import { SESSION_COOKIE, safeFrom } from "@/lib/session";

import type { LoginState } from "./state";

/** 从 Set-Cookie 头里取出会话值。后端格式固定,取不到即视为登录失败。 */
function sessionValue(setCookie: string | null): string | null {
  if (!setCookie) return null;
  const pair = setCookie.split(";")[0] ?? "";
  const [name, value] = pair.split("=");
  if (name?.trim() !== SESSION_COOKIE || !value) return null;
  return value.trim();
}

export async function loginAction(_prev: LoginState, formData: FormData): Promise<LoginState> {
  const username = String(formData.get("username") ?? "").trim();
  const password = String(formData.get("password") ?? "");
  const from = safeFrom(String(formData.get("from") ?? "/"));

  if (!username || !password) {
    return { error: "请输入用户名和密码" };
  }

  let result: Awaited<ReturnType<typeof apiPost>>;
  try {
    result = await apiPost("/api/v1/auth/login", { username, password });
  } catch {
    // 后端不可达:按 ERR-001 的口气说明白,别让用户以为是口令错了
    return { error: "服务暂时不可用,请确认后端已启动后重试" };
  }

  const token = sessionValue(result.setCookie);
  if (result.status === 200 && token) {
    const store = await cookies();
    store.set(SESSION_COOKIE, token, {
      httpOnly: true,
      sameSite: "lax",
      path: "/",
      maxAge: 60 * 60 * 24 * 30,
      secure: process.env.NODE_ENV === "production",
    });
    redirect(from);
  }

  // RULE-002:限流与凭证错误要分清 —— 前者等一会儿就好,后者要重新输
  if (result.status === 429) {
    return { error: "操作太频繁了,稍后再试" };
  }
  return { error: "用户名或密码不正确" };
}

/** 退出登录:清 Cookie 并回到登录页。 */
export async function logoutAction(): Promise<void> {
  const store = await cookies();
  store.delete(SESSION_COOKIE);
  redirect("/login");
}
