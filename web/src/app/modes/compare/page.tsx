/**
 * P08 对比页(server 壳,G 期票 06;prd-g §8,REQ-20260925-02)。
 *
 * 职责:解析 URL 勾选态 → 调 preview(单请求,响应自含元数据与试算,ADR-G-002)
 * → 参数表格。三种形态(prd-g §8.3):
 * - 无勾选 / 数量不足 → 引导回模式库(EMPTY 形态)
 * - 422(数量超限 / 未知模式 / 档案不完整)→ 错误态带后端文案
 * - 正常 → 表格(维度成行、模式成列,金额等宽右对齐,不可行列如实提示,RULE-043)
 * 全程 server 渲染,零客户端状态。
 */
import Link from "next/link";
import { cookies } from "next/headers";

import { trackPageView } from "@/lib/analytics";
import { apiPost } from "@/lib/api";
import { formatCurrency } from "@/lib/format-currency";
import { requireSession, SESSION_COOKIE } from "@/lib/session";
import { cn } from "@/lib/utils";

import { CREDIBILITY_BADGE } from "../state";
import type { components } from "@/lib/api-types";

type PreviewResponse = components["schemas"]["PreviewResponse"];

function ThCell({ children }: { children: React.ReactNode }) {
  return (
    <th scope="col" className="min-w-40 px-base-md py-base-sm text-left align-top">
      {children}
    </th>
  );
}

function TdCell({ children }: { children: React.ReactNode }) {
  return <td className="px-base-md py-base-sm align-top">{children}</td>;
}

