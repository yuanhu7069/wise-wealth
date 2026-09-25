/**
 * P07 知识库页类型(G 期票 04;prd-g §8,REQ-20260925-02)。
 *
 * 数据来自 GET /api/v1/knowledge(元数据)与 GET /api/v1/knowledge/{id}(全文);
 * 内容为服务端内嵌配置(ADR-G-001),前端按 sections 结构渲染,零 markdown 依赖。
 */
import type { components } from "@/lib/api-types";

/** GET /api/v1/knowledge 的响应。 */
export type KnowledgeData = components["schemas"]["KnowledgeListView"];

/** 文章元数据。 */
export type KnowledgeItem = components["schemas"]["KnowledgeListItemView"];

/** 文章全文。 */
export type KnowledgeArticle = components["schemas"]["KnowledgeArticleView"];

/** 文章板块(四枚举的字符串面)。 */
export type KnowledgeKind = components["schemas"]["KnowledgeKind"];

/** 可信度徽章三态的展示配置(文字 + 颜色双通道,prd-f §9.3 立场)。 */
export const CREDIBILITY_BADGE: Record<
  "verified" | "disputed" | "caution",
  { label: string; cls: string }
> = {
  verified: { label: "已考证", cls: "bg-success-subtle text-success" },
  disputed: { label: "存疑", cls: "bg-attention-subtle text-warning" },
  caution: { label: "谨慎", cls: "bg-attention-subtle text-warning" },
};

/** 页面状态:取数失败 / 防御空态 / 正常(含可选的深链阅读态)。 */
export type KnowledgeState =
  | { kind: "error" }
  | { kind: "empty" }
  | { kind: "ready"; data: KnowledgeData };
