/**
 * P01 产品首页(B 期改造,design-v2 §1.1;od-redesign 期按固化产物重排视觉)。
 *
 * 与 A 期的区别:这是一个**产品**首页,承载两种真实形态(有方案 / 没有方案)。
 * A 期为验证工程骨架放在这里的服务状态卡片与 Token 样例区**已移除** ——
 * 健康状态随页脚(RULE-020 的页脚承载),不再占用首页正文。
 *
 * 取数在此完成(server 组件),形态判断与渲染交给 `HomeView`;本页不做任何重算。
 * 产物(od-redesign)的 hero 是素带:标题 + 副标题,无 mesh、无分隔线 —— 原
 * MeshBackdrop 已随基线切换移除,hero 视觉全靠排版(产物即如此)。
 */
import { cookies } from "next/headers";

import { SiteFooterShell } from "@/components/site-footer-shell";
import { trackPageView } from "@/lib/analytics";
import { apiGet } from "@/lib/api";
import { requireSession, SESSION_COOKIE } from "@/lib/session";

import { HomeView, type HomeState } from "./home-view";
import type { PlanView } from "./plan/state";

/** 读当前方案:200 有方案 / 404 还没生成过 / 其它与网络失败都归「读不出来」。 */
async function loadHomeState(): Promise<HomeState> {
  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value ?? "";

  try {
    const { status, envelope } = await apiGet<PlanView>("/api/v1/plans/active", {
      cookie: `${SESSION_COOKIE}=${token}`,
    });
    if (status === 200 && envelope.success && envelope.data) {
      return { kind: "plan", plan: envelope.data };
    }
    if (status === 404) return { kind: "empty" };
    return { kind: "error" };
  } catch {
    return { kind: "error" };
  }
}

export default async function Page() {
  // RULE-001:P01 属受保护页面,未登录跳登录页并在登录后回跳
  await requireSession("/");
  // 埋点(prd-v1 §9.5:页面触达)。放在会话校验之后:未登录不该产生触达记录
  await trackPageView("p01");

  const state = await loadHomeState();

  return (
    <>
      {/* hero 带(产物:素带,标题 + 副标题,下无分隔线) */}
      <section className="mx-auto w-full max-w-xl px-base-lg pt-base-xxl pb-base-lg">
        <h1 className="text-display-xxl text-ink">智策理财</h1>
        <p className="mt-base-sm text-body-md text-ink-mute">
          双层理财决策工具 · 免费层告诉你怎么做
        </p>
      </section>
      <main className="mx-auto flex w-full max-w-xl flex-1 flex-col gap-base-xl px-base-lg py-base-lg">
        <HomeView state={state} />
      </main>
      {/* 首页页脚保留健康状态与一行免责声明(design-v2 v0.3 / RULE-020) */}
      <SiteFooterShell />
    </>
  );
}
