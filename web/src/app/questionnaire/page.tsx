/**
 * P03 问卷页(server 壳)。职责:要求会话 → 读档案(断点恢复与预填)→ 交给向导。
 */
import { Card, CardContent } from "@/components/ui/card";
import { trackPageView, trackQuestionnaireStart } from "@/lib/analytics";
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

interface QuestionnairePageProps {
  /** Next 以 Promise 形式提供 searchParams(Next 15+) */
  searchParams: Promise<{ restart?: string }>;
}

export default async function QuestionnairePage({ searchParams }: QuestionnairePageProps) {
  // RULE-001:问卷属受保护页面,未登录跳登录并在登录后回到本页
  await requireSession("/questionnaire");
  // 埋点(prd-v1 §9.5:页面触达)
  await trackPageView("p03");

  const store = await cookies();
  const token = store.get(SESSION_COOKIE)?.value ?? "";

  // restart=1(重新生成入口):从步 1 走,答案照常预填;无参则按断点续答
  const restart = (await searchParams).restart === "1";

  let step = 1;
  let answers: Answers = {};
  /** 档案读到了没有 —— 决定要不要记「问卷开始」(读不到时不该记,见下) */
  let hasProfile = false;
  try {
    const { envelope } = await apiGet<ProfileView>("/api/v1/profiles/me", {
      cookie: `${SESSION_COOKIE}=${token}`,
    });
    if (envelope.success && envelope.data) {
      hasProfile = true;
      const p = envelope.data;
      // 断点即进度:续答场景(未带 restart)答完的人再进来直接看到推荐(步 6)
      step = restart ? 1 : Math.max(1, Math.min(p.draft_step, TOTAL_STEPS));
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

  // 埋点(prd-v1 §9.5:问卷开始)。判据是「读到了档案且草稿还停在步 1」——
  // 任何一步保存成功都会把 draft_step 推到 ≥ 2(profile_service),所以它就是
  // 「尚无草稿」最直接的表达。档案读不到时不记:那是后端故障,不是问卷开始。
  // 口径:每次「无草稿进入」都算一次(与 page_view 同),未作答就反复进来会重复记 ——
  // 自用期用来看链路通不通,不做产品指标;真要算开始率,分母按去重后的问卷数算。
  if (hasProfile && step === 1 && !restart) {
    await trackQuestionnaireStart();
  }

  return (
    // 外壳全高拉伸(design-v2 v0.5「操作栏位置恒定」的落地前提):main/Card/CardContent
    // 逐层 flex-1,内容矮时操作栏经 wizard 根的 mt-auto 恒钉视口底,不再随步骤内容高度跳动
    <main className="mx-auto flex w-full max-w-xl flex-1 flex-col px-base-lg pb-base-lg pt-base-xxl">
      <Card className="flex flex-1 flex-col">
        <CardContent className="flex flex-1 flex-col pt-base-xl">
          <Wizard initialStep={step} initialAnswers={answers} />
        </CardContent>
      </Card>
    </main>
  );
}
