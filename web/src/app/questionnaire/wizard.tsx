"use client";

/**
 * 问卷向导(P03)。一屏一题,每步自动保存,中断可续。
 *
 * 步 1-5 是作答,步 6 是推荐位(ticket 04 接入)——本工单先把作答链路做通,
 * 完成态如实告诉用户下一步会是什么,不用占位内容假装已实现。
 */
import Link from "next/link";
import { useState, useTransition } from "react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

import { saveStepAction } from "./actions";
import {
  type Answers,
  DRAWDOWN_OPTIONS,
  GOAL_OPTIONS,
  HORIZON_OPTIONS,
  LAST_ANSWER_STEP,
  STABILITY_OPTIONS,
  STEP_COPY,
  TOTAL_STEPS,
  type DrawdownResponse,
  type Goal,
  type Horizon,
  type IncomeStability,
  toCents,
} from "./state";

interface WizardProps {
  /** 断点恢复:上次答到的步号 */
  initialStep: number;
  /** 预填:已保存的答案(重新生成时进来能只改变化的数字) */
  initialAnswers: Answers;
}

/** 单选项卡片。选中态同时有颜色与勾图标,不单靠颜色传达(design 基线 §10.2)。 */
function Choice<T extends string>({
  options,
  value,
  onSelect,
}: {
  options: ReadonlyArray<{ value: T; label: string }>;
  value: T | undefined;
  onSelect: (v: T) => void;
}) {
  return (
    <div className="flex flex-col gap-base-sm">
      {options.map((opt) => {
        const selected = value === opt.value;
        return (
          <button
            key={opt.value}
            type="button"
            onClick={() => onSelect(opt.value)}
            aria-pressed={selected}
            className={cn(
              "flex min-h-11 items-center gap-base-md rounded-md border px-base-lg py-base-md text-left text-body",
              selected
                ? "border-2 border-primary bg-primary-bg font-medium text-text-title"
                : "border-divider bg-bg-card text-text-body hover:border-text-aux",
            )}
          >
            <span
              aria-hidden="true"
              className={cn(
                "size-4 shrink-0 rounded-full border-2",
                selected ? "border-primary bg-primary" : "border-divider",
              )}
            />
            {opt.label}
          </button>
        );
      })}
    </div>
  );
}

function NumberField({
  id,
  label,
  hint,
  value,
  onChange,
  required,
}: {
  id: string;
  label: string;
  hint?: string;
  value: string;
  onChange: (v: string) => void;
  required?: boolean;
}) {
  return (
    <div className="flex flex-col gap-base-xs">
      <label htmlFor={id} className="text-label font-medium text-text-title">
        {label} {required ? <span className="text-danger">*</span> : null}
      </label>
      <input
        id={id}
        inputMode="decimal"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="h-10 rounded-sm border border-divider bg-bg-card px-base-md text-body text-text-body focus:border-primary focus:outline-none"
      />
      {hint ? <p className="text-label text-text-aux">{hint}</p> : null}
    </div>
  );
}

/** 步号序列。用步号本身作 key 而非数组下标 —— 分段的位置即它的身份。 */
const STEP_NUMBERS = Array.from({ length: TOTAL_STEPS }, (_, i) => i + 1);

