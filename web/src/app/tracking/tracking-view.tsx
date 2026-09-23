/**
 * P05 追踪页视图(server 组件,票 04)。视觉对齐固化产物
 * `docs/design/e-tracking/p05-tracking-github.html`:五段纵排,段序固定。
 *
 * 全部数字来自后端 summary(ADR-E-002:前端零计算);桶名取当前 active 方案,
 * 历史行遇未知桶 id 按原样展示(RULE-028:历史按各自版本冻结)。
 * 「删除最新月」与进度空态文案是产物没有、产品规则要求的功能扩展
 * (见 .scratch/e-tracking/spec.md §4 调整清单 #2/#3)。
 */
import Link from "next/link";

import { buttonVariants } from "@/components/ui/button";
import { formatCurrency, formatMonthsTenths } from "@/lib/format-currency";
import { cn } from "@/lib/utils";

import { DeleteButton } from "./delete-button";
import { EntryCard } from "./entry-card";
import type { SnapshotItem, TrackingSummaryView } from "./state";

/** 视图所需的方案信息(桶名/桶集);无方案时为 null(整页空态)。 */
interface PlanInfo {
  buckets: { id: string; name: string }[];
}

function SectionTag({ children }: { children: React.ReactNode }) {
  return (
    <p className="mb-base-xs flex items-center gap-base-sm text-caption text-ink-mute">
      {children}
      <span aria-hidden="true" className="h-px flex-1 bg-hairline" />
    </p>
  );
}

/** 段 2 · 距离感进度(RULE-026:文本为主,具象月数;达标 = 绿色变体)。 */
function ProgressPanel({
  summary,
  hasSnapshots,
}: {
  summary: TrackingSummaryView;
  hasSnapshots: boolean;
}) {
  const emergency = summary.emergency;
  if (!emergency) {
    if (hasSnapshots) return null; // 有快照但方案缺应急桶(理论上不发生):不编造结论
    return (
      <div className="rounded-sm border border-hairline bg-canvas-soft px-base-lg py-base-md">
        <p className="text-body-md text-ink-secondary">
          完成第一次记录后,这里会显示「还差几个月攒满应急金」。
        </p>
      </div>
    );
  }
  const met = emergency.met;
  return (
    <div className="flex flex-col gap-base-xs rounded-sm border border-hairline bg-canvas-soft px-base-lg py-base-md">
      <p className="flex items-start gap-base-sm text-body-md text-ink-secondary">
        <span
          aria-hidden="true"
          className={cn("mt-2 size-2 shrink-0 rounded-full", met ? "bg-success" : "bg-warning")}
        />
        <span>
          {met ? (
            <>
              应急金已达标,缺口 <span className="font-mono tabular-nums text-success">0</span> 个月
            </>
          ) : (
            <>
              还差{" "}
              <span className="font-mono tabular-nums">
                {formatMonthsTenths(emergency.gap_months_tenths)}
              </span>{" "}
              个月攒满应急金
            </>
          )}
        </span>
      </p>
      <p className="text-caption text-ink-mute">
        已坚持 <span className="font-mono tabular-nums">{summary.persisted_months}</span> 个月
      </p>
    </div>
  );
}

/** 段 3 · 偏离提示条(温和文案,涨绿/跌红按已批准产物;未触发整段不渲染)。 */
function DeviationBars({
  summary,
  bucketNames,
  currentMonth,
}: {
  summary: TrackingSummaryView;
  bucketNames: Map<string, string>;
  currentMonth: string;
}) {
  const deviations = summary.latest?.deviations;
  if (!deviations || deviations.length === 0) return null; // null = 没得比;空 = 一切如常 —— 两者都不占位
  // 结论挂在「哪条快照」上就说哪个月:最新快照不是当月(如跳过当月)时,
  // 「本月」二字会变成对用户数字的误述(评审发现 #8)
  const labelMonth =
    summary.latest?.month === currentMonth ? "本月" : (summary.latest?.month ?? "");
  return (
    <div className="flex flex-col gap-base-sm">
      {deviations.map((d) => {
        const up = d.direction === "up";
        const percent = Math.round(d.deviation_bp / 100);
        const name = bucketNames.get(d.bucket_id) ?? d.bucket_id;
        return (
          <div
            key={d.bucket_id}
            role="status"
            className="flex items-start gap-base-md rounded-sm border border-hairline bg-canvas-soft px-base-lg py-base-md text-body-md text-ink-secondary"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
              className="mt-0.5 size-4 shrink-0 text-ink-mute"
            >
              <path
                d={
                  up
                    ? "M2.5 11.5 6 8l2.5 2 5-5.5M9.5 4.5h4v4"
                    : "M2.5 4.5 6 8l2.5-2 5 5.5M9.5 11.5h4v-4"
                }
              />
            </svg>
            <span>
              {labelMonth}
              {name}比上月{up ? "多" : "少"}了{" "}
              <span className={cn("font-mono tabular-nums", up ? "text-success" : "text-danger")}>
                {percent}%
              </span>
              。如果是有意的,标记『本月特殊』就好
            </span>
          </div>
        );
      })}
    </div>
  );
}

