"use client";

/**
 * 快照录入卡(客户端岛;E 期票 04)。
 *
 * 视觉对齐固化产物 `docs/design/e-tracking/p05-tracking-github.html` 的段 1:
 * 桶数自适应(2 列栅格,3/4 桶同构)、「本月特殊」勾选行、通栏主按钮。
 * 两处产物没有、产品规则要求的功能扩展(见 .scratch/e-tracking/spec.md §4 清单):
 * 「跳过本月」(RULE-023 观测口)与覆盖确认(RECORD,ERR-E-03)。
 *
 * 金额以**元字符串**保存与提交,分秒不碰 float:预览格式化是纯字符串运算,
 * 与后端 `parse_yuan_to_cents` 的规则一致(两位小数、可负),但**权威校验在后端**。
 */
import { useRouter } from "next/navigation";
import { useState, useTransition } from "react";

import { submitSnapshotAction, skipMonthAction } from "./actions";
import { SNAPSHOT_NETWORK_ERROR } from "./state";

interface EntryCardProps {
  /** 当前自然月(YYYY-MM,服务端给) */
  month: string;
  /** 桶列表(id + 展示名),来自当前 active 方案 */
  buckets: { id: string; name: string }[];
  /** 当月已录时的预填值(元字符串;null = 首录) */
  initial: Record<string, string> | null;
  /** 当月是否已有快照(决定按钮文案与覆盖确认) */
  recorded: boolean;
}

/** 元输入的展示格式化(纯字符串,不走 float):「3200.5」→「¥3,200.5」。解析不了返回 null。 */
function formatYuanPreview(raw: string): string | null {
  const s = raw.trim();
  if (!s) return null;
  const m = /^(-?)(\d{1,10})(?:\.(\d{0,2}))?$/.exec(s);
  if (!m) return null;
  const [, sign, intPart, frac] = m;
  let grouped = "";
  for (let i = 0; i < intPart.length; i += 1) {
    if (i > 0 && (intPart.length - i) % 3 === 0) grouped += ",";
    grouped += intPart[i];
  }
  return `${sign}¥${grouped}${frac === undefined ? "" : `.${frac}`}`;
}

export function EntryCard({ month, buckets, initial, recorded }: EntryCardProps) {
  const router = useRouter();
  const [values, setValues] = useState<Record<string, string>>(() => {
    const seed: Record<string, string> = {};
    for (const b of buckets) seed[b.id] = initial?.[b.id] ?? "";
    return seed;
  });
  const [special, setSpecial] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [skipped, setSkipped] = useState(false);
  const [pending, startTransition] = useTransition();

  const allFilled = buckets.every((b) => formatYuanPreview(values[b.id] ?? "") !== null);

  function submit() {
    setError(null);
    setSuccess(false);
    if (!allFilled) {
      setError("每个账户都要填,金额最多两位小数");
      return;
    }
    // ERR-E-03:覆盖当月快照前确认(RECORD)
    if (recorded && !window.confirm("这个月已有记录,覆盖吗?")) return;

    startTransition(async () => {
      const result = await submitSnapshotAction(month, values, special);
      if (result.ok) {
        setSuccess(true); // 「本月已记录」确认条(role=status)
        router.refresh(); // 进度 / 偏离 / 历史走 summary 重取
      } else {
        setError(result.error);
      }
    });
  }

  function skip() {
    setSkipped(true);
    // 跳过月不落任何数据;上报失败也不提示(观测口,非主流程)
    startTransition(async () => {
      await skipMonthAction();
    });
  }

  return (
    <section className="flex flex-col rounded-sm border border-hairline bg-canvas-card">
      <div className="border-b border-hairline bg-canvas-soft px-base-lg py-base-sm text-caption font-semibold text-ink">
        本月快照 · {month}
      </div>
      <div className="flex flex-col gap-base-md p-base-lg">
        {success ? (
          <p
            role="status"
            className="flex items-center gap-base-sm rounded-sm bg-success-subtle px-base-lg py-base-md text-body-md text-success"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.6"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <circle cx="8" cy="8" r="6.2" />
              <path d="M5.2 8.2 7.2 10.2 10.8 6.2" />
            </svg>
            本月已记录
          </p>
        ) : null}

        {error ? (
          <div
            role="alert"
            className="flex items-start gap-base-md rounded-sm border border-danger-border bg-danger-subtle px-base-lg py-base-md"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 16 16"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
              className="mt-0.5 size-4 shrink-0 text-danger"
            >
              <circle cx="8" cy="8" r="6.2" />
              <path d="M8 5v3.6" />
              <circle cx="8" cy="11" r="0.9" fill="currentColor" stroke="none" />
            </svg>
            <div className="flex flex-col gap-base-xs">
              <p className="text-body-md font-semibold text-danger">本月没有记录成功</p>
              <p className="text-body-md text-ink-secondary">
                {error === SNAPSHOT_NETWORK_ERROR ? `${error}。你填的数字都还在,没有丢失。` : error}
              </p>
            </div>
          </div>
        ) : null}

        <div className="grid grid-cols-2 gap-base-sm">
          {buckets.map((b) => {
            const value = values[b.id] ?? "";
            const preview = formatYuanPreview(value);
            return (
              <div key={b.id} className="flex flex-col">
                <label className="text-body-md font-semibold text-ink" htmlFor={`bucket-${b.id}`}>
                  {b.name}
                </label>
                <input
                  id={`bucket-${b.id}`}
                  inputMode="decimal"
                  autoComplete="off"
                  className="mt-base-xs min-h-11 rounded-sm border border-hairline-input bg-canvas px-base-md font-mono text-body-tabular text-ink focus:border-primary focus:outline-none"
                  value={value}
                  onChange={(e) => {
                    setValues((v) => ({ ...v, [b.id]: e.target.value }));
                    setSuccess(false);
                  }}
                  placeholder="0.00"
                />
                <p
                  className="mt-base-xs overflow-hidden text-ellipsis whitespace-nowrap text-right font-mono text-caption text-ink-mute"
                  aria-live="polite"
                >
                  {preview === null ? "—" : preview}
                </p>
              </div>
            );
          })}
        </div>

        <label className="flex min-h-11 items-center gap-base-sm text-body-md font-semibold text-ink">
          <input
            type="checkbox"
            checked={special}
            onChange={(e) => setSpecial(e.target.checked)}
            className="size-4.5 accent-primary"
          />
          本月特殊
        </label>
        <p className="text-caption text-ink-mute">情况特殊?勾上这个月就不做偏离比较</p>

        <div className="mt-base-sm flex flex-col gap-base-sm">
          <button
            type="button"
            onClick={submit}
            disabled={pending}
            className="inline-flex min-h-11 w-full items-center justify-center gap-base-sm rounded-sm border border-btn-border bg-btn-primary px-base-lg text-button-md text-on-primary shadow-btn-primary hover:bg-btn-primary-hover disabled:opacity-60"
          >
            {recorded ? "覆盖本月" : "记录本月"}
          </button>
          {skipped ? (
            <p className="text-center text-caption text-ink-mute" role="status">
              本月已跳过,下月再来。已有的累计不会清零。
            </p>
          ) : (
            <button
              type="button"
              onClick={skip}
              disabled={pending}
              className="min-h-11 text-caption text-ink-mute underline-offset-2 hover:text-action hover:underline"
            >
              这个月先跳过
            </button>
          )}
        </div>
      </div>
    </section>
  );
}
