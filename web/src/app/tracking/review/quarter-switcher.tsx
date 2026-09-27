"use client";

/**
 * 季度切换器(H 期,产物 .q-switch 的落地)。
 *
 * 与产物的差异(T2 调整清单 #5):产物是前端循环样例,落地改为 **URL 查询参数驱动** ——
 * 点击只产生合法的 `?quarter=YYYY-QN` 跳转,数据真源在服务端(arch-h §1:切换器只产合法值)。
 * `aria-live` 沿产物:季度标签变化时播报。
 */
import { useRouter } from "next/navigation";
import { ChevronLeft, ChevronRight } from "lucide-react";

import { quarterLabel, REVIEW_COPY, stepQuarter } from "./state";

export function QuarterSwitcher({ quarter }: { quarter: string }) {
  const router = useRouter();

  return (
    <fieldset className="mb-base-md flex items-center gap-base-sm border-0 p-0">
      <legend className="sr-only">{REVIEW_COPY.switcherLabel}</legend>
      <button
        type="button"
        onClick={() => router.push(`/tracking/review?quarter=${stepQuarter(quarter, -1)}`)}
        className="inline-flex h-11 items-center gap-base-xs rounded-sm border border-hairline bg-canvas-soft px-base-md text-button-md text-ink hover:bg-canvas-soft/60"
      >
        <ChevronLeft className="size-4" aria-hidden="true" />
        上一季
      </button>
      <span
        role="status"
        aria-live="polite"
        className="flex-1 truncate rounded-sm border border-primary/30 bg-primary-soft px-base-md py-base-sm text-center text-body-md font-semibold text-primary"
      >
        {quarterLabel(quarter)}
      </span>
      <button
        type="button"
        onClick={() => router.push(`/tracking/review?quarter=${stepQuarter(quarter, 1)}`)}
        className="inline-flex h-11 items-center gap-base-xs rounded-sm border border-hairline bg-canvas-soft px-base-md text-button-md text-ink hover:bg-canvas-soft/60"
      >
        下一季
        <ChevronRight className="size-4" aria-hidden="true" />
      </button>
    </fieldset>
  );
}
