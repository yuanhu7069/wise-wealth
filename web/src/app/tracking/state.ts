/**
 * P05 追踪页的类型与文案(E 期;口径:prd-e §6/§8,视图零计算 —— ADR-E-002)。
 *
 * 全部业务数字来自 `GET /api/v1/snapshots` 的 summary(后端 tracking 纯函数算好),
 * 本页只做「数字 → 中文」的格式化;桶名来自当前 active 方案,历史行里的未知桶 id
 * 按原样展示(RULE-028:历史快照按各自版本的桶集冻结)。
 */
import type { components } from "@/lib/api-types";

/** 一条历史快照(生成类型,不手写形状 —— RULE-005)。 */
export type SnapshotItem = components["schemas"]["SnapshotView"];

/** 追踪摘要(已坚持月数 / 应急金结论 / 最新偏离)。 */
export type TrackingSummaryView = components["schemas"]["TrackingSummaryView"];

/** GET /snapshots 的整体响应。 */
export interface SnapshotsData {
  items: SnapshotItem[];
  summary: TrackingSummaryView;
}

/** 页面级取数结果:三态在 server 壳里定型,视图只认这三种。 */
export type TrackingState =
  | { kind: "ready"; data: SnapshotsData }
  | { kind: "no-plan" }
  | { kind: "error" };

/** ERR-E-01(录入提交失败)文案:prd-e §8.4,数据不能丢。 */
export const SNAPSHOT_NETWORK_ERROR = "网络不太稳,再试一次";
/** 会话过期文案(与后端 401 信封同源,action 里 401 时使用)。 */
export const SNAPSHOT_EXPIRED_ERROR = "登录已过期,请重新登录";
