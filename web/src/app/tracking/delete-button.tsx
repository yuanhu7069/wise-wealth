"use client";

/**
 * 删除最新月快照(RULE-029 录错恢复口;ERR-E-04 二次确认)。
 * 产物没有此控件 —— 产品规则要求的功能扩展,见 .scratch/e-tracking/spec.md §4 清单 #2。
 * 只出现在历史列表的第一行(最新月);删除后 router.refresh() 重取列表与摘要。
 */
import { useRouter } from "next/navigation";
import { useState, useTransition } from "react";

import { deleteSnapshotAction } from "./actions";

export function DeleteButton({ month }: { month: string }) {
  const router = useRouter();
  const [error, setError] = useState<string | null>(null);
  const [pending, startTransition] = useTransition();

  return (
    <span className="inline-flex flex-col items-end">
      <button
        type="button"
        disabled={pending}
        className="min-h-11 text-caption text-ink-mute underline-offset-2 hover:text-danger hover:underline sm:min-h-10 disabled:opacity-60"
        onClick={() => {
          setError(null);
          // ERR-E-04:删除当月快照需二次确认
          if (!window.confirm(`删除 ${month} 的快照?删除后这个月将算作跳过。`)) return;
          startTransition(async () => {
            const result = await deleteSnapshotAction(month);
            if (result.ok) router.refresh();
            else setError(result.error);
          });
        }}
      >
        删除
      </button>
      {error ? (
        <span role="alert" className="text-caption text-danger">
          {error}
        </span>
      ) : null}
    </span>
  );
}
