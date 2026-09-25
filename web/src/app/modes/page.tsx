/**
 * P06 模式库页(server 壳,F 期票 04;prd-f §8,REQ-20260925-01)。
 *
 * 职责:要求会话 → 取模式列表 → 交给视图。三种页面形态(prd-f §8.3):
 * - 后端不可达/其它错误 → 页级错误态 + 重试(AC-11,不白屏)
 * - 列表为空 → 防御空态(EMPTY-F-01;构建期兜底使其实际不可达,AC-13)
 * - 正常 → 卡片网格(模式卡为客户端岛:收展 + CTA)
 */
import { cookies } from "next/headers";

import { trackPageView } from "@/lib/analytics";
import { apiGet } from "@/lib/api";
import { requireSession, SESSION_COOKIE } from "@/lib/session";

import { ModesView } from "./modes-view";
import type { ModesData, ModesState } from "./state";

export default async function ModesPage({
  searchParams,
}: {
  searchParams: Promise<{ modes?: string }>;
}) {
  // RULE-001:受保护页面,未登录跳登录并在登录后回到本页
  await requireSession("/modes");

  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value ?? "";
  const cookie = `${SESSION_COOKIE}=${token}`;

  let state: ModesState;
  try {
    const res = await apiGet<ModesData>("/api/v1/modes", { cookie });
    if (res.status === 200 && res.envelope.success && res.envelope.data) {
      const data = res.envelope.data as ModesData;
      state = data.items.length > 0 ? { kind: "ready", data } : { kind: "empty" };
    } else {
      state = { kind: "error" };
    }
  } catch {
    state = { kind: "error" };
  }

  // 埋点:列表页触达(F 期 §9.5,page_view p06)
  await trackPageView("p06");

  if (state.kind === "error") {
    return (
      <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
        <div className="flex flex-col items-center gap-base-md rounded-sm border border-danger-border bg-danger-subtle px-base-lg py-base-xxl text-center">
          <h1 className="text-heading-lg text-danger">模式库暂时读不出来</h1>
          <p className="text-body-md text-ink-secondary">
            服务端没有回应,可能是网络中断。稍后重试即可,你的问卷与方案都不受影响。
          </p>
          <a
            href="/modes"
            className="inline-flex min-h-11 items-center rounded-sm border border-hairline bg-canvas-soft px-base-lg text-button-md text-ink hover:bg-canvas-soft/60"
          >
            重试
          </a>
        </div>
      </main>
    );
  }

  if (state.kind === "empty") {
    // EMPTY-F-01:构建期兜底(config/modes 目录空即构建失败)使其实际不可达,纯防御
    return (
      <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
        <div className="flex flex-col items-center gap-base-md rounded-sm border border-hairline bg-canvas-card px-base-lg py-base-xxl text-center">
          <span
            aria-hidden="true"
            className="grid size-12 place-items-center rounded-full bg-primary-soft text-primary"
          >
            <svg
              aria-hidden="true"
              width="24"
              height="24"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M2.5 3.5h4.2c.8 0 1.3.4 1.3 1v8.2c0-.6-.5-1-1.3-1H2.5Zm11 0H9.3c-.8 0-1.3.4-1.3 1v8.2c0-.6.5-1 1.3-1h4.2Z" />
            </svg>
          </span>
          <h1 className="text-display-md text-ink">模式库暂时为空</h1>
          <p className="text-body-md text-ink-mute">暂时没有可用的分账模式,稍后再来看看。</p>
          <a
            href="/modes"
            className="inline-flex min-h-11 items-center rounded-sm border border-hairline bg-canvas-soft px-base-lg text-button-md text-ink hover:bg-canvas-soft/60"
          >
            重试
          </a>
        </div>
      </main>
    );
  }

  // 勾选态回传(AC-14):对比页「返回模式库」链接带回 ?modes=,URL 承载零持久化
  const params = await searchParams;
  const initialSelected =
    typeof params.modes === "string"
      ? params.modes
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean)
      : [];

  return <ModesView data={state.data} initialSelected={initialSelected} />;
}
