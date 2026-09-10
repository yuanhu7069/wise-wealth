/**
 * P03 问卷页(server 壳)。职责:要求会话 → 读档案(断点恢复与预填)→ 交给向导。
 */
import { Card, CardContent } from "@/components/ui/card";
import { apiGet } from "@/lib/api";
import { requireSession, SESSION_COOKIE } from "@/lib/session";
import { cookies } from "next/headers";

import { type Answers, TOTAL_STEPS } from "./state";
import { Wizard } from "./wizard";

interface ProfileView {
  draft_step: number;
  questionnaire_completed: boolean;
  horizon?: Answers["horizon"];
  drawdown_response?: Answers["drawdown_response"];
  income_stability?: Answers["income_stability"];
  has_social_security?: boolean;
  has_commercial_insurance?: boolean;
  mortgage_balance_cents?: number;
  dependents?: number;
  inflow_cents?: number;
  expense_fixed_monthly_cents?: number;
  savings_cents?: number;
  goal?: Answers["goal"];
}

export default async function QuestionnairePage() {
  // RULE-001:问卷属受保护页面,未登录跳登录并在登录后回到本页
  await requireSession("/questionnaire");

  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value ?? "";

  let step = 1;
  let answers: Answers = {};
  try {
    const { envelope } = await apiGet<ProfileView>("/api/v1/profiles/me", {
      cookie: `${SESSION_COOKIE}=${token}`,
    });
    if (envelope.success && envelope.data) {
      const p = envelope.data;
      // 断点即进度:答完的人再进来直接看到推荐(步 6),要改答案就点「上一步」
      step = Math.max(1, Math.min(p.draft_step, TOTAL_STEPS));
      answers = {
        horizon: p.horizon,
        drawdown_response: p.drawdown_response,
        income_stability: p.income_stability,
        has_social_security: p.has_social_security,
        has_commercial_insurance: p.has_commercial_insurance,
        mortgage_balance_cents: p.mortgage_balance_cents ?? undefined,
        dependents: p.dependents ?? undefined,
        inflow_cents: p.inflow_cents ?? undefined,
        expense_fixed_monthly_cents: p.expense_fixed_monthly_cents ?? undefined,
        savings_cents: p.savings_cents ?? undefined,
        goal: p.goal,
      };
    }
  } catch {
    // 取档案失败不该挡住问卷:从第一步开始答,草稿仍在服务端
    step = 1;
  }

  return (
    <main className="mx-auto w-full max-w-xl px-base-lg py-base-xxl">
      <Card>
        <CardContent className="pt-base-xl">
          <Wizard initialStep={step} initialAnswers={answers} />
        </CardContent>
      </Card>
    </main>
  );
}
