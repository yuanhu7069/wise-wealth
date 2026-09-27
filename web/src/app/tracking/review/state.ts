/**
 * P09 季度复盘页的类型与文案(H 期,REQ-20260927-01)。
 *
 * 类型取自 api-types 的 `components["schemas"]`(以实际 openapi.json 为准的手工移植,
 * ADR-G-004 立场);这里放的是**页面文案**与季度步进的纯函数 ——
 * 距离感表述的口径在 RULE-050,后端算好数字,前端不重算。
 */
import type { components } from "@/lib/api-types";

export type ReviewData = components["schemas"]["ReviewView"];

/** 季度标识 → 展示文案(2026-Q4 → 2026 年第 4 季度)。 */
export function quarterLabel(quarter: string): string {
  const [year, q] = quarter.split("-Q");
  return `${year} 年第 ${q} 季度`;
}

/** 季度步进(纯函数,只产合法值):2026-Q1 的上一季 = 2025-Q4?不 —— 年份下限 2026,
 * 到头就停在 2026-Q1(service 校验口径,阿司对称;下一季无上界,后端兜底)。 */
export function stepQuarter(quarter: string, delta: -1 | 1): string {
  const [yearStr, qStr] = quarter.split("-Q");
  let year = Number.parseInt(yearStr, 10);
  let q = Number.parseInt(qStr, 10);
  q += delta;
  if (q > 4) {
    q = 1;
    year += 1;
  }
  if (q < 1) {
    q = 4;
    year -= 1;
  }
  // 年份下限与后端一致:早于 2026 的季度会被 422,界面上就不产生这个值
  if (year < 2026) return "2026-Q1";
  return `${year}-Q${q}`;
}

/** 页面文案(唯一来源;组件里禁止内联这些句子)。 */
export const REVIEW_COPY = {
  title: "季度复盘",
  subtitle: "每季度回一次头,看坚持的痕迹",
  persistedEyebrow: "坚持月数",
  persistedUnit: "个月",
  persistedCaption: "已坚持 · 全部快照累计",
  deltaEyebrow: "各桶变化",
  specialNote: "本月特殊,未计入变化",
  gapEyebrow: "应急缺口收敛",
  gapStartToEnd: "季初缺口 → 当前缺口",
  gapAvg: "月均收敛额",
  forecastPrefix: "按当前速度约",
  forecastSuffix: "个月攒满",
  forecastNote: "按线性外推估算,不构成承诺",
  metText: "应急目标已攒满",
  singleOne: "本季仅 1 条快照,变化无从比较",
  singleMany: "本季可比较的快照不足 2 条,变化无从比较",
  noEmergency: "这一季没有可核算的应急口径数据",
  emptyTitle: "这一季还没有快照",
  emptyCta: "去录入本月快照",
  errorTitle: "复盘数据暂时读不出来",
  errorBody: "服务端没有回应,可能是网络中断。稍后重试即可,你的快照都在。",
  retry: "重试",
  invalidQuarterTitle: "这个季度格式不对",
  invalidQuarterBody: "链接里的季度参数不合法。回到当前季度即可查看复盘。",
  invalidQuarterCta: "回到当前季度",
  prevQuarter: "上一季",
  nextQuarter: "下一季",
  switcherLabel: "季度切换",
} as const;
