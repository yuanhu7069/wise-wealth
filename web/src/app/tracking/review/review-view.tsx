/**
 * P09 季度复盘视图(票 04;视觉对齐固化产物 docs/design/h-plus/p09-review-github.html)。
 *
 * 纯 server 组件:全部数字来自 `/api/v1/tracking/review` 单请求(ADR-H-003),
 * 页面不重算 —— 距离感表述的口径(RULE-050)在域层,这里只渲染。
 * 三卡布局沿产物:桌面横排(容器放宽,T2 调整清单 #1)/ 移动端纵排。
 */
import { formatCurrency } from "@/lib/format-currency";
import { cn } from "@/lib/utils";

import { QuarterSwitcher } from "./quarter-switcher";
import { REVIEW_COPY, type ReviewData } from "./state";

/** 卡壳(产物 .stat-card):hairline 一像素边 + 平面卡体。 */
function StatCard({ eyebrow, children }: { eyebrow: string; children: React.ReactNode }) {
  return (
    <section className="flex min-w-0 flex-col rounded-sm border border-hairline bg-canvas-card p-base-lg">
      <p className="text-caption font-semibold tracking-wide text-ink-mute">{eyebrow}</p>
      {children}
    </section>
  );
}

/** 带符号变化额:正 = success 绿,负 = danger 红(色盲双通道:符号永远在)。 */
function Delta({ cents }: { cents: number }) {
  const up = cents >= 0;
  return (
    <p
      className={cn(
        "mt-base-xs text-right font-mono text-body-md tabular-nums",
        up ? "text-success" : "text-danger",
      )}
    >
      {up ? "+" : "−"}
      {formatCurrency(Math.abs(cents))}
    </p>
  );
}

export function ReviewView({ data }: { data: ReviewData }) {
  const e = data.emergency;
  return (
    <div className="flex flex-col gap-base-lg">
      <header className="flex flex-col gap-base-xs">
        <h1 className="text-display-lg text-ink">{REVIEW_COPY.title}</h1>
        <p className="text-caption text-ink-mute">{REVIEW_COPY.subtitle}</p>
      </header>

      <QuarterSwitcher quarter={data.quarter} />

      <div className="grid grid-cols-1 items-start gap-base-md md:grid-cols-[1fr_1.4fr_1fr]">
        {/* 卡 1 · 坚持月数(RULE-050①:全部快照累计,具象距离,无完成度百分比) */}
        <StatCard eyebrow={REVIEW_COPY.persistedEyebrow}>
          <p className="mt-base-sm font-mono text-heading-xl font-semibold tabular-nums text-ink">
            {data.persisted_months}
            <span className="ml-1 text-body-lg font-semibold">{REVIEW_COPY.persistedUnit}</span>
          </p>
          <p className="mt-base-xs text-caption text-ink-mute">{REVIEW_COPY.persistedCaption}</p>
        </StatCard>

        {/* 卡 2 · 各桶变化(RULE-050②:同名桶季末 − 季初;不可环比给说明不编零) */}
        <StatCard eyebrow={REVIEW_COPY.deltaEyebrow}>
          {data.comparable ? (
            <div className="flex flex-col">
              {data.special_months.length > 0 ? (
                /* T2 调整清单 #4:产物把 pill 放在单个桶行;数据真源是「月」,按卡级标注 */
                <p className="mb-base-xs">
                  <span className="inline-flex items-center rounded-full bg-attention-subtle px-base-sm py-px text-micro-cap font-semibold tracking-wide text-warning">
                    {data.special_months.join("、")} {REVIEW_COPY.specialNote}
                  </span>
                </p>
              ) : null}
              {data.buckets.map((b) => (
                <div
                  key={b.bucket_id}
                  className="border-t border-hairline py-base-sm first:border-t-0"
                >
                  <p className="flex items-center justify-between gap-base-sm">
                    <span className="min-w-0 truncate text-body-md font-semibold text-ink">
                      {b.name}
                    </span>
                  </p>
                  <p className="mt-base-xs overflow-hidden text-right font-mono text-body-md tabular-nums whitespace-nowrap text-ink">
                    {formatCurrency(b.quarter_start_cents)} → {formatCurrency(b.latest_cents)}
                  </p>
                  <Delta cents={b.delta_cents} />
                </div>
              ))}
            </div>
          ) : (
            <p className="py-base-lg text-center text-body-md text-ink-mute">
              {data.quarter_snapshot_count === 1 ? REVIEW_COPY.singleOne : REVIEW_COPY.singleMany}
            </p>
          )}
        </StatCard>

        {/* 卡 3 · 应急缺口收敛(RULE-050③:线性外推是参考不是承诺) */}
        <StatCard eyebrow={REVIEW_COPY.gapEyebrow}>
          {e ? (
            <>
              <div className="flex items-baseline justify-between gap-base-md border-t border-hairline py-base-sm first:border-t-0">
                <span className="shrink-0 text-caption text-ink-mute">
                  {REVIEW_COPY.gapStartToEnd}
                </span>
                <span className="truncate text-right font-mono text-body-md tabular-nums text-ink">
                  {formatCurrency(e.start_gap_cents)} → {formatCurrency(e.current_gap_cents)}
                </span>
              </div>
              {e.avg_monthly_convergence_cents != null ? (
                <div className="flex items-baseline justify-between gap-base-md border-t border-hairline py-base-sm">
                  <span className="shrink-0 text-caption text-ink-mute">{REVIEW_COPY.gapAvg}</span>
                  <span className="text-right font-mono text-body-md tabular-nums text-ink">
                    {formatCurrency(e.avg_monthly_convergence_cents)}
                  </span>
                </div>
              ) : null}
              <div className="mt-base-md flex items-start gap-base-sm border-t border-hairline pt-base-md text-body-md text-ink-secondary">
                <span
                  aria-hidden="true"
                  className={cn(
                    "mt-2 size-2 shrink-0 rounded-full",
                    e.met ? "bg-success" : "bg-warning",
                  )}
                />
                {e.met ? (
                  <span>{REVIEW_COPY.metText}</span>
                ) : e.months_to_goal !== null ? (
                  <span>
                    {REVIEW_COPY.forecastPrefix}{" "}
                    <span className="font-mono tabular-nums">{e.months_to_goal}</span>{" "}
                    {REVIEW_COPY.forecastSuffix}
                  </span>
                ) : (
                  <span>{REVIEW_COPY.singleMany}</span>
                )}
              </div>
              {e.months_to_goal != null && !e.met ? (
                <p className="mt-base-xs text-caption text-ink-mute">{REVIEW_COPY.forecastNote}</p>
              ) : null}
            </>
          ) : (
            <p className="py-base-lg text-center text-body-md text-ink-mute">
              {data.quarter_snapshot_count > 0 ? REVIEW_COPY.noEmergency : REVIEW_COPY.singleMany}
            </p>
          )}
        </StatCard>
      </div>
    </div>
  );
}
