/**
 * P04 方案页(server 壳)。职责:要求会话 → 读当前方案 → 交给五段视图。
 *
 * 无方案时**防御性重定向首页**(design-v2 §3.2「P04 无数据兜底:不会出现(P01 已兜)」):
 * 空态属于 P01,方案页没有「空方案」这一形态,不给用户一页看不懂的空白。
 */
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { SiteFooterShell } from "@/components/site-footer-shell";
import { apiGet } from "@/lib/api";
import { requireSession, SESSION_COOKIE } from "@/lib/session";

import { PlanView } from "./plan-view";
import type { PlanView as Plan } from "./state";

export default async function PlanPage() {
  // RULE-001:方案页属受保护页面,未登录跳登录并在登录后回到本页
  await requireSession("/plan");

  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value ?? "";

  let plan: Plan | null = null;
  try {
    const { status, envelope } = await apiGet<Plan>("/api/v1/plans/active", {
      cookie: `${SESSION_COOKIE}=${token}`,
    });
    // 404 = 还没生成过方案;后端不可达与其它错误同样落到首页空态,不把 500 抛给用户
    if (status === 200 && envelope.success && envelope.data) plan = envelope.data;
  } catch {
    plan = null;
  }

  if (!plan) redirect("/");

  return (
    <>
      {/* 720 阅读宽度上限(design 基线 §6.4 第 1 条;方案页的数字列需要比表单更宽) */}
      <main className="mx-auto w-full max-w-[45rem] flex-1 px-base-lg py-base-xxl">
        <PlanView plan={plan} />
      </main>
      {/* P04 的声明由五段之「五、风险提示」承载,页脚不再重复(RULE-020) */}
      <SiteFooterShell hideLegal />
    </>
  );
}
