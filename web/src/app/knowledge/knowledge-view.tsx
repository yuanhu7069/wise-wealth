/**
 * P07 知识库页视图(server 组件,G 期票 04)。视觉对齐固化产物
 * `docs/design/g-knowledge/p07-knowledge-github.html`:四板块分区纵排。
 *
 * 板块序固定:模式解读 → 出处考据 → 理财百科 → 不可落地专区;
 * 百科用小卡网格(桌面 2 列),其余纵排。深链文章在页头下以阅读态置顶呈现(AC-12)。
 */
import Link from "next/link";

import { ArticleCard } from "./article-card";
import type { KnowledgeArticle, KnowledgeData } from "./state";

function SectionTag({ children }: { children: React.ReactNode }) {
  return (
    <p className="mb-base-xs flex items-center gap-base-sm text-caption text-ink-mute">
      {children}
      <span aria-hidden="true" className="h-px flex-1 bg-hairline" />
    </p>
  );
}

export function KnowledgeView({
  data,
  /** 深链命中的文章(服务端预取全文) */
  deepArticle,
  /** 深链未命中(未知 id → 404 文案,AC-12) */
  deepNotFound,
}: {
  data: KnowledgeData;
  deepArticle?: { id: string; article: KnowledgeArticle } | null;
  deepNotFound?: string | null;
}) {
  const items = data.items;
  const interp = items.filter((i) => i.kind === "mode_interpretation");
  const verif = items.filter((i) => i.kind === "verification");
  const enc = items.filter((i) => i.kind === "encyclopedia");
  const nonImpl = items.filter((i) => i.kind === "non_implementable");

  return (
    <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
      <header className="flex flex-col gap-base-xs">
        <h1 className="text-display-lg text-ink">知识库</h1>
        <p className="text-caption text-ink-mute">每种方法的来龙去脉,都摊开给你看</p>
      </header>

      {deepNotFound ? (
        <div
          role="status"
          className="mt-base-lg rounded-sm border border-danger-border bg-danger-subtle px-base-lg py-base-md"
        >
          <p className="text-body-md font-semibold text-danger">文章不存在</p>
          <p className="mt-base-xs text-caption text-ink-secondary">
            这个链接没有对应的内容,可以在下方四板块里找,或返回
            <Link href="/modes" className="text-primary underline">
              模式库
            </Link>
            。
          </p>
        </div>
      ) : null}

      <div className="mt-base-lg flex flex-col gap-base-xl">
        {/* 深链阅读态置顶(服务端预取全文) */}
        {deepArticle ? (
          <section className="flex flex-col">
            <SectionTag>正在阅读</SectionTag>
            <ArticleCard
              item={deepArticle.article}
              credibility={deepArticle.article.credibility ?? null}
              initialArticle={deepArticle.article}
            />
          </section>
        ) : null}

        <section className="flex flex-col">
          <SectionTag>模式解读</SectionTag>
          <div className="flex flex-col gap-base-md">
            {interp.map((item) => (
              <ArticleCard key={item.id} item={item} credibility={item.credibility ?? null} />
            ))}
          </div>
        </section>

        <section className="flex flex-col">
          <SectionTag>出处考据</SectionTag>
          <div className="flex flex-col gap-base-md">
            {verif.map((item) => (
              <ArticleCard key={item.id} item={item} />
            ))}
          </div>
        </section>

        <section className="flex flex-col">
          <SectionTag>理财百科</SectionTag>
          <div className="grid gap-base-md sm:grid-cols-2">
            {enc.map((item) => (
              <ArticleCard key={item.id} item={item} />
            ))}
          </div>
        </section>

        <section className="flex flex-col">
          <SectionTag>不可落地专区</SectionTag>
          <div className="flex flex-col gap-base-md">
            {nonImpl.map((item) => (
              <ArticleCard key={item.id} item={item} />
            ))}
          </div>
        </section>
      </div>
    </main>
  );
}