export function Wizard({ initialStep, initialAnswers }: WizardProps) {
  const [step, setStep] = useState(Math.min(initialStep, LAST_ANSWER_STEP));
  const [answers, setAnswers] = useState<Answers>(initialAnswers);
  // 金额输入以「元」为单位保留字符串,提交时才转分 —— 避免把「12000.」这类中间态吃掉
  const [yuan, setYuan] = useState({
    mortgage: initialAnswers.mortgage_balance_cents
      ? String(initialAnswers.mortgage_balance_cents / 100)
      : "",
    inflow: initialAnswers.inflow_cents ? String(initialAnswers.inflow_cents / 100) : "",
    fixed: initialAnswers.expense_fixed_monthly_cents
      ? String(initialAnswers.expense_fixed_monthly_cents / 100)
      : "",
    savings: initialAnswers.savings_cents ? String(initialAnswers.savings_cents / 100) : "",
    dependents: initialAnswers.dependents !== undefined ? String(initialAnswers.dependents) : "0",
  });
  const [error, setError] = useState("");
  const [done, setDone] = useState(false);
  const [pending, startTransition] = useTransition();

  const patch = (p: Partial<Answers>) => setAnswers((prev) => ({ ...prev, ...p }));

  /**
   * 本步要提交的答案。
   *
   * 以**全部已答项**为基础再补本步的数字字段 —— 只挑本步字段会漏掉单选答案
   * (步 1-3 的答案存在 answers 里,不带上就等于提交了空对象)。
   * 后端按步校验,与本步无关的字段会被忽略。
   */
  function stepAnswers(current: number): Answers {
    const payload: Answers = { ...answers };
    if (current === 4) {
      payload.dependents = Number(yuan.dependents) || 0;
      payload.has_social_security = answers.has_social_security ?? false;
      payload.has_commercial_insurance = answers.has_commercial_insurance ?? false;
      payload.mortgage_balance_cents = toCents(yuan.mortgage) ?? undefined;
    }
    if (current === 5) {
      payload.inflow_cents = toCents(yuan.inflow) ?? undefined;
      payload.expense_fixed_monthly_cents = toCents(yuan.fixed) ?? undefined;
      payload.savings_cents = toCents(yuan.savings) ?? 0;
    }
    return payload;
  }

  function submit(next: number) {
    const payload = stepAnswers(step);
    setError("");
    startTransition(async () => {
      const result = await saveStepAction(step, payload);
      if (!result.ok) {
        setError(result.error ?? "保存失败,请稍后重试");
        return;
      }
      if (step === LAST_ANSWER_STEP) {
        setDone(true);
        return;
      }
      setStep(next);
    });
  }

  if (done) {
    return (
      <div className="flex flex-col gap-base-lg">
        <div className="rounded-lg border border-divider bg-bg-card p-base-xl">
          <p className="text-body font-medium text-text-title">问卷已完成</p>
          <p className="pt-base-xs text-label text-text-aux">
            你的档案已保存。模式推荐与方案生成是下一步开发的内容,届时这里会直接给出推荐结果。
          </p>
        </div>
        <Link href="/" className="text-body text-primary underline">
          返回首页
        </Link>
      </div>
    );
  }

  const copy = STEP_COPY[step - 1];

  return (
    <div className="flex flex-col gap-base-lg">
      {/* 顶部工具行:退出向导与草稿提示(design-v2 §1.4) */}
      <div className="flex flex-wrap items-center justify-between gap-base-md text-label">
        <Link href="/" className="text-text-aux hover:text-primary">
          ← 返回首页
        </Link>
        <span className="text-text-aux">每步自动保存草稿,中断后可恢复</span>
      </div>

      {/* 步进条 */}
      <div
        role="progressbar"
        aria-valuenow={step}
        aria-valuemin={1}
        aria-valuemax={TOTAL_STEPS}
        aria-label={`第 ${step} 步,共 ${TOTAL_STEPS} 步`}
        className="flex items-center gap-base-sm"
      >
        {STEP_NUMBERS.map((n) => (
          <span
            key={n}
            aria-hidden="true"
            className={cn(
              "h-1 flex-1 rounded-full",
              n < step ? "bg-primary-light" : n === step ? "bg-primary" : "bg-divider",
            )}
          />
        ))}
        <span className="shrink-0 text-label text-text-aux">
          {step} / {TOTAL_STEPS}
        </span>
      </div>

      <header className="flex flex-col gap-base-xs">
        <h1 className="text-section-title text-text-title">{copy.title}</h1>
        <p className="text-label text-text-aux">{copy.subtitle}</p>
      </header>

      {step === 1 ? (
        <Choice<Horizon>
          options={HORIZON_OPTIONS}
          value={answers.horizon}
          onSelect={(v) => patch({ horizon: v })}
        />
      ) : null}

      {step === 2 ? (
        <Choice<DrawdownResponse>
          options={DRAWDOWN_OPTIONS}
          value={answers.drawdown_response}
          onSelect={(v) => patch({ drawdown_response: v })}
        />
      ) : null}

      {step === 3 ? (
        <Choice<IncomeStability>
          options={STABILITY_OPTIONS}
          value={answers.income_stability}
          onSelect={(v) => patch({ income_stability: v })}
        />
      ) : null}

      {step === 4 ? (
        <div className="flex flex-col gap-base-lg">
          <div className="flex flex-col gap-base-sm">
            <span className="text-label font-medium text-text-title">社保</span>
            <Choice<"yes" | "no">
              options={[
                { value: "yes", label: "有" },
                { value: "no", label: "没有" },
              ]}
              value={
                answers.has_social_security === undefined
                  ? undefined
                  : answers.has_social_security
                    ? "yes"
                    : "no"
              }
              onSelect={(v) => patch({ has_social_security: v === "yes" })}
            />
          </div>
          <div className="flex flex-col gap-base-sm">
            <span className="text-label font-medium text-text-title">商业保险</span>
            <Choice<"yes" | "no">
              options={[
                { value: "yes", label: "有" },
                { value: "no", label: "没有" },
              ]}
              value={
                answers.has_commercial_insurance === undefined
                  ? undefined
                  : answers.has_commercial_insurance
                    ? "yes"
                    : "no"
              }
              onSelect={(v) => patch({ has_commercial_insurance: v === "yes" })}
            />
          </div>
          <NumberField
            id="mortgage"
            label="房贷余额(选填)"
            hint="本期仅记录,不参与计算"
            value={yuan.mortgage}
            onChange={(v) => setYuan((p) => ({ ...p, mortgage: v }))}
          />
          <NumberField
            id="dependents"
            label="需赡养人数"
            hint="会影响应急金月数(上浮一档)"
            value={yuan.dependents}
            onChange={(v) => setYuan((p) => ({ ...p, dependents: v }))}
            required
          />
        </div>
      ) : null}

      {step === 5 ? (
        <div className="flex flex-col gap-base-lg">
          <NumberField
            id="inflow"
            label="月收入(税后)"
            value={yuan.inflow}
            onChange={(v) => setYuan((p) => ({ ...p, inflow: v }))}
            required
          />
          <NumberField
            id="fixed"
            label="月固定支出"
            hint="房租 / 房贷月供 / 车贷等每月必付"
            value={yuan.fixed}
            onChange={(v) => setYuan((p) => ({ ...p, fixed: v }))}
            required
          />
          <NumberField
            id="savings"
            label="现有存款(选填)"
            hint="缺省视为 0"
            value={yuan.savings}
            onChange={(v) => setYuan((p) => ({ ...p, savings: v }))}
          />
          <div className="flex flex-col gap-base-sm">
            <span className="text-label font-medium text-text-title">
              理财目标 <span className="text-danger">*</span>
            </span>
            <Choice<Goal>
              options={GOAL_OPTIONS}
              value={answers.goal}
              onSelect={(v) => patch({ goal: v })}
            />
          </div>
        </div>
      ) : null}

      {error ? (
        <p
          role="alert"
          className="rounded-sm bg-bg-subtle px-base-md py-base-sm text-label text-danger"
        >
          {error}
        </p>
      ) : null}

      {/* 固定操作栏:跨步骤位置恒定,不随内容高度跳动(design-v2 §2.1) */}
      <div className="sticky bottom-0 flex gap-base-md border-t border-divider bg-bg-page py-base-md">
        {step > 1 ? (
          <Button
            variant="secondary"
            type="button"
            onClick={() => setStep(step - 1)}
            disabled={pending}
          >
            ← 上一步
          </Button>
        ) : null}
        <Button
          type="button"
          className="ml-auto"
          onClick={() => submit(step + 1)}
          disabled={pending}
        >
          {pending ? "保存中…" : step === LAST_ANSWER_STEP ? "完成问卷" : "下一步 →"}
        </Button>
      </div>
    </div>
  );
}
