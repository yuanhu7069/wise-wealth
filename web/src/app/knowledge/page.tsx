/**
 * P07 知识库页(server 壳,G 期票 04;prd-g §8,REQ-20260925-02)。
 *
 * 职责:要求会话 → 取文章元数据 → 交给四板块视图。解读卡的徽章由
 * 列表响应自带(服务端从模式库富化,前端零富化);深链 `/knowledge?id=<id>`
 * 服务端预取全文、阅读态置顶(AC-12),未知 id → 404 文案不白屏。
 */
import { cookies } from "next/headers";

import { SiteFooterShell } from "@/components/site-footer-shell";
import { trackPageView } from "@/lib/analytics";
import { apiGet } from "@/lib/api";
import { requireSession, SESSION_COOKIE } from "@/lib/session";

import { KnowledgeView } from "./knowledge-view";
import type { KnowledgeArticle, KnowledgeData, KnowledgeState } from "./state";

export default async function KnowledgePage({
  searchParams,
}: {
  searchParams: Promise<{ id?: string }>;
}) {
  // RULE-001:受保护页面,未登录跳登录并在登录后回到本页
  await requireSession("/knowledge");

  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value ?? "";
  const cookie = `${SESSION_COOKIE}=${token}`;

  const params = await searchParams;
  const deepId = typeof params.id === "string" && params.id.trim() ? params.id : null;

  let state: KnowledgeState;
  try {
    const res = await apiGet<KnowledgeData>("/api/v1/knowledge", { cookie });
    if (res.status === 200 && res.envelope.success && res.envelope.data) {
      const data = res.envelope.data as KnowledgeData;
      state = data.items.length > 0 ? { kind: "ready", data } : { kind: "empty" };
    } else {
      state = { kind: "error" };
    }
  } catch {
    state = { kind: "error" };
  }

  // 埋点:内容页触达(G 期 §9.5,page_view p07)
  await trackPageView("p07");

  if (state.kind === "error") {
    return (
      <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
        <div className="flex flex-col items-center gap-base-md rounded-sm border border-danger-border bg-danger-subtle px-base-lg py-base-xxl text-center">
          <h1 className="text-heading-lg text-danger">知识库暂时读不出来</h1>
          <p className="text-body-md text-ink-secondary">
            服务端没有回应,可能是网络中断。稍后重试即可,你的问卷与方案都不受影响。
          </p>
          <a
            href="/knowledge"
            className="inline-flex min-h-11 items-center rounded-sm border border-hairline bg-canvas-soft px-base-lg text-button-md text-ink hover:bg-canvas-soft/60"
          >
            重试
          </a>
        </div>
      </main>
    );
  }

  if (state.kind === "empty") {
    // EMPTY-G-01:装载校验兜底(空目录即构建失败)使其实际不可达,纯防御
    return (
      <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
        <div className="flex flex-col items-center gap-base-md rounded-sm border border-hairline bg-canvas-card px-base-lg py-base-xxl text-center">
          <h1 className="text-display-md text-ink">内容暂时为空</h1>
          <p className="text-body-md text-ink-mute">暂时没有可读的内容,稍后再来看看。</p>
          <a
            href="/knowledge"
            className="inline-flex min-h-11 items-center rounded-sm border border-hairline bg-canvas-soft px-base-lg text-button-md text-ink hover:bg-canvas-soft/60"
          >
            重试
          </a>
        </div>
      </main>
    );
  }

  // 深链:预取全文(未知 id → 404 文案由视图承载,AC-12);失败不阻塞列表
  let deepArticle: { id: string; article: KnowledgeArticle } | null = null;
  let deepNotFound: string | null = null;
  if (deepId) {
    try {
      const res = await apiGet<KnowledgeArticle>(
        `/api/v1/knowledge/${encodeURIComponent(deepId)}`,
        { cookie },
      );
      if (res.status === 200 && res.envelope.success && res.envelope.data) {
        deepArticle = { id: deepId, article: res.envelope.data };
      } else if (res.status === 404) {
        deepNotFound = deepId;
      }
    } catch {
      deepNotFound = null;
    }
  }

  return (
    <>
      <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
        <KnowledgeView data={state.data} deepArticle={deepArticle} deepNotFound={deepNotFound} />
      </main>
      {/* P07 页脚沿 RULE-020/044:一行简述 + 完整声明展开 */}
      <SiteFooterShell />
    </>
  );
}
