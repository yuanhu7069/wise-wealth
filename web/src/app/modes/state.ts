/**
 * P06 模式库页类型(F 期票 04;prd-f §8,REQ-20260925-01)。
 *
 * 数据全部来自 GET /api/v1/modes 一次取回(ADR-F-004:详情展开零额外请求);
 * 桶份额口径 share_desc 由服务端字符串化,前端零解读(ADR-B-004 立场)。
 */
import type { components } from "@/lib/api-types";

/** GET /api/v1/modes 的响应(含主推与 L2 预览)。 */
export type ModesData = components["schemas"]["ModesView"];

/** 一张模式卡(含出处与桶概览)。 */
export type ModeCard = components["schemas"]["ModeCardView"];

/** 桶概览一行。 */
export type BucketOverview = components["schemas"]["BucketOverviewView"];

/** 可信度 wire 枚举。 */
export type Credibility = NonNullable<components["schemas"]["PlanView"]["l1_credibility"]>;

/** 可信度徽章三态展示(文字+色双通道;与 knowledge/state 同族,勿内联进组件) */
export const CREDIBILITY_BADGE: Record<
  "verified" | "disputed" | "caution",
  { label: string; cls: string }
> = {
  verified: { label: "已考证", cls: "bg-success-subtle text-success" },
  disputed: { label: "存疑", cls: "bg-attention-subtle text-warning" },
  caution: { label: "谨慎", cls: "bg-attention-subtle text-warning" },
};

/** 页面三态(与 tracking 同构):取数失败 / 防御空态 / 正常。 */
export type ModesState = { kind: "error" } | { kind: "empty" } | { kind: "ready"; data: ModesData };
