"use client";

/**
 * 问卷向导(P03)。一屏一题,每步自动保存,中断可续。
 *
 * 步 1-5 是作答,步 6 是推荐位(ticket 04 接入)——本工单先把作答链路做通,
 * 完成态如实告诉用户下一步会是什么,不用占位内容假装已实现。
 */
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useState, useTransition } from "react";

import { Button } from "@/components/ui/button";
import { LEGAL_LINE, LEGAL_LINE_SHORT } from "@/lib/disclaimer-copy";
import { cn } from "@/lib/utils";

import { generatePlanAction } from "@/app/plan/actions";
import { loadModesAction, saveStepAction } from "./actions";
import {
  type Answers,
  DRAWDOWN_OPTIONS,
  GOAL_OPTIONS,
  HORIZON_OPTIONS,
  CREDIBILITY_LABEL,
  type ModesData,
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
              "flex min-h-11 items-center gap-base-md rounded-sm border px-base-lg py-base-md text-left text-body-md",
              selected
                ? "border-2 border-primary bg-primary/10 text-ink"
                : "border-hairline bg-canvas-card text-ink-secondary hover:border-ink-mute",
            )}
          >
            <span
              aria-hidden="true"
              className={cn(
                "size-4 shrink-0 rounded-full border-2",
                selected ? "border-primary bg-primary" : "border-hairline",
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
      <label htmlFor={id} className="text-caption text-ink">
        {label} {required ? <span className="text-danger">*</span> : null}
      </label>
      <input
        id={id}
        inputMode="decimal"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="h-11 rounded-sm border border-hairline bg-canvas-card px-base-md text-body-md text-ink-secondary focus:border-primary focus:outline-none sm:h-10"
      />
      {hint ? <p className="text-caption text-ink-mute">{hint}</p> : null}
    </div>
  );
}

/** 步号序列。用步号本身作 key 而非数组下标 —— 分段的位置即它的身份。 */
const STEP_NUMBERS = Array.from({ length: TOTAL_STEPS }, (_, i) => i + 1);