/** 段 4 · 历史快照列表(月倒序;特殊徽标;2×2 桶栅格;仅最新月带删除)。 */
function HistoryList({
  items,
  bucketNames,
  currentMonth,
}: {
  items: SnapshotItem[];
  bucketNames: Map<string, string>;
  currentMonth: string;
}) {
  if (items.length === 0) {
    return (
      <div className="flex flex-col items-center rounded-sm border border-hairline bg-canvas-card px-base-lg py-base-xxl text-center">
        <span
          aria-hidden="true"
          className="mb-base-lg grid size-12 place-items-center rounded-full bg-primary-soft text-primary"
        >
          <svg
            width="24"
            height="24"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <path d="M3.5 2.5h9v11l-4.5-3-4.5 3Z" />
          </svg>
        </span>
        <h2 className="text-display-md text-ink">还没有快照</h2>
        <p className="mt-base-xs text-body-md text-ink-mute">
          第 <span className="font-mono tabular-nums">1</span> 个月,从这里开始
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col rounded-sm border border-hairline bg-canvas-card px-base-lg py-base-xs">
      {items.map((item, index) => (
        <div
          key={item.id}
          className={cn(
            "flex flex-col gap-base-xs py-base-md",
            index > 0 && "border-t border-hairline",
          )}
        >
          <div className="flex items-center justify-between gap-base-sm">
            <span className="min-w-0 truncate text-body-md font-semibold text-ink">
              <span className="font-mono tabular-nums">{item.month}</span>
              {item.month === currentMonth ? (
                <span className="ml-base-sm text-caption font-normal text-ink-mute">当月</span>
              ) : null}
            </span>
            <div className="flex shrink-0 items-center gap-base-sm">
              {item.special_month ? (
                <span className="inline-flex items-center rounded-full bg-attention-subtle px-base-md py-px text-caption font-semibold text-warning">
                  特殊
                </span>
              ) : null}
              {index === 0 ? <DeleteButton month={item.month} /> : null}
            </div>
          </div>
          <div className="grid grid-cols-2 gap-x-base-lg gap-y-base-xs">
            {Object.entries(item.balances).map(([bucketId, cents]) => (
              <span
                key={bucketId}
                className="flex min-w-0 items-baseline justify-between gap-base-sm"
              >
                <span className="min-w-0 truncate text-caption text-ink-mute">
                  {bucketNames.get(bucketId) ?? bucketId}
                </span>
                <span
                  className={cn(
                    "whitespace-nowrap text-right font-mono text-body-tabular text-ink",
                    cents < 0 && "text-danger",
                  )}
                >
                  {formatCurrency(cents)}
                </span>
              </span>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

export function TrackingView({
  plan,
  data,
}: {
  plan: PlanInfo | null;
  data: {
    items: SnapshotItem[];
    summary: TrackingSummaryView;
    current_month: string;
  };
}) {
  const currentMonth = data.current_month; // 后端权威当月(评审发现 #4):前端不自算
  // EMPTY-E-01:无 active 方案 → 整页空态引导(prd-e §7.3)
  if (!plan) {
    return (
      <div className="flex flex-col items-center rounded-sm border border-hairline bg-canvas-card px-base-lg py-base-xxl text-center">
        <span
          aria-hidden="true"
          className="mb-base-lg grid size-12 place-items-center rounded-full bg-primary-soft text-primary"
        >
          <svg
            width="24"
            height="24"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <circle cx="8" cy="8" r="6.2" />
            <path d="M10.6 5.4 9 9 5.4 10.6 7 7Z" />
          </svg>
        </span>
        <h2 className="text-display-md text-ink">还没有方案,先去生成一份</h2>
        <p className="mt-base-xs text-body-md text-ink-mute">
          方案是追踪的基准:先回答几道题,拿到你的分账表。
        </p>
        <Link
          href="/questionnaire"
          className={cn(buttonVariants({ variant: "default" }), "mt-base-xl")}
        >
          去问卷
        </Link>
      </div>
    );
  }

  const bucketNames = new Map(plan.buckets.map((b) => [b.id, b.name]));
  const latest = data.items[0] ?? null;
  const recorded = latest?.month === currentMonth;
  const initial: Record<string, string> | null = recorded
    ? Object.fromEntries(
        Object.entries(latest.balances).map(([id, cents]) => [id, centsToYuanInput(cents)]),
      )
    : null;

  return (
    <div className="flex flex-col gap-base-xl">
      <header className="flex flex-col gap-base-xs">
        <h1 className="text-display-lg text-ink">追踪</h1>
        <p className="text-caption text-ink-mute">月度快照打卡</p>
      </header>

      {/* 段 1 · 录入卡(客户端岛) */}
      <EntryCard
        month={currentMonth}
        buckets={plan.buckets}
        initial={initial}
        initialSpecial={recorded ? (latest?.special_month ?? false) : false}
        recorded={recorded}
      />

      {/* 段 2 · 距离感进度 */}
      <section className="flex flex-col">
        <SectionTag>应急金进度</SectionTag>
        <ProgressPanel summary={data.summary} hasSnapshots={data.items.length > 0} />
      </section>

      {/* 段 3 · 偏离提示(触发时才占位) */}
      <section className="flex flex-col">
        <SectionTag>偏离提醒</SectionTag>
        <DeviationBars
          summary={data.summary}
          bucketNames={bucketNames}
          currentMonth={currentMonth}
        />
      </section>

      {/* 段 4 · 历史快照(首屏近 24 个月;更早的走 CSV 导出 —— 评审发现 #10 的如实说明) */}
      <section className="flex flex-col">
        <SectionTag>历史快照</SectionTag>
        <HistoryList items={data.items} bucketNames={bucketNames} currentMonth={currentMonth} />
        {data.summary.persisted_months > data.items.length ? (
          <p className="mt-base-xs text-caption text-ink-mute">
            共 <span className="font-mono tabular-nums">{data.summary.persisted_months}</span>{" "}
            个月,这里显示最近 <span className="font-mono tabular-nums">{data.items.length}</span>{" "}
            个月;更早的请用上方 CSV 导出查看。
          </p>
        ) : null}
      </section>

      {/* 段 5 · 导出(下载走 Route Handler 代理,红线 16) */}
      <section className="flex flex-col">
        <SectionTag>导出</SectionTag>
        <div className="flex flex-wrap gap-base-sm">
          <a
            href="/api/export/snapshots"
            className={cn(
              buttonVariants({ variant: "ghost" }),
              data.items.length === 0 && "pointer-events-none opacity-50",
            )}
            aria-disabled={data.items.length === 0}
            title={data.items.length === 0 ? "还没有快照可导出" : undefined}
          >
            导出快照 CSV
          </a>
          <a href="/api/export/plan" className={buttonVariants({ variant: "ghost" })}>
            导出方案 CSV
          </a>
        </div>
        {data.items.length === 0 ? (
          <p className="mt-base-xs text-caption text-ink-mute">还没有快照可导出</p>
        ) : null}
      </section>
    </div>
  );
}

/** 分 → 元输入字符串(整数运算,禁 float;与 format-currency 同族)。 */
function centsToYuanInput(cents: number): string {
  const sign = cents < 0 ? "-" : "";
  const abs = Math.abs(Math.trunc(cents));
  return `${sign}${Math.floor(abs / 100)}.${String(abs % 100).padStart(2, "0")}`;
}
