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

/** 可信度 wire 枚举(RULE-035;读取时解析,模式已下架 → null) */
export type Credibility = "verified" | "disputed" | "caution";

/**
 * 推理链节点(H 期 RULE-047):后端冻结值原样透传,前端不增删改、不编节点。
 * 内容字段仅在 plus 会话下发(ADR-H-001 服务侧闸门);free 态只有 trace_count。
 */
export interface PlanTrace {
  rule_id: string;
  /** 整数分(单位 cents)或月数(单位 months) */
  output: number;
  unit: "cents" | "months";
  rationale: string;
}

export interface PlanView {
  id: string;
  version: number;
  /** YYYY-MM-DD */
  created_date: string;
  l1_mode: string;
  l1_mode_name: string;
  /** 模式可信度(读取时解析,ADR-F-002;下架 → null,不渲染提示条) */
  l1_credibility?: Credibility | null;
  /** 模式出处(读取时解析;disputed 警示条文案组成) */
  l1_source?: string | null;
  /** 投资桶的每月转入(整数分):首页摘要的「每月可投资」 */
  investable_monthly_cents: number;
  buckets: PlanBucket[];
  l2: PlanL2;
  emergency: PlanEmergency;
  notices: PlanNotice[];
  /** 推理链条数(恒下发;free 态占位文案的「N 步」以此为准,RULE-045) */
  trace_count: number;
  /** 推理链全量。仅 plus 会话存在这个键(free = 服务侧未下发,非前端隐藏) */
  traces?: PlanTrace[];
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
 * 可信度提示条文案与变体(RULE-035):disputed = 警示(warn)、caution = 提示(info)、
 * verified = 不显示。评级描述模式的知识状态,文案是模式的属性而非单次生成的 ——
 * 与出处一并由读取时解析下发。禁止在组件里内联这些句子(同 NOTICE_COPY 立场)。
 */
export const CREDIBILITY_NOTICE: Record<
  Exclude<Credibility, "verified">,
  { variant: "warn" | "info"; title: string; description: string }
> = {
  disputed: {
    variant: "warn",
    title: "本方案使用的分账模式可信度存疑",
    description:
      "这个模式被广泛流传,但它的出处不成立 —— 请把它当作一种参考思路,而不是经过验证的方法。",
  },
  caution: {
    variant: "info",
    title: "本方案使用的分账模式需谨慎看待",
    description: "这个模式的来源真实,但存在应用边界,不一定适合所有人的情况。",
  },
};

/**
 * 推理链面板文案与规则标签(H 期)。唯一来源 = 本表,组件里禁止内联这些句子。
 * 未知 rule_id 的兜底标签 = 原样显示 id(引擎新增规则而前端未跟上时,不显示成空白)。
 */
export const TRACE_RULE_COPY: Record<string, string> = {
  emergency_fund_months: "应急月数",
  emergency_fund_target: "应急目标",
  emergency_fund_pacing: "每月补应急",
  safety_first_yield: "让位",
};

export const TRACE_COPY = {
  /** 收展入口(应急金状态区下方) */
  entry: "为什么是这个数",
  /** plus 态展开区的标题说明 */
  plusHint: "每个数字怎么来的,一步一步摊开",
  /** free 态占位:标题 / 示例句 / 结尾说明(规则名 = 通栏示例,不是你的数据) */
  freeTitle: "Plus 可展开完整推理链",
  freeExample: "示例:收入稳定性 = 波动大 → 应急月数取 9 个月",
  freeStepsSuffix: "步推理",
} as const;

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
