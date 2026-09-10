/**
 * P01 产品首页的双形态(design-v2 §1.1 / §3.1;形态与文案对照原型 design-v1.html 的 P01)。
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
import { Compass, RefreshCw, ShieldCheck } from "lucide-react";
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
 */
function emergencyLine(plan: PlanView): string {
  const coverage = formatMonthsTenths(plan.emergency.coverage_tenths);
  if (plan.emergency.is_met) return `应急金已覆盖(约 ${coverage} 个月)`;
  return `应急金当前约 ${coverage} 个月,还差约 ${plan.emergency.months_to_fill ?? 0} 个月达标`;
}

/** 有方案:摘要卡 + [查看完整方案][重新生成]。 */
function PlanSummary({ plan }: { plan: PlanView }) {
  return (
    <section className="flex flex-col gap-base-lg rounded-lg bg-card p-base-xl shadow-card">
      <div className="flex flex-wrap items-center gap-base-sm">
        <Badge>{plan.l1_mode_name}</Badge>
        <span className="text-aux text-text-aux">
          第 {plan.version} 版 · 生成于 {plan.created_date}
        </span>
      </div>

      <div className="flex flex-col gap-base-xs">
        <p className="text-aux text-text-aux">每月可投资</p>
        {/* 摘要里唯一的「大字」:用户扫一眼首页要拿到的就是这个数 */}
        <p className="text-page-title text-primary tabular-nums">
          {formatCurrency(plan.investable_monthly_cents)}
        </p>
      </div>

      <p className="flex items-start gap-base-sm rounded-md bg-bg-subtle px-base-lg py-base-md text-body text-text-body">
        <ShieldCheck className="mt-0.5 size-4 shrink-0 text-success" aria-hidden="true" />
        <span>{emergencyLine(plan)}</span>
      </p>

      <div className="flex flex-wrap gap-base-md">
        <Link href="/plan" className={buttonVariants({ size: "lg" })}>
          查看完整方案
        </Link>
        {/* 重新生成 = 回问卷(答案预填在服务端档案里),改完数字再生成即版本 +1 */}
        <Link href="/questionnaire" className={buttonVariants({ variant: "ghost", size: "lg" })}>
          <RefreshCw className="size-4" aria-hidden="true" />
          重新生成
        </Link>
      </div>
    </section>
  );
}

/** 空态三要素(EMPTY-001):图标 + 一句人话说明 + 明确的引导操作。 */
function EmptyGuide() {
  return (
    <section className="flex flex-col items-center gap-base-md rounded-lg bg-card p-base-xxl text-center shadow-card">
      <span className="grid size-14 place-items-center rounded-full bg-primary-bg text-primary">
        <Compass className="size-7" aria-hidden="true" />
      </span>
      <p className="text-body font-medium text-text-title">还没有方案</p>
      <p className="text-aux text-text-aux">完成 6 步问卷(约 2 分钟),得到你的第一份可照做的方案</p>
      <Link href="/questionnaire" className={cn(buttonVariants({ size: "lg" }), "mt-base-xs")}>
        开始问卷
      </Link>
    </section>
  );
}

/**
 * 取数失败(局部):三要素「发生了什么 + 为什么 + 怎么办」,且给一个真的能重试的动作。
 * 用整页刷新而非客户端 refetch:首页没有客户端状态,重试就是重新取一次数。
 */
function LoadError() {
  return (
    <section
      role="alert"
      className="flex flex-col gap-base-md rounded-lg bg-card p-base-xl shadow-card"
    >
      <div className="flex flex-col gap-base-xs">
        <p className="text-body font-medium text-text-title">方案暂时读不出来</p>
        <p className="text-aux text-text-body">
          服务端没有回应,可能是后端未启动或网络中断。你的问卷与方案都存在服务端,不会因此丢失。
        </p>
      </div>
      {/* 用 <a> 而非 <Link>:要的是「重新向服务端取一次数」,不是客户端路由跳转 */}
      <a href="/" className={cn(buttonVariants({ variant: "secondary" }), "w-fit")}>
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