export default async function ComparePage({
  searchParams,
}: {
  searchParams: Promise<{ modes?: string }>;
}) {
  // RULE-001:受保护页面
  await requireSession("/modes/compare");

  const params = await searchParams;
  const raw = typeof params.modes === "string" ? params.modes : "";
  const ids = raw
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);

  // 埋点:对比页触达(G 期 §9.5,page_view p08)
  await trackPageView("p08");

  // EMPTY 形态:0 或 1 个勾选,引导回模式库(AC-5 前置)
  if (ids.length < 2) {
    return (
      <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
        <div className="flex flex-col items-center gap-base-md rounded-sm border border-hairline bg-canvas-card px-base-lg py-base-xxl text-center">
          <h1 className="text-display-md text-ink">还没有勾选模式</h1>
          <p className="text-body-md text-ink-mute">
            先回模式库勾选 <span className="font-mono tabular-nums">2</span>-
            <span className="font-mono tabular-nums">3</span> 个模式,再开始对比。
          </p>
          <Link
            href="/modes"
            className="inline-flex min-h-11 items-center rounded-sm bg-primary px-base-lg text-button-md font-semibold text-on-primary"
          >
            回模式库勾选
          </Link>
        </div>
      </main>
    );
  }

  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value ?? "";
  const cookie = `${SESSION_COOKIE}=${token}`;

  let items: PreviewResponse["items"] | null = null;
  let backendMessage: string | null = null;
  try {
    const res = await apiPost<PreviewResponse>("/api/v1/plans/preview", { mode_ids: ids }, cookie);
    if (res.status === 200 && res.envelope.success && res.envelope.data) {
      items = res.envelope.data.items;
    } else {
      backendMessage = res.envelope.message ?? "暂时读不出来,再试一次";
    }
  } catch {
    backendMessage = "后端服务未响应。请运行 scripts/start.sh 启动后再试,或点击重试。";
  }

  if (items === null) {
    return (
      <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
        <div className="flex flex-col items-center gap-base-md rounded-sm border border-danger-border bg-danger-subtle px-base-lg py-base-xxl text-center">
          <h1 className="text-heading-lg text-danger">对比没算出来</h1>
          <p className="text-body-md text-ink-secondary">{backendMessage}</p>
          <a
            href={`/modes/compare?modes=${ids.join(",")}`}
            className="inline-flex min-h-11 items-center rounded-sm border border-hairline bg-canvas-soft px-base-lg text-button-md text-ink hover:bg-canvas-soft/60"
          >
            重试
          </a>
          <Link href="/modes" className="text-body-md text-primary underline">
            返回模式库
          </Link>
        </div>
      </main>
    );
  }

  // 维度行(行 = 维度,列 = 模式;RULE-041 参数表格,免费层无图形)
  return (
    <main className="mx-auto w-full max-w-6xl flex-1 px-base-lg py-base-xxl">
      <header className="flex flex-col gap-base-xs">
        <h1 className="text-display-lg text-ink">模式对比</h1>
        <p className="text-caption text-ink-mute">
          最多选 <span className="font-mono tabular-nums">3</span> 个,金额按你的档案试算
        </p>
      </header>

      <div className="mt-base-lg overflow-x-auto rounded-sm border border-hairline bg-canvas-card">
        <table className="w-full border-collapse text-body-md">
          {/* 模式名列 */}
          <thead>
            <tr className="border-b border-hairline bg-canvas-soft">
              <ThCell>对比维度</ThCell>
              {items.map((it) => (
                <ThCell key={it.mode_id}>
                  <span
                    className="block truncate text-body-md font-semibold text-ink"
                    title={it.name}
                  >
                    {it.name}
                  </span>
                </ThCell>
              ))}
            </tr>
          </thead>
          <tbody>
            <tr className="border-b border-hairline">
              <th
                scope="row"
                className="bg-canvas-soft px-base-md py-base-sm text-left align-top text-caption font-semibold text-ink-mute"
              >
                可信度
              </th>
              {items.map((it) => (
                <TdCell key={it.mode_id}>
                  <span
                    className={cn(
                      "inline-flex items-center rounded-full px-base-sm py-px text-micro-cap font-semibold",
                      CREDIBILITY_BADGE[it.credibility].cls,
                    )}
                  >
                    {CREDIBILITY_BADGE[it.credibility].label}
                  </span>
                </TdCell>
              ))}
            </tr>
            <tr className="border-b border-hairline">
              <th
                scope="row"
                className="bg-canvas-soft px-base-md py-base-sm text-left align-top text-caption font-semibold text-ink-mute"
              >
                一句话理念
              </th>
              {items.map((it) => (
                <TdCell key={it.mode_id}>{it.tagline}</TdCell>
              ))}
            </tr>
            <tr className="border-b border-hairline">
              <th
                scope="row"
                className="bg-canvas-soft px-base-md py-base-sm text-left align-top text-caption font-semibold text-ink-mute"
              >
                适合人群
              </th>
              {items.map((it) => (
                <TdCell key={it.mode_id}>
                  <span className="flex flex-wrap gap-base-xs">
                    {it.fit_for.map((f) => (
                      <span
                        key={f}
                        className="rounded-full bg-canvas-soft px-base-sm py-px text-micro-cap text-ink-secondary"
                      >
                        {f}
                      </span>
                    ))}
                  </span>
                </TdCell>
              ))}
            </tr>
            <tr className="border-b border-hairline">
              <th
                scope="row"
                className="bg-canvas-soft px-base-md py-base-sm text-left align-top text-caption font-semibold text-ink-mute"
              >
                桶结构
              </th>
              {items.map((it) => (
                <TdCell key={it.mode_id}>
                  <span className="flex flex-col gap-base-xs">
                    {it.buckets_meta.map((b) => (
                      <span key={b.name}>
                        <span className="font-medium text-ink">{b.name}</span>
                        <span className="block text-caption text-ink-mute">{b.share_desc}</span>
                      </span>
                    ))}
                  </span>
                </TdCell>
              ))}
            </tr>
            <tr className="border-b border-hairline">
              <th
                scope="row"
                className="bg-canvas-soft px-base-md py-base-sm text-left align-top text-caption font-semibold text-ink-mute"
              >
                每月转入(按你的档案)
              </th>
              {items.map((it) => {
                const insufficient = it.solution?.notices.includes("insufficient_income");
                return (
                  <TdCell key={it.mode_id}>
                    {it.solution && !insufficient ? (
                      <span className="flex flex-col gap-base-xs">
                        {it.solution.buckets.map((b) => (
                          <span
                            key={b.name}
                            className="flex items-baseline justify-between gap-base-sm"
                          >
                            <span className="text-caption text-ink-mute">{b.name}</span>
                            <span className="whitespace-nowrap text-right font-mono text-body-tabular text-ink">
                              {formatCurrency(b.amount_monthly_cents)}
                            </span>
                          </span>
                        ))}
                      </span>
                    ) : (
                      <span className="text-body-md text-warning">收入不足以覆盖当前结构</span>
                    )}
                  </TdCell>
                );
              })}
            </tr>
            <tr className="border-b border-hairline">
              <th
                scope="row"
                className="bg-canvas-soft px-base-md py-base-sm text-left align-top text-caption font-semibold text-ink-mute"
              >
                必要月支出
              </th>
              {items.map((it) => (
                <TdCell key={it.mode_id}>
                  {it.solution ? (
                    <span className="font-mono tabular-nums">
                      {formatCurrency(it.solution.necessary_monthly_cents)}
                    </span>
                  ) : (
                    "-"
                  )}
                </TdCell>
              ))}
            </tr>
            <tr>
              <th
                scope="row"
                className="bg-canvas-soft px-base-md py-base-sm text-left align-top text-caption font-semibold text-ink-mute"
              >
                应急金状态
              </th>
              {items.map((it) => (
                <TdCell key={it.mode_id}>
                  {it.solution ? (
                    it.solution.emergency_met ? (
                      <span className="text-success">已达标</span>
                    ) : (
                      <span className="text-warning">
                        进行中
                        {it.solution.emergency_months_to_fill != null ? (
                          <>
                            ,还差{" "}
                            <span className="font-mono tabular-nums">
                              {it.solution.emergency_months_to_fill}
                            </span>{" "}
                            个月
                          </>
                        ) : null}
                      </span>
                    )
                  ) : (
                    "-"
                  )}
                </TdCell>
              ))}
            </tr>
          </tbody>
        </table>
      </div>

      <div className="mt-base-lg flex flex-wrap gap-base-md">
        <Link
          href={ids.length > 0 ? `/modes?modes=${ids.join(",")}` : "/modes"}
          className="inline-flex min-h-11 items-center rounded-sm border border-hairline bg-canvas-soft px-base-lg text-button-md text-ink hover:bg-canvas-soft/60"
        >
          返回模式库
        </Link>
        <p className="self-center text-caption text-ink-mute">
          金额为按当前档案的每月转入试算,与单独生成该模式的结果一致
        </p>
      </div>
    </main>
  );
}
