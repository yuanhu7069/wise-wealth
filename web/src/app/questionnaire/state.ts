/**
 * 问卷向导的共享类型与文案。
 *
 * 分别成文件的两个原因:`"use server"` 文件只能导出 async 函数;
 * 而选项文案要被 client 组件与页头共用,不适合塞进组件内部。
 */

export type Horizon = "within_1y" | "y1_3" | "y3_5" | "y5_10" | "over_10";
export type DrawdownResponse = "liquidate" | "reduce" | "hold" | "add";
export type IncomeStability = "stable" | "normal" | "volatile" | "freelance";
export type Goal = "house" | "education" | "retirement" | "wealth";

/** 向导里累积的答案(与后端 StepRequest 的字段一一对应)。 */
export interface Answers {
  horizon?: Horizon;
  drawdown_response?: DrawdownResponse;
  income_stability?: IncomeStability;
  has_social_security?: boolean;
  has_commercial_insurance?: boolean;
  mortgage_balance_cents?: number;
  dependents?: number;
  inflow_cents?: number;
  expense_fixed_monthly_cents?: number;
  savings_cents?: number;
  goal?: Goal;
}

/** 保存某一步的结果。 */
export interface SaveResult {
  ok: boolean;
  /** 失败时面向用户的文案 */
  error?: string;
  /** 成功时后端返回的下一步号 */
  draftStep?: number;
}

/** 答案步 1-5;步 6 是推荐(不落库,ticket 04 接入)。 */
export const LAST_ANSWER_STEP = 5;
export const TOTAL_STEPS = 6;

export const HORIZON_OPTIONS: ReadonlyArray<{ value: Horizon; label: string }> = [
  { value: "within_1y", label: "1 年内" },
  { value: "y1_3", label: "1-3 年" },
  { value: "y3_5", label: "3-5 年" },
  { value: "y5_10", label: "5-10 年" },
  { value: "over_10", label: "10 年以上" },
];

export const DRAWDOWN_OPTIONS: ReadonlyArray<{ value: DrawdownResponse; label: string }> = [
  { value: "liquidate", label: "清仓,先落袋" },
  { value: "reduce", label: "减仓观察" },
  { value: "hold", label: "不动,按计划执行" },
  { value: "add", label: "加仓,跌了更便宜" },
];

export const STABILITY_OPTIONS: ReadonlyArray<{ value: IncomeStability; label: string }> = [
  { value: "stable", label: "很稳" },
  { value: "normal", label: "一般" },
  { value: "volatile", label: "波动大" },
  { value: "freelance", label: "自由职业" },
];

export const GOAL_OPTIONS: ReadonlyArray<{ value: Goal; label: string }> = [
  { value: "house", label: "购房" },
  { value: "education", label: "子女教育" },
  { value: "retirement", label: "退休" },
  { value: "wealth", label: "财富增值" },
];

/** 各步的标题与副标题(文案唯一来源:prd-v1 §8.2 的问卷内容) */
export const STEP_COPY: ReadonlyArray<{ title: string; subtitle: string }> = [
  { title: "这笔钱几年后要用?", subtitle: "唯一直接决定这笔钱能不能进权益类的变量" },
  { title: "100 万跌到 70 万,你的第一反应是?", subtitle: "行为比自评更诚实" },
  { title: "收入稳定吗?", subtitle: "决定应急金要留几个月" },
  { title: "已有保障与负债?", subtitle: "房贷与保险本期仅记录,不影响方案数字" },
  { title: "你的基础财务数字", subtitle: "只采 3 个数字,不问流水" },
  { title: "为你推荐", subtitle: "任选其一,都能生成完整方案" },
];

/**
 * 「元」→「分」。界面收元、落库一律分(ADR-004 全链路整数分)。
 * 空串/非数字返回 null 由调用方决定是「缺失」还是「视为 0」。
 */
export function toCents(yuan: string): number | null {
  const trimmed = yuan.trim();
  if (!trimmed) return null;
  const value = Number(trimmed);
  if (!Number.isFinite(value)) return null;
  return Math.round(value * 100);
}
