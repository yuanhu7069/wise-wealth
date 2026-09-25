"use client";

/**
 * 知识库文章卡(客户端岛,G 期票 04)。视觉对齐固化产物
 * `docs/design/g-knowledge/p07-knowledge-github.html`:标题 + 徽章 + 导语 +
 * 「展开全文 / 收起全文」;展开后按 sections 结构渲染(小节标题 + 段落)。
 *
 * 全文经 Server Action 按需取(展开才取,板块首屏只背元数据);
 * 深链(/knowledge?id=)由页面服务端预取全文并作为 initial 传入。
 */
import { useState, useTransition } from "react";

import { getKnowledgeArticleAction } from "./actions";
import { CREDIBILITY_BADGE, type KnowledgeArticle, type KnowledgeItem } from "./state";

export function ArticleCard({
  item,
  /** 关联模式可信度(仅解读类;服务端从模式库富化后传入) */
  credibility,
  /** 深链预取的全文(id 命中时非空) */
  initialArticle,
}: {
  item: KnowledgeItem;
  credibility?: "verified" | "disputed" | "caution" | null;
  initialArticle?: KnowledgeArticle | null;
}) {
  const [expanded, setExpanded] = useState(initialArticle != null);
  const [article, setArticle] = useState<KnowledgeArticle | null>(initialArticle ?? null);
  const [error, setError] = useState<string | null>(null);
  const [pending, startTransition] = useTransition();

  // 解读/考据卡带可信度徽章;不可落地卡带灰色「仅供理解」警示徽章(RULE-040)
  const badge =
    item.kind === "non_implementable" ? (
      <span className="inline-flex items-center rounded-full border border-hairline bg-canvas-soft px-base-sm py-px text-micro-cap font-semibold text-ink-secondary">
        仅供理解,不建议照搬
      </span>
    ) : credibility ? (
      <span
        className={`inline-flex items-center rounded-full px-base-sm py-px text-micro-cap font-semibold ${CREDIBILITY_BADGE[credibility].cls}`}
      >
        {CREDIBILITY_BADGE[credibility].label}
      </span>
    ) : null;

  const toggle = () => {
    setError(null);
    if (expanded || article) {
      setExpanded(!expanded);
      return;
    }
    startTransition(async () => {
      const result = await getKnowledgeArticleAction(item.id);
      if (result.ok) {
        setArticle(result.article);
        setExpanded(true);
      } else {
        setError(result.notFound ? "文章不存在" : result.error);
      }
    });
  };

  return (
    <article className="flex min-w-0 flex-col rounded-sm border border-hairline bg-canvas-card p-base-lg">
      <div className="flex items-start justify-between gap-base-sm">
        <h3 className="min-w-0 truncate text-body-lg font-semibold text-ink" title={item.title}>
          {item.title}
        </h3>
        <div className="flex shrink-0 items-center gap-base-xs">{badge}</div>
      </div>
      <p className="mt-base-xs text-body-md text-ink-secondary">{item.summary}</p>

      <button
        type="button"
        aria-expanded={expanded}
        aria-controls={`kb-detail-${item.id}`}
        onClick={toggle}
        disabled={pending}
        className="mt-base-sm inline-flex min-h-11 items-center gap-base-xs self-start text-caption font-semibold text-ink-mute transition-colors hover:text-ink"
      >
        <svg
          aria-hidden="true"
          width="12"
          height="12"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
          strokeLinejoin="round"
          className={`transition-transform ${expanded ? "rotate-90" : ""}`}
        >
          <path d="M6 3.5 10.5 8 6 12.5" />
        </svg>
        {pending ? "取全文…" : expanded ? "收起全文" : "展开全文"}
      </button>

      <div
        id={`kb-detail-${item.id}`}
        hidden={!expanded}
        className="mt-base-md flex flex-col gap-base-lg border-t border-hairline pt-base-lg"
      >
        {error ? (
          <p role="status" className="text-body-md text-danger">
            {error}
          </p>
        ) : null}
        {article?.sections.map((s) => (
          <section key={s.heading} className="flex flex-col gap-base-sm">
            <h4 className="text-body-md font-semibold text-ink">{s.heading}</h4>
            {s.paragraphs.map((p, i) => (
              <p key={p.slice(0, 32)} className="text-body-md leading-relaxed text-ink-secondary">
                {p}
              </p>
            ))}
          </section>
        ))}
      </div>
    </article>
  );
}
