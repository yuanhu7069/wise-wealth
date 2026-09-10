/**
 * 方案页(P04)的共享类型与文案。
 *
 * 类型与后端 `dto/plan.rs` 一一对应:字段名即 wire 名,不做驼峰转换 ——
 * 转换层会掩盖「后端改了字段名而前端没跟上」这类缺陷。
 *
 * 这里放的是**页面文案**(提示条的三要素说法),不放金额格式:后者属 lib/format-currency.ts,
 * 全站唯一一份。
 */

/** 一个桶(账户)的每月转入。 */
export interface PlanBucket {
  bucket_id: string;
  name: string;
  purpose: string;
  /** 整数分(ADR-004) */
  amount_monthly_cents: number;
  /** 目标金额(分),仅规则桶未达标时有值 */
  target_cents?: number | null;
}

/** 投资桶里的一个资产大类(只到大类,不出现具体产品) */
export interface L2Class {
  name: string;
  /** 万分比(6000 = 60%) */
  basis_points: number;
}

/** 投资账户内部配置(L2 快照) */
export interface PlanL2 {
  id: string;
  name: string;
  classes: L2Class[];
  note?: string | null;
  reason: string;
}

/** 应急金状态(RULE-009 ~ RULE-013) */
export interface PlanEmergency {
  /** 应急目标月数 */
  months: number;
  necessary_monthly_cents: number;
  target_cents: number;
  gap_cents: number;
  monthly_toward_emergency_cents: number;
  /** 按当前节奏还差几个月;已达标为 null */
  months_to_fill?: number | null;
  /** 当前覆盖月数 × 10(30 = 约 3.0 个月) */
  coverage_tenths: number;
  /** 超出应急目标的部分(分):投资账户的「已有家底」 */
  surplus_cents: number;
  is_met: boolean;
}

/** 引擎提示(RULE-014 等)。UI 按此渲染提示条,文案在 NOTICE_COPY。 */
export type PlanNotice =
  | "insufficient_income"
  | "fixed_exceeds_necessary"
  | "short_horizon_cash_only";

export interface PlanView {
  id: string;
  version: number;
  /** YYYY-MM-DD */
  created_date: string;
  l1_mode: string;
  l1_mode_name: string;
  /** 投资桶的每月转入(整数分):首页摘要的「每月可投资」 */
  investable_monthly_cents: number;
  buckets: PlanBucket[];
  l2: PlanL2;
  emergency: PlanEmergency;
  notices: PlanNotice[];
}

/**
 * 提示条文案(RULE-008 三要素:发生了什么 + 为什么 + 怎么办)。
 * 后端只给事实(notice 枚举),文案的唯一来源是本表 —— 禁止在组件里内联这些句子。
 */
export const NOTICE_COPY: Record<PlanNotice, { title: string; description: string }> = {
  insufficient_income: {
    title: "收入不足以覆盖当前结构",
    description:
      "月固定支出与必要比例已经用满整月收入,投资账户与备用账户均为 ¥0.00。先提高收入或压缩固定支出,再重新生成方案。",
  },
  fixed_exceeds_necessary: {
    title: "固定支出超过必要开支额度",
    description:
      "本模式的必要账户按收入比例计算,你的固定支出已经超过它 —— 超出的部分会挤占其余账户的份额。",
  },
  short_horizon_cash_only: {
    title: "这笔钱 1 年内要用",
    description: "方案不配置权益类资产,投资部分全部按现金类安排,优先保住本金与流动性。",
  },
};

/**
 * 分段比例条的色块类名(基线 §6.3:分类色**按固定顺序取**,禁止随机生成)。
 *
 * 写成静态字面量数组而非 `bg-chart-${i}` 拼接:Tailwind 只扫描源码里的完整类名,
 * 拼接出来的类名不会被生成 —— 颜色会静默丢失。
 */
export const CHART_BG_CLASSES = [
  "bg-chart-1",
  "bg-chart-2",
  "bg-chart-3",
  "bg-chart-4",
  "bg-chart-5",
  "bg-chart-6",
] as const;

/** 大类数 > 6 时归为「其他」(基线 §6.3「单图分类不超过 6 个」)。 */
export function chartBgClass(index: number): string {
  if (index < CHART_BG_CLASSES.length) return CHART_BG_CLASSES[index];
  return CHART_BG_CLASSES[CHART_BG_CLASSES.length - 1];
}