export function Wizard({ initialStep, initialAnswers }: WizardProps) {
  // 钳制到总步数(不是"最后作答步"):答完的人应当直接落在步 6 的推荐上
  const [step, setStep] = useState(Math.min(Math.max(initialStep, 1), TOTAL_STEPS));
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
  const [pending, startTransition] = useTransition();
  // 步 6:推荐按需拉取(可能是用户本次刚答完,页面初次渲染时还没有档案)
  const [modes, setModes] = useState<ModesData | null>(null);
  const [modesError, setModesError] = useState("");
  const [chosen, setChosen] = useState<string>("");
  // 方案生成的失败态:文案 + 是否只是会话过期(决定给「重试」还是「重新登录」)
  const [planError, setPlanError] = useState("");
  const [planExpired, setPlanExpired] = useState(false);
  const router = useRouter();

  /**
   * 步 6 的「生成方案」。生成成功跳 P04;失败留在本步并给出可重试的错误态 ——
   * 已答问卷都在服务端,重试不会让用户重答(prd-v1 §8.2 异常路径)。
   */
  function generatePlan() {
    setPlanError("");
    setPlanExpired(false);
    startTransition(async () => {
      const result = await generatePlanAction(chosen);
      if (result.ok) {
        router.push("/plan");
        return;
      }
      setPlanExpired(Boolean(result.expired));
      setPlanError(result.error);
    });
  }

  useEffect(() => {
    if (step !== TOTAL_STEPS || modes) return;
    let alive = true;
    void loadModesAction().then((result) => {
      if (!alive) return;
      if (result.ok) {
        setModes(result.data);
        // 默认选中推荐项:「生成方案」不因用户没点卡片而禁用 —— 推荐本身就是系统的
        // 首选建议,用户仍可改选另一张卡;仅当尚未显式选择时才代填(函数式更新避免闭包旧值)
        const rec = result.data.items.find((c) => c.is_recommended) ?? result.data.items[0];
        if (rec) setChosen((prev) => prev || rec.id);
      } else setModesError(result.error);
    });
    return () => {
      alive = false;
    };
  }, [step, modes]);

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
      setStep(next);
    });
  }

  const copy = STEP_COPY[step - 1];

  return (
    <div className="flex flex-1 flex-col gap-base-lg">
      {/* 顶部工具行:退出向导与草稿提示(design-v2 §1.4) */}
      <div className="flex flex-wrap items-center justify-between gap-base-md text-caption">
        {/* 触控热区 ≥ h-11(基线 §16):padding 撑开热区 + 负 margin 抵消,文字的视觉位置不变 */}
        <Link href="/" className="-my-base-lg py-base-lg text-ink-mute hover:text-primary">
          ← 返回首页
        </Link>
        <span className="text-ink-mute">每步自动保存草稿,中断后可恢复</span>
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
              n < step ? "bg-primary" : n === step ? "bg-primary" : "bg-hairline",
            )}
          />
        ))}
        <span className="shrink-0 text-caption text-ink-mute">
          {step} / {TOTAL_STEPS}
        </span>
      </div>

      <header className="flex flex-col gap-base-xs">
        <h1 className="text-heading-md text-ink">{copy.title}</h1>
        <p className="text-caption text-ink-mute">{copy.subtitle}</p>
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
        <div className="flex flex-1 flex-col gap-base-lg">
          <div className="flex flex-col gap-base-sm">
            <span className="text-caption text-ink">社保</span>
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
            <span className="text-caption text-ink">商业保险</span>
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
        <div className="flex flex-1 flex-col gap-base-lg">
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
            <span className="text-caption text-ink">
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

      {step === TOTAL_STEPS ? (
        <div className="flex flex-col gap-base-md">
          {modesError ? (
            <p role="alert" className="text-caption text-danger">
              {modesError}
            </p>
          ) : null}
          {!modes && !modesError ? (
            <p className="text-caption text-ink-mute">正在生成推荐…</p>
          ) : null}
          {modes ? (
            <>
              <div className="grid gap-base-md md:grid-cols-2">
                {modes.items.map((c) => (
                  <button
                    key={c.id}
                    type="button"
                    aria-pressed={chosen === c.id}
                    onClick={() => setChosen(c.id)}
                    className={cn(
                      "flex flex-col gap-base-sm rounded-sm border p-base-lg text-left",
                      chosen === c.id
                        ? "border-2 border-primary bg-primary/10"
                        : "border-hairline bg-canvas-card hover:border-ink-mute",
                    )}
                  >
                    <span className="flex flex-wrap items-center gap-base-xs">
                      {c.is_recommended ? (
                        <span className="rounded-full bg-primary px-base-sm text-micro-cap text-on-primary">
                          推荐
                        </span>
                      ) : null}
                      {/* 前景用 accent-foreground 而非 primary:暗色下 primary 落在浅绿底上对比度不足,
                          与 Badge 组件同一口径(2026-09-11 走查 §8-E) */}
                      <span className="rounded-full bg-primary-bg-subdued-hover px-base-sm text-micro-cap text-accent-foreground">
                        {CREDIBILITY_LABEL[c.credibility]}
                      </span>
                    </span>
                    <span className="text-body-lg text-ink">{c.name}</span>
                    <span className="text-caption text-ink-mute">{c.tagline}</span>
                    {c.is_recommended ? (
                      <span className="border-t border-hairline pt-base-sm text-caption text-ink-secondary">
                        {modes.recommendation_reason}
                      </span>
                    ) : null}
                  </button>
                ))}
              </div>
              <p className="text-caption text-ink-mute">
                投资部分将按 {modes.l2.name} 配置:
                {modes.l2.classes
                  .map((cl) => `${cl.name} ${Math.round(cl.basis_points / 100)}%`)
                  .join(" / ")}
              </p>
            </>
          ) : null}
          {planError ? (
            <div
              role="alert"
              className="flex flex-col gap-base-xs rounded-sm border border-hairline bg-canvas-card px-base-lg py-base-md"
            >
              <p className="text-body-md text-danger">方案没能生成</p>
              <p className="text-caption text-ink-secondary">{planError}</p>
              <p className="text-caption text-ink-mute">
                {planExpired
                  ? "重新登录后会回到这一步,已答的问卷都在。"
                  : "你已答的问卷都在,点下方「生成方案」可原样重试。"}
              </p>
            </div>
          ) : null}
        </div>
      ) : null}

      {error ? (
        <p
          role="alert"
          className="rounded-sm bg-canvas-card px-base-md py-base-sm text-caption text-danger"
        >
          {error}
        </p>
      ) : null}

      {/*
        固定操作栏:跨步骤位置恒定,不随内容高度跳动(design-v2 §2.1 / v0.5)。
        两条视觉分层的带(v0.7):上带读作「内容区的一部分」(页面底色 + 发丝分隔线),
        下带读作「独立声明条」(次级底色)。P03 **不显示站点页脚** —— 声明就在这一行(RULE-020)。
      */}
      <div className="sticky bottom-0 mt-auto border-t border-hairline">
        <div className="flex gap-base-md bg-canvas py-base-md">
          {step > 1 ? (
            <Button
              variant="ghost"
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
            onClick={() => (step === TOTAL_STEPS ? generatePlan() : submit(step + 1))}
            disabled={pending || (step === TOTAL_STEPS && !chosen)}
          >
            {pending
              ? step === TOTAL_STEPS
                ? "生成中…"
                : "保存中…"
              : step === TOTAL_STEPS
                ? "生成方案"
                : "下一步 →"}
          </Button>
        </div>
        <div className="border-t border-hairline bg-canvas-soft px-base-md py-base-xs">
          {/* 移动端用更短文案(text-micro 档)保证 375 宽下真正单行不折 —— RULE-020 的显式偏离 */}
          <p className="text-micro text-ink-mute-2 sm:text-caption">
            <span className="sm:hidden">{LEGAL_LINE_SHORT}</span>
            <span className="hidden sm:inline">{LEGAL_LINE}</span>
          </p>
        </div>
      </div>
    </div>
  );
}
