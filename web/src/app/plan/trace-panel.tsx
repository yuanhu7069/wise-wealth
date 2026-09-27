"use client";

/**
 * 推理链面板(H 期 FEATURE-004,「Plus 不是独立页面,而是每个界面的展开能力」)。
 *
 * 数据随方案响应一次性下发(ADR-H-004):展开/收起是纯前端收展,零额外请求。
 * 双态由 wire 形状决定 —— `traces` 键存在 = plus(全链渲染);
 * 不存在 = free(服务侧没下发,RULE-045;占位只有条数与通栏示例,无任何解锁动作,
 * 本期无购买路径 OUT-001)。节点内容原样透传,不增删改(RULE-047)。
 *
 * 挂载点:一、分配总览卡的应急金状态区下方 —— 链条解释的正是那一区几个数字。
 */
import { ChevronDown, Plus } from "lucide-react";
import { useId, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { formatCurrency } from "@/lib/format-currency";
import { cn } from "@/lib/utils";

import { TRACE_COPY, TRACE_RULE_COPY, type PlanTrace } from "./state";

/** 输出值按单位渲染:金额走全站唯一格式化,月数加单位(等宽,数值红线)。 */
function TraceOutput({ trace }: { trace: PlanTrace }) {
  return (
    <span className="font-mono tabular-nums">
      {trace.unit === "cents" ? formatCurrency(trace.output) : `${trace.output} 个月`}
    </span>
  );
}

export function TracePanel({
  traceCount,
  traces,
}: {
  traceCount: number;
  /** plus 会话才有这个键;undefined = free(服务侧闸门,前端不做第二判断) */
  traces?: PlanTrace[];
}) {
  const [expanded, setExpanded] = useState(false);
  const panelId = useId();

  return (
    <div className="flex flex-col gap-base-sm">
      <button
        type="button"
        aria-expanded={expanded}
        aria-controls={panelId}
        onClick={() => setExpanded((v) => !v)}
        className="flex h-11 items-center gap-base-sm self-start rounded-sm px-base-sm text-body-md text-primary hover:bg-primary-soft focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
      >
        <ChevronDown
          aria-hidden="true"
          className={cn("size-4 transition-transform", expanded && "rotate-180")}
        />
        {TRACE_COPY.entry}
      </button>

      <div id={panelId} hidden={!expanded} className="flex flex-col gap-base-sm">
        {traces ? (
          <>
            <p className="text-caption text-ink-mute">{TRACE_COPY.plusHint}</p>
            <ol className="flex flex-col">
              {traces.map((trace, i) => (
                <li
                  key={trace.rule_id}
                  className={cn(
                    "relative flex flex-col gap-0.5 pb-base-md pl-base-lg",
                    i < traces.length - 1 &&
                      "before:absolute before:top-5 before:left-2 before:h-full before:border-l before:border-hairline",
                  )}
                >
                  {/* 圆点中心 (8,20) 与连接线(left-2/top-5)严格同心 */}
                  <span
                    aria-hidden="true"
                    className="absolute top-3 left-0 size-4 rounded-full border-2 border-primary bg-canvas-card"
                  />
                  <p className="text-caption font-semibold text-ink-mute">
                    {TRACE_RULE_COPY[trace.rule_id] ?? trace.rule_id}
                  </p>
                  <p className="text-body-md text-ink">
                    {trace.rationale}
                    {" → "}
                    <TraceOutput trace={trace} />
                  </p>
                </li>
              ))}
            </ol>
          </>
        ) : (
          <>
            {/* 占位:链条形状可见、内容不可读(aria-hidden,不误导读屏);示例句可读 */}
            <div aria-hidden="true" className="flex select-none flex-col gap-base-sm">
              {[0, 1, 2].map((i) => (
                <div
                  key={i}
                  className="flex items-center gap-base-sm blur-sm"
                  style={{ opacity: 0.9 - i * 0.15 }}
                >
                  <span className="size-3.5 shrink-0 rounded-full border-2 border-hairline bg-canvas-soft" />
                  <span
                    className="h-3 rounded-full bg-canvas-soft"
                    style={{ width: `${72 - i * 14}%` }}
                  />
                </div>
              ))}
            </div>
            <p className="flex items-center gap-base-sm text-body-md text-ink">
              <Badge>
                <Plus className="size-3.5" aria-hidden="true" />
                Plus
              </Badge>
              {TRACE_COPY.freeTitle}
              <span className="text-caption text-ink-mute">
                ({traceCount} {TRACE_COPY.freeStepsSuffix})
              </span>
            </p>
            <p className="text-caption text-ink-mute">{TRACE_COPY.freeExample}</p>
          </>
        )}
      </div>
    </div>
  );
}
