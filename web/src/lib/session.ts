/**
 * 会话读取与保护(RULE-001)。**仅服务端**:令牌是 HttpOnly Cookie,
 * 浏览器端脚本拿不到也不该拿到(基线 §5.3)。
 */
import "server-only";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { apiGet } from "./api";

/** 会话 Cookie 名(与后端 `api::middleware::SESSION_COOKIE` 保持一致) */
export const SESSION_COOKIE = "ww_session";

export interface Session {
  username: string;
}

/**
 * 读取当前会话;无会话或后端不可达时返回 null。
 *
 * 不抛错是刻意的:页面渲染时的「未登录」与「后端挂了」都该走登录引导,
 * 而不是把一个 500 抛给用户 —— 后者会在页面上留下整页错误态。
 */
export async function getSession(): Promise<Session | null> {
  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value;
  if (!token) return null;

  try {
    const { status, envelope } = await apiGet<Session>("/api/v1/auth/me", {
      cookie: `${SESSION_COOKIE}=${token}`,
    });
    if (status === 200 && envelope.success && envelope.data) {
      return envelope.data;
    }
    return null;
  } catch {
    return null;
  }
}

/**
 * 受保护页面的入口:无会话则跳登录,并把原路径交给 from 参数(登录后回跳)。
 */
export async function requireSession(from: string): Promise<Session> {
  const session = await getSession();
  if (!session) {
    // 区分「从没登录」与「登录过期了」:只在前者之外附 expired 标记。
    // 判据是有没有会话 Cookie —— 有 Cookie 却被后端拒绝,才叫过期(ERR-006)。
    const store = await cookies();
    const hadCookie = store.has(SESSION_COOKIE);
    const target = safeFrom(from);
    redirect(`/login?from=${encodeURIComponent(target)}${hadCookie ? "&expired=1" : ""}`);
  }
  return session;
}

/**
 * 回跳目标白名单:只接受以单个 `/` 开头的站内路径(RULE-001 / AC-4)。
 *
 * `//evil.com` 与 `https://evil.com` 都会被拒绝 —— 前者在浏览器里是同协议外域,
 * 正是开放式重定向最常见的绕过写法。
 */
export function safeFrom(raw: string | undefined | null): string {
  if (!raw) return "/";
  if (!raw.startsWith("/") || raw.startsWith("//")) return "/";
  return raw;
}
