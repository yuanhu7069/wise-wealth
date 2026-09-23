/**
 * CSV 下载代理(E 期,票 04;ADR-E-004 / 红线 16)。
 *
 * 后端导出端点直出 `text/csv` 附件,浏览器要靠「同源请求自带会话 Cookie」下载;
 * 而前端禁止裸调后端地址(红线 16),故由这个 Route Handler 代理:
 * 浏览器 → /api/export/{kind}(同源,Cookie 自动携带)→ 这里读 Cookie 转发后端 → 流式透传。
 * 响应头按后端给的透传(content-type / content-disposition,文件名含导出日期)。
 */
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { apiProxy } from "@/lib/api";
import { SESSION_COOKIE } from "@/lib/session";

/** kind → 后端路径;白名单之外一律 404(不当通用代理用)。 */
const EXPORT_PATHS: Record<string, string> = {
  snapshots: "/api/v1/snapshots/export",
  plan: "/api/v1/plans/active/export",
};

export async function GET(_req: Request, ctx: { params: Promise<{ kind: string }> }) {
  const { kind } = await ctx.params;
  const path = EXPORT_PATHS[kind];
  if (!path) {
    return new Response("not found", { status: 404 });
  }

  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value;
  if (!token) {
    // 未登录的下载请求:回登录页(下载链接不做信封提示)
    redirect(`/login?from=/tracking`);
  }

  let upstream: Response;
  try {
    upstream = await apiProxy(path, `${SESSION_COOKIE}=${token}`);
  } catch {
    return new Response("导出服务暂时不可用,请稍后重试", { status: 502 });
  }

  if (upstream.status === 401) {
    redirect(`/login?from=/tracking&expired=1`);
  }
  if (!upstream.ok) {
    return new Response("导出失败,请稍后重试", { status: 502 });
  }

  const headers = new Headers();
  const contentType = upstream.headers.get("content-type");
  if (contentType) headers.set("content-type", contentType);
  const disposition = upstream.headers.get("content-disposition");
  if (disposition) headers.set("content-disposition", disposition);
  return new Response(upstream.body, { status: 200, headers });
}
