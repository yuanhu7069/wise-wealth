/**
 * P04 方案页五段(P04 五段结构见 prd-v1 §8.2,文案取值参照原型 design-v1.html 的 P04)。
 *
 * 五段:分配总览 → 投资账户内部配置(L2) → 执行规则(L3) → 落地建议 → 风险提示。
 * 全部数字来自后端方案**快照**(`/api/v1/plans/active`),页面不重算、不猜测 ——
 * 页面与引擎不一致过一次,用户就再也不会信任这张表。
 *
 * 无客户端状态:生成/重试发生在 P03 步 6,本页只渲染既有方案,故是纯 server 组件
 * (基线 §7.4「能服务端渲染就不上客户端」)。
 */
import { RefreshCw } from "lucide-react";
import Link from "next/link";

import { Disclaimer } from "@/components/disclaimer";
import { Badge } from "@/components/ui/badge";
import { buttonVariants } from "@/components/ui/button";
import { formatCurrency, formatMonthsTenths } from "@/lib/format-currency";
import { cn } from "@/lib/utils";

import { chartBgClass, NOTICE_COPY, type PlanView as Plan } from "./state";

/** 段骨架:统一段间距(--space-xxl)与段标题字阶(design-v2 §3.1「P04 五段」行)。 */
function Section({
  title,
  children,
  className,
}: {
  title: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <section className={cn("flex flex-col gap-base-md", className)}>
      <h2 className="text-heading-md text-ink">{title}</h2>
      {children}
    </section>
  );
}

/** 提示条:引擎给事实,文案取自 NOTICE_COPY(RULE-008 三要素)。 */
function NoticeBar({ notice }: { notice: keyof typeof NOTICE_COPY }) {
  const copy = NOTICE_COPY[notice];
  const severe = notice === "insufficient_income";
  /*
   * 底色用 bg-card 而非 bg-subtle:严重提示的文字色是 --color-danger,它在 bg-subtle 上
   * 亮暗两态都低于正文下限 4.5:1(实测 4.00:1 / 4.24:1),换到 bg-card 后是 4.55:1 / 4.61:1
   * (2026-09-11 走查实测,test-report-b.md §8-E)。
   */
  return (
    <div
      role="status"
      className="flex flex-col gap-base-xs rounded-md border border-hairline bg-canvas-card px-base-lg py-base-md"
    >
      <p className={cn("text-body-md", severe ? "text-danger" : "text-ink")}>{copy.title}</p>
      <p className="text-caption text-ink-secondary">{copy.description}</p>
    </div>
  );
}

/**
 * 应急金状态(AC-25/26/31)。**不塞进表格单元格**:表格只放模式配置里的用途原文,
 * 状态类信息集中在这里 —— 否则同一事实(备用的用途)会有配置与状态两个来源。
 */
function EmergencyPanel({ emergency }: { emergency: Plan["emergency"] }) {
  const coverage = formatMonthsTenths(emergency.coverage_tenths);
  return (
    <div className="flex flex-col gap-base-xs rounded-md bg-canvas-soft px-base-lg py-base-md">
      <p className="text-body-md text-ink-secondary">
        应急金:
        <span className={emergency.is_met ? "text-success" : "text-ink"}>
          {emergency.is_met ? "已达标" : "未达标"}
        </span>
        {" · "}目标 {emergency.months} 个月 = {formatCurrency(emergency.target_cents)}
        <span className="text-caption text-ink-mute-2">
          (必要月支出 {formatCurrency(emergency.necessary_monthly_cents)})
        </span>
      </p>
      <p className="text-caption text-ink-secondary">
        现有存款已覆盖约 {coverage} 个月
        {emergency.is_met
          ? ""
          : `,按每月 ${formatCurrency(emergency.monthly_toward_emergency_cents)} 的节奏,约 ${
              emergency.months_to_fill ?? 0
            } 个月补齐`}
      </p>
      {emergency.is_met ? (
        <p className="text-caption text-ink-secondary">
          {emergency.surplus_cents > 0
            ? `超出应急目标的部分不再单独预留:投资账户已有家底 ${formatCurrency(emergency.surplus_cents)}`
            : "超出应急目标的部分不再单独预留,份额自动进入投资账户"}
        </p>
      ) : null}
    </div>
  );
}

