/**
 * P05 追踪页(server 壳,票 04;prd-e §8,REQ-20260922-01)。
 *
 * 职责:要求会话 → 并行取数(快照+摘要、当前方案)→ 交给五段视图。
 * 三种页面形态在这里定型(prd-e §8.3):
 * - 无 active 方案(404) → 整页空态引导去问卷(EMPTY-E-01)
 * - 后端不可达/其它错误 → 页级错误态 + 重试(AC-16,不白屏)
 * - 正常 → 五段视图(录入卡为客户端岛)
 * 与 P04 不同:**无方案不重定向** —— 追踪页自己承载引导(产物整页空态);
 * 桶名(视图需要)来自同一次方案取数,不二次请求。
 */
import { cookies } from "next/headers";

import { SiteFooterShell } from "@/components/site-footer-shell";
import { trackPageView } from "@/lib/analytics";
import { apiGet } from "@/lib/api";
import { requireSession, SESSION_COOKIE } from "@/lib/session";

import type { PlanView as Plan } from "../plan/state";
import type { SnapshotsData, TrackingState } from "./state";
import { TrackingView } from "./tracking-view";

/** 服务端认定的当前自然月(YYYY-MM)——仅整页空态的占位数据用;正常路径一律以后端下发的 current_month 为准。 */
function currentMonth(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
}

/** 无方案时的占位摘要(空态由视图渲染引导,不显示任何数字)。 */
const EMPTY_SUMMARY: SnapshotsData["summary"] = {
  persisted_months: 0,
  latest: null,
  emergency: null,
};

export default async function TrackingPage() {
  // RULE-001:受保护页面,未登录跳登录并在登录后回到本页
  await requireSession("/tracking");

  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value ?? "";
  const cookie = `${SESSION_COOKIE}=${token}`;

  let state: TrackingState;
  let plan: Plan | null = null;
  try {
    const [snapshots, planRes] = await Promise.all([
      apiGet<SnapshotsData>("/api/v1/snapshots?limit=24", { cookie }),
      apiGet<Plan>("/api/v1/plans/active", { cookie }),
    ]);

    // 三态判定:方案「没有」(404) 是合法形态;两份数据任何一份「读不出来」(非 404)
    // 都按错误态 —— 不能拿半份数据渲染。
    const planMissing = planRes.status === 404;
    if (planRes.status === 200 && planRes.envelope.success && planRes.envelope.data) {
      plan = planRes.envelope.data;
    }
    const planBroken = !plan && !planMissing;
    const snapshotsOk =
      snapshots.status === 200 && snapshots.envelope.success && snapshots.envelope.data !== null;

    if (planBroken || !snapshotsOk) {
      state = { kind: "error" };
    } else if (planMissing) {
      state = { kind: "no-plan" };
    } else {
      state = { kind: "ready", data: snapshots.envelope.data as SnapshotsData };
    }
  } catch {
    state = { kind: "error" };
  }

  // 埋点:无方案空态用户也真实看到了本页(与 P04「跳转前不记」不同)
  await trackPageView("p05");

  if (state.kind === "error") {
    return (
      <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
        <div className="flex flex-col items-center gap-base-md rounded-sm border border-danger-border bg-danger-subtle px-base-lg py-base-xxl text-center">
          <h1 className="text-heading-lg text-danger">追踪数据没有取回来</h1>
          <p className="text-body-md text-ink-secondary">
            可能是网络或后端服务暂时不可用。恢复后点击重试,你的数据都在。
          </p>
          <a
            href="/tracking"
            className="inline-flex min-h-11 items-center rounded-sm border border-hairline bg-canvas-soft px-base-lg text-button-md text-ink hover:bg-canvas-soft/60"
          >
            重试
          </a>
        </div>
      </main>
    );
  }

  const planBuckets = plan?.buckets.map((b) => ({ id: b.bucket_id, name: b.name })) ?? null;

  return (
    <>
      <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
        {state.kind === "no-plan" || planBuckets === null ? (
          <TrackingView
            plan={null}
            data={{ items: [], summary: EMPTY_SUMMARY, current_month: currentMonth() }}
          />
        ) : (
          <TrackingView plan={{ buckets: planBuckets }} data={state.data} />
        )}
      </main>
      {/* P05 页脚沿 RULE-020/031:一行简述 + 完整声明展开 */}
      <SiteFooterShell />
    </>
  );
}
