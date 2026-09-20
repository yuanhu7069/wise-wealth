/**
 * P01 产品首页的三形态(od-redesign 固化产物 p01-home-github.html;形态与文案对照原型 design-v1.html 的 P01)。
 *
 * 三种形态来自同一份取数结果:
 * - **有方案**:摘要卡(模式名 + 每月可投资 + 应急金状态)与两个入口
 * - **空态**:EMPTY-001 三要素(图标 + 一句说明 + 引导操作)
 * - **取数失败**:局部错误态 + 重试 —— 后端不可达时**不能**降级成「你还没有方案」,
 *   那会让用户去重答一遍问卷,而问题根本不在问卷
 *
 * 摘要数字全部来自方案**快照**(`/api/v1/plans/active`),与 P04 同源 ——
 * 首页与方案页显示同一个金额,是「同一份方案」这件事最直接的证据。
 */
import { AlertTriangle, Compass, RefreshCw } from "lucide-react";
import Link from "next/link";

import type { PlanView } from "@/app/plan/state";
import { Badge } from "@/components/ui/badge";
import { buttonVariants } from "@/components/ui/button";
import { formatCurrency, formatMonthsTenths } from "@/lib/format-currency";
import { cn } from "@/lib/utils";

/** 首页形态。取数在页面(server)完成,这里只负责渲染。 */
export type HomeState = { kind: "plan"; plan: PlanView } | { kind: "empty" } | { kind: "error" };

/**
 * 应急金状态句(design-v2 §3.1「应急状态句」):距离感文案,不显示百分比。
 * 达标是状态不是提示 —— 同一事实只在这里说一次,不加额外的告警条。
 * 状态用圆点表达(产物 .status-dot):绿 = 达标,黄 = 有差距。
 */
function emergencyLine(plan: PlanView): string {
  const coverage = formatMonthsTenths(plan.emergency.coverage_tenths);
  if (plan.emergency.is_met) return `应急金已覆盖(约 ${coverage} 个月)`;
  return `应急金当前约 ${coverage} 个月,还差约 ${plan.emergency.months_to_fill ?? 0} 个月达标`;
}

/** 有方案:摘要卡 + [查看完整方案][重新生成](产物形态 A)。 */
function PlanSummary({ plan }: { plan: PlanView }) {
  return (
    <section
      className="flex flex-col rounded-sm border border-hairline bg-canvas-card"
      aria-label="当前方案摘要"
    >
      <div className="flex flex-wrap items-center gap-base-sm border-b border-hairline px-base-lg py-base-md">
        <Badge>{plan.l1_mode_name}</Badge>
        <span className="text-caption text-ink-mute">
          第 <span className="font-mono tabular-nums">{plan.version}</span> 版 · 生成于{" "}
          <span className="font-mono tabular-nums">{plan.created_date}</span>
        </span>
      </div>

      <div className="px-base-lg py-base-lg">
        <p className="text-caption text-ink-mute">每月可投资</p>
        {/* 摘要里唯一的「大字」:用户扫一眼首页要拿到的就是这个数(产物 28 像素等宽右对齐) */}
        <p className="text-right font-mono text-display-xl text-ink tabular-nums">
          {formatCurrency(plan.investable_monthly_cents)}
        </p>
      </div>

      <p className="flex items-start gap-base-sm border-t border-hairline px-base-lg py-base-md text-body-md text-ink">
        <span
          aria-hidden="true"
          className={cn(
            "mt-2 size-2 shrink-0 rounded-full",
            plan.emergency.is_met ? "bg-success" : "bg-warning",
          )}
        />
        <span>{emergencyLine(plan)}</span>
      </p>

      <div className="flex flex-wrap gap-base-md px-base-lg pb-base-lg">
        <Link href="/plan" className={buttonVariants({ size: "lg" })}>
          查看完整方案
        </Link>
        {/* 重新生成 = 回问卷从步 1 走(restart=1;答案仍预填,改完数字再生成即版本 +1) */}
        <Link
          href="/questionnaire?restart=1"
          className={buttonVariants({ variant: "ghost", size: "lg" })}
        >
          <RefreshCw className="size-4" aria-hidden="true" />
          重新生成
        </Link>
      </div>
    </section>
  );
}

/** 空态三要素(EMPTY-001):图标 + 一句人话说明 + 明确的引导操作(产物形态 B)。 */
function EmptyGuide() {
  return (
    <section
      className="flex flex-col items-center rounded-sm border border-hairline bg-canvas-card px-base-lg py-base-xxl text-center"
      aria-label="还没有方案"
    >
      <span
        aria-hidden="true"
        className="grid size-12 place-items-center rounded-full bg-primary-bg-subdued-hover text-primary"
      >
        <Compass className="size-6" />
      </span>
      <h2 className="mt-base-lg text-heading-lg text-ink">还没有方案</h2>
      <p className="mt-base-sm text-body-md text-ink-mute">
        完成 <span className="font-mono tabular-nums">6</span> 步问卷(约{" "}
        <span className="font-mono tabular-nums">2</span> 分钟),得到你的第一份可照做的方案
      </p>
      <Link href="/questionnaire" className={cn(buttonVariants({ size: "lg" }), "mt-base-xl")}>
        开始问卷
      </Link>
    </section>
  );
}

/**
 * 取数失败(局部):三要素「发生了什么 + 为什么 + 怎么办」,且给一个真的能重试的动作(产物形态 C)。
 * 用整页刷新而非客户端 refetch:首页没有客户端状态,重试就是重新取一次数。
 */
function LoadError() {
  return (
    <section
      role="alert"
      aria-label="方案读取失败"
      className="flex flex-col rounded-sm border border-danger-border bg-danger-subtle p-base-lg"
    >
      <div className="flex items-start gap-base-md">
        <AlertTriangle className="mt-0.5 size-5 shrink-0 text-danger" aria-hidden="true" />
        <div>
          <h2 className="text-heading-md text-ink">方案暂时读不出来</h2>
          <p className="mt-base-xs text-body-md text-ink-secondary">
            服务端没有回应,可能是后端未启动或网络中断。你的问卷与方案都存在服务端,不会因此丢失。
          </p>
        </div>
      </div>
      {/* 用 <a> 而非 <Link>:要的是「重新向服务端取一次数」,不是客户端路由跳转 */}
      <a href="/" className={cn(buttonVariants({ variant: "destructive" }), "mt-base-lg w-fit")}>
        <RefreshCw className="size-4" aria-hidden="true" />
        重试
      </a>
    </section>
  );
}

export function HomeView({ state }: { state: HomeState }) {
  if (state.kind === "plan") return <PlanSummary plan={state.plan} />;
  if (state.kind === "empty") return <EmptyGuide />;
  return <LoadError />;
}