/** 一、分配总览(基线 §6.1 表格:数值右对齐 / tabular-nums / 行高 44 / 仅横向细线)。 */
function OverviewSection({ plan }: { plan: Plan }) {
  return (
    <Section title="一、分配总览">
      <div className="overflow-hidden rounded-lg bg-card shadow-card">
        <table className="w-full border-collapse">
          <thead>
            <tr className="bg-canvas-soft text-caption text-ink-mute-2">
              <th scope="col" className="px-base-lg py-base-sm text-left">
                账户
              </th>
              <th scope="col" className="px-base-lg py-base-sm text-right">
                每月转入
              </th>
              <th scope="col" className="px-base-lg py-base-sm text-left">
                用途
              </th>
            </tr>
          </thead>
          <tbody>
            {plan.buckets.map((bucket, i) => (
              <tr key={bucket.bucket_id} className="border-t border-hairline">
                <th scope="row" className="h-11 px-base-lg text-left text-body-md font-normal">
                  <span className="flex items-center gap-base-sm text-ink">
                    <span
                      aria-hidden="true"
                      className={cn("size-2.5 shrink-0 rounded-full", chartBgClass(i))}
                    />
                    {bucket.name}
                  </span>
                </th>
                <td className="h-11 px-base-lg text-right text-body-tabular text-ink whitespace-nowrap">
                  {formatCurrency(bucket.amount_monthly_cents)}
                </td>
                <td className="h-11 px-base-lg text-caption text-ink-mute">{bucket.purpose}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <EmergencyPanel emergency={plan.emergency} />
    </Section>
  );
}

/** 二、投资账户内部配置:横向比例条 + 文字并列(色盲可读,基线 §6.2 规则 2)。 */
function L2Section({ l2 }: { l2: Plan["l2"] }) {
  return (
    <Section title="二、投资账户内部配置">
      <div className="flex flex-col gap-base-md rounded-lg bg-card p-base-lg shadow-card">
        <div className="flex flex-wrap items-center gap-base-sm">
          <Badge>{l2.name}</Badge>
          <span className="text-caption text-ink-mute">{l2.reason}</span>
        </div>

        <div className="flex h-3 w-full overflow-hidden rounded-full bg-canvas-soft">
          {l2.classes.map((cls, i) => (
            <span
              key={cls.name}
              className={cn("h-full", chartBgClass(i))}
              // 宽度即占比(万分比 → %)。比例条是图形,宽度仍需内联 —— 工具类无法表达任意百分比。
              style={{ width: `${cls.basis_points / 100}%` }}
            />
          ))}
        </div>

        <ul className="flex flex-wrap gap-base-md">
          {l2.classes.map((cls, i) => (
            <li
              key={cls.name}
              className="flex items-center gap-base-xs text-caption text-ink-secondary tabular-nums"
            >
              <span
                aria-hidden="true"
                className={cn("size-2.5 shrink-0 rounded-full", chartBgClass(i))}
              />
              {cls.name} {Math.round(cls.basis_points / 100)}%
            </li>
          ))}
        </ul>

        {l2.note ? <p className="text-caption text-ink-mute">{l2.note}</p> : null}
        <p className="text-caption text-ink-mute">仅到大类资产,不涉及任何具体产品</p>
      </div>
    </Section>
  );
}

/** 三、执行规则(L3 自动给出,产品 PRD §4.3.2 三 + §7.3 现金再平衡为默认)。 */
function ExecutionSection({ plan }: { plan: Plan }) {
  return (
    <Section title="三、执行规则">
      <div className="flex flex-col gap-base-sm rounded-lg bg-card p-base-lg text-body-md text-ink-secondary shadow-card">
        <p>1. 每月发薪日定投,金额按上表执行</p>
        <p>2. 投资账户偏离目标比例 5% 时触发再平衡</p>
        <p>3. 优先用新增资金补低配(现金再平衡,规避赎回费与择时)</p>
        {plan.emergency.is_met ? null : (
          <p className="text-caption text-ink-mute">
            应急金未达标期间,备用账户优先补应急(每月{" "}
            {formatCurrency(plan.emergency.monthly_toward_emergency_cents)},约{" "}
            {plan.emergency.months_to_fill ?? 0} 个月达标),达标后这部分份额转入投资账户。
          </p>
        )}
      </div>
    </Section>
  );
}

/**
 * 四、落地建议。文案按**桶派生**而非按模式写死:加一个模式不该回头改这个页面
 * (原型里的四条卡建议是四账户模式的示例,派生后对三桶模式同样成立)。
 */
function SuggestionSection({ plan }: { plan: Plan }) {
  const names = plan.buckets.map((b) => `${b.name}(${b.purpose})`).join(" / ");
  return (
    <Section title="四、落地建议">
      <div className="flex flex-col gap-base-sm rounded-lg bg-card p-base-lg shadow-card">
        <p className="text-body-md text-ink-secondary">
          建议开 2-3 张银行卡分别对应上表账户:{names}。资金到账后按上表金额分配。
        </p>
        <p className="text-caption text-ink-mute">仅为账户组织建议,不涉及任何划转操作</p>
      </div>
    </Section>
  );
}

export function PlanView({ plan }: { plan: Plan }) {
  return (
    <div className="flex flex-col gap-base-xxl">
      <header className="flex flex-col gap-base-xs">
        <h1 className="text-display-md text-ink">你的方案</h1>
        <p className="text-caption text-ink-mute">
          {plan.l1_mode_name} · 第 {plan.version} 版 · 生成于 {plan.created_date}
        </p>
      </header>

      {plan.notices.length > 0 ? (
        <div className="flex flex-col gap-base-sm">
          {plan.notices.map((notice) => (
            <NoticeBar key={notice} notice={notice} />
          ))}
        </div>
      ) : null}

      <OverviewSection plan={plan} />
      <L2Section l2={plan.l2} />
      <ExecutionSection plan={plan} />
      <SuggestionSection plan={plan} />

      <Section title="五、风险提示">
        {/* 声明全文的唯一来源是 lib/disclaimer-copy.ts;这里去掉组件的分隔线与宽度上限 ——
            它在方案页是「一段」而不是「页脚一块」 */}
        <Disclaimer className="mx-0 max-w-none border-t-0 pt-0" />
      </Section>

      <div className="flex flex-wrap gap-base-md">
        {/* 重新生成 = 回 P03 从步 1 走(restart=1;作答仍预填,只改变化的数字),再生成即版本 +1 */}
        <Link href="/questionnaire?restart=1" className={buttonVariants({ variant: "ghost" })}>
          <RefreshCw className="size-4" aria-hidden="true" />
          重新生成
        </Link>
        <Link href="/" className={buttonVariants({ variant: "ghost" })}>
          返回首页
        </Link>
      </div>
    </div>
  );
}
