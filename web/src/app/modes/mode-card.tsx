"use client";

/**
 * 模式卡(客户端岛,F 期票 04)。视觉对齐固化产物
 * `docs/design/f-modes/p06-modes-github.html`:名称+徽章行 / tagline / 人群 pill /
 * 「展开详情」文字按钮 / 出处块 + 桶概览 + 主按钮 CTA。
 *
 * 交互两件:收起 ↔ 展开(aria-expanded + hidden 面板,产物同款)、
 * CTA 生成中禁重复点击(RULE-034:生成语义与问卷同路,entry=mode_lib 仅埋点口径)。
 * CTA 失败以卡内错误条呈现(ERR-F-01,调整清单 #1:产物为示意跳转,无失败态样本)。
 */
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState, useTransition } from "react";

import { generatePlanAction } from "@/app/plan/actions";
import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

import type { Credibility, ModeCard as ModeCardData } from "./state";

/** 可信度徽章:文字 + 颜色双通道(prd-f §9.3,不只靠色)。谨慎与存疑同用黄色族(产物未给谨慎样例,调整清单 #2)。 */
const CREDIBILITY_BADGE: Record<Credibility, { label: string; cls: string }> = {
  verified: { label: "已考证", cls: "bg-success-subtle text-success" },
  disputed: { label: "存疑", cls: "bg-attention-subtle text-warning" },
  caution: { label: "谨慎", cls: "bg-attention-subtle text-warning" },
};

export function ModeCard({
  card,
  compareSelected = false,
  onToggleCompare,
}: {
  card: ModeCardData;
  /** 是否已被勾选进对比(G 期票 06);未传入时隐藏勾选入口 */
  compareSelected?: boolean;
  /** 勾选切换回调(由 ModesBrowser 注入;未传 = 不显示勾选) */
  onToggleCompare?: (id: string) => void;
}) {
  const router = useRouter();
  // 主推卡默认展开(产物语义:推荐卡收起态无展示必要;调整清单 #3)
  const [expanded, setExpanded] = useState(card.is_recommended);
  const [error, setError] = useState<{ text: string; expired: boolean } | null>(null);
  const [pending, startTransition] = useTransition();

  const badge = CREDIBILITY_BADGE[card.credibility];

  const generate = () => {
    setError(null);
    startTransition(async () => {
      const result = await generatePlanAction(card.id, "mode_lib");
      if (result.ok) {
        router.push("/plan");
        return;
      }
      setError({ text: result.error, expired: result.expired === true });
    });
  };

  return (
    <article className="flex min-w-0 flex-col rounded-sm border border-hairline bg-canvas-card p-base-lg">
      <div className="flex items-start justify-between gap-base-sm">
        <h2 className="min-w-0 truncate text-body-lg font-semibold text-ink" title={card.name}>
          {card.name}
        </h2>
        <div className="flex flex-none flex-wrap items-center gap-base-xs">
          {card.is_recommended ? (
            <span className="rounded-full bg-primary px-base-sm py-px text-micro-cap font-semibold text-on-primary">
              推荐
            </span>
          ) : null}
          <span
            className={cn("rounded-full px-base-sm py-px text-micro-cap font-semibold", badge.cls)}
          >
            {badge.label}
          </span>
        </div>
      </div>

      {card.tagline ? <p className="mt-base-xs text-body-md text-ink">{card.tagline}</p> : null}

      {card.fit_for.length > 0 ? (
        <div className="mt-base-md flex flex-wrap gap-base-xs">
          {card.fit_for.map((f) => (
            <span
              key={f}
              className="rounded-full bg-canvas-soft px-base-sm py-px text-micro-cap text-ink-secondary"
            >
              {f}
            </span>
          ))}
        </div>
      ) : null}

      <button
        type="button"
        aria-expanded={expanded}
        aria-controls={`mode-detail-${card.id}`}
        onClick={() => setExpanded((v) => !v)}
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
          className={cn("transition-transform", expanded && "rotate-90")}
        >
          <path d="M6 3.5 10.5 8 6 12.5" />
        </svg>
        {expanded ? "收起详情" : "展开详情"}
      </button>

      {onToggleCompare ? (
        <div className="mt-base-sm flex flex-wrap items-center gap-base-lg">
          <label className="inline-flex min-h-11 cursor-pointer items-center gap-base-sm text-caption text-ink-secondary">
            <input
              type="checkbox"
              checked={compareSelected}
              onChange={() => onToggleCompare(card.id)}
              className="size-4 accent-[var(--primary)]"
            />
            加入对比
          </label>
          <Link
            href={`/knowledge?id=ki-${card.id}`}
            className="inline-flex min-h-11 items-center text-caption font-semibold text-primary underline"
          >
            阅读完整解读
          </Link>
        </div>
      ) : null}

      <div
        id={`mode-detail-${card.id}`}
        hidden={!expanded}
        className="mt-base-md flex flex-col gap-base-lg border-t border-hairline pt-base-lg"
      >
        {card.source ? (
          <div className="rounded-sm border border-hairline bg-canvas-soft px-base-lg py-base-md">
            <p className="mb-base-xs text-caption font-semibold text-ink-mute">出处</p>
            <p className="text-body-md [overflow-wrap:anywhere]">{card.source}</p>
          </div>
        ) : null}

        <div>
          <p className="mb-base-xs text-caption font-semibold text-ink-mute">桶概览</p>
          <div className="border-t border-hairline">
            {card.buckets.map((b) => (
              <div key={b.name} className="border-b border-hairline py-base-sm last:border-b-0">
                <div className="flex items-baseline justify-between gap-base-md">
                  <span className="text-body-md font-semibold text-ink">{b.name}</span>
                  <span className="whitespace-nowrap text-right text-caption text-ink-mute">
                    {b.share_desc}
                  </span>
                </div>
                {b.purpose ? <p className="text-caption text-ink-mute">{b.purpose}</p> : null}
              </div>
            ))}
          </div>
        </div>

        <button
          type="button"
          onClick={generate}
          disabled={pending}
          aria-busy={pending}
          className={cn(buttonVariants({ variant: "default" }), "w-full")}
        >
          {pending ? (
            <>
              <svg
                aria-hidden="true"
                width="16"
                height="16"
                viewBox="0 0 16 16"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.8"
                strokeLinecap="round"
                className="animate-spin motion-reduce:animate-none"
              >
                <path d="M8 1.5a6.5 6.5 0 1 1-4.6 1.9" />
              </svg>
              生成中…
            </>
          ) : (
            "用此模式生成方案"
          )}
        </button>

        {error ? (
          <div
            role="status"
            className="flex items-start gap-base-md rounded-sm border border-danger-border bg-danger-subtle px-base-lg py-base-md"
          >
            <svg
              aria-hidden="true"
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="mt-0.5 size-4 shrink-0 text-danger"
            >
              <circle cx="8" cy="8" r="6.2" />
              <path d="M8 5v3.6" />
              <circle cx="8" cy="11" r="0.9" fill="currentColor" stroke="none" />
            </svg>
            <p className="text-body-md text-ink-secondary">
              {error.text}
              {error.expired ? (
                <>
                  {" "}
                  <a href="/login?from=/modes" className="text-primary underline">
                    重新登录
                  </a>
                </>
              ) : (
                <>
                  {" "}
                  <button
                    type="button"
                    onClick={generate}
                    className="font-semibold text-primary underline"
                  >
                    再试一次
                  </button>
                </>
              )}
            </p>
          </div>
        ) : null}
      </div>
    </article>
  );
}
