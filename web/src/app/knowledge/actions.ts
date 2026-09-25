"use server";

/**
 * 文章全文取数(Server Action,G 期票 04)。
 *
 * 卡片展开时经此取全文:R-004 BFF 唯一出口 —— 令牌是 HttpOnly Cookie,
 * 只有服务端能把会话转发给后端。未知 id 返回结构化 notFound(AC-12),
 * 由调用方渲染 404 文案而非整页错误边界。
 */
import { cookies } from "next/headers";

import { apiGet } from "@/lib/api";
import { SESSION_COOKIE } from "@/lib/session";

import type { KnowledgeArticle } from "./state";

export type ArticleResult =
  | { ok: true; article: KnowledgeArticle }
  | { ok: false; notFound?: boolean; error: string };

export async function getKnowledgeArticleAction(id: string): Promise<ArticleResult> {
  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value;
  if (!token) {
    return { ok: false, error: "登录已过期,请重新登录" };
  }

  try {
    const { status, envelope } = await apiGet<KnowledgeArticle>(
      `/api/v1/knowledge/${encodeURIComponent(id)}`,
      { cookie: `${SESSION_COOKIE}=${token}` },
    );
    if (status === 200 && envelope.success && envelope.data) {
      return { ok: true, article: envelope.data };
    }
    if (status === 404) {
      return { ok: false, notFound: true, error: "文章不存在" };
    }
    return { ok: false, error: envelope.message ?? "暂时读不出来,再试一次" };
  } catch {
    return { ok: false, error: "后端服务未响应,请稍后再试" };
  }
}
