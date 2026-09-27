/**
 * P09 季度复盘页(server 壳,票 04)。
 *
 * 职责:要求会话 → 解析季度(缺省当前季,后端权威)→ 单请求取复盘聚合 → 视图。
 * 三种页面形态:正常三卡 / 空季与单点(数据里带 count 与 comparable,视图层给文案)/
 * 错误(非法 quarter 422 给专用文案;取数失败给错误条 + 重试,均不白屏)。
 * 复盘免费全量(RULE-051):页面与端点都不读分层。
 */
import { cookies } from "next/headers";
import Link from "next/link";

import { SiteFooterShell } from "@/components/site-footer-shell";
import { trackPageView } from "@/lib/analytics";
import { apiGet } from "@/lib/api";
import { requireSession, SESSION_COOKIE } from "@/lib/session";

import { ReviewView } from "./review-view";
import { REVIEW_COPY, type ReviewData } from "./state";

/** 非法季度直链的错误形态(ERR-H-03):可理解的文案 + 回当前季,不白屏。 */
function InvalidQuarter() {
  return (
    <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
      <div className="flex flex-col items-center gap-base-md rounded-sm border border-hairline bg-canvas-card px-base-lg py-base-xxl text-center">
        <h1 className="text-display-md text-ink">{REVIEW_COPY.invalidQuarterTitle}</h1>
        <p className="text-body-md text-ink-mute">{REVIEW_COPY.invalidQuarterBody}</p>
        <Link
          href="/tracking/review"
          className="inline-flex min-h-11 items-center rounded-sm bg-primary px-base-lg text-button-md font-semibold text-on-primary"
        >
          {REVIEW_COPY.invalidQuarterCta}
        </Link>
      </div>
    </main>
  );
}

export default async function ReviewPage({
  searchParams,
}: {
  searchParams: Promise<{ quarter?: string }>;
}) {
  // RULE-001:受保护页面,未登录跳登录并在登录后回到本页
  await requireSession("/tracking/review");

  const params = await searchParams;
  const quarterParam = params.quarter;

  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value ?? "";
  const cookie = `${SESSION_COOKIE}=${token}`;

  const query = quarterParam ? `?quarter=${encodeURIComponent(quarterParam)}` : "";
  let data: ReviewData | null = null;
  let invalidQuarter = false;
  try {
    const res = await apiGet<ReviewData>(`/api/v1/tracking/review${query}`, { cookie });
    if (res.status === 200 && res.envelope.success && res.envelope.data) {
      data = res.envelope.data;
    } else if (res.status === 422) {
      invalidQuarter = true;
    }
  } catch {
    data = null;
  }

  // 埋点:空态/单点/错误形态用户也都真实看到了本页
  await trackPageView("p09");

  if (invalidQuarter) {
    return <InvalidQuarter />;
  }

  if (!data) {
    return (
      <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
        <div className="flex flex-col items-center gap-base-md rounded-sm border border-danger-border bg-danger-subtle px-base-lg py-base-xxl text-center">
          <h1 className="text-heading-lg text-danger">{REVIEW_COPY.errorTitle}</h1>
          <p className="text-body-md text-ink-secondary">{REVIEW_COPY.errorBody}</p>
          <a
            href="/tracking/review"
            className="inline-flex min-h-11 items-center rounded-sm border border-hairline bg-canvas-soft px-base-lg text-button-md text-ink hover:bg-canvas-soft/60"
          >
            {REVIEW_COPY.retry}
          </a>
        </div>
      </main>
    );
  }

  return (
    <>
      {/* 三卡桌面横排需要更宽容器(产物 60rem;T2 调整清单 #1),其余页保持 36rem 节奏 */}
      <main className="mx-auto w-full max-w-[60rem] flex-1 px-base-lg py-base-xxl">
        <ReviewView data={data} />
      </main>
      {/* 页脚沿 RULE-020/031:一行简述 + 完整声明展开(复盘含金额与外推,声明必须同页) */}
      <SiteFooterShell />
    </>
  );
}
