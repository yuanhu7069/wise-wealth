//! profiles 表数据访问。一行/用户,整行 upsert(合并逻辑在 service 层,
//! 那里能看到「这一步该改哪几个字段」的业务语义)。

use sqlx::PgPool;
use uuid::Uuid;

/// 档案行。枚举列以文本存取,解析成领域枚举在 service 层做 —— repo 不做业务判断。
#[derive(Debug, Clone)]
pub struct ProfileRow {
    /// 资金久期
    pub horizon: Option<String>,
    /// 回撤反应
    pub drawdown_response: Option<String>,
    /// 收入稳定性
    pub income_stability: Option<String>,
    /// 社保(本期仅记录)
    pub has_social_security: bool,
    /// 商业保险(本期仅记录)
    pub has_commercial_insurance: bool,
    /// 房贷余额(本期仅记录)
    pub mortgage_balance_cents: Option<i64>,
    /// 需赡养人数
    pub dependents: Option<i16>,
    /// 税后月收入(分)
    pub inflow_cents: Option<i64>,
    /// 月固定支出(分)
    pub expense_fixed_monthly_cents: Option<i64>,
    /// 现有存款(分)
    pub savings_cents: Option<i64>,
    /// 理财目标
    pub goal: Option<String>,
    /// 下一步该答的步号(1-6)
    pub draft_step: i16,
    /// 问卷是否已完成
    pub questionnaire_completed: bool,
}

impl Default for ProfileRow {
    fn default() -> Self {
        Self {
            horizon: None,
            drawdown_response: None,
            income_stability: None,
            has_social_security: false,
            has_commercial_insurance: false,
            mortgage_balance_cents: None,
            dependents: None,
            inflow_cents: None,
            expense_fixed_monthly_cents: None,
            savings_cents: None,
            goal: None,
            draft_step: 1,
            questionnaire_completed: false,
        }
    }
}

impl ProfileRow {
    /// 问卷是否真的答全了。**不看 `questionnaire_completed` 标志** ——
    /// 那个标志只说明「用户答过步 5」,不保证步 1-4 都答了(跳步就能绕过)。
    pub fn is_complete(&self) -> bool {
        self.horizon.is_some()
            && self.drawdown_response.is_some()
            && self.income_stability.is_some()
            && self.dependents.is_some()
            && self.inflow_cents.is_some()
            && self.expense_fixed_monthly_cents.is_some()
            && self.goal.is_some()
    }
}

/// 读取用户档案。`None` 表示还没答过任何一步。
pub async fn get(pool: &PgPool, user_id: Uuid) -> Result<Option<ProfileRow>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT horizon, drawdown_response, income_stability,
               has_social_security, has_commercial_insurance, mortgage_balance_cents,
               dependents, inflow_cents, expense_fixed_monthly_cents, savings_cents,
               goal, draft_step, questionnaire_completed
        FROM profiles WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| ProfileRow {
        horizon: r.horizon,
        drawdown_response: r.drawdown_response,
        income_stability: r.income_stability,
        has_social_security: r.has_social_security,
        has_commercial_insurance: r.has_commercial_insurance,
        mortgage_balance_cents: r.mortgage_balance_cents,
        dependents: r.dependents,
        inflow_cents: r.inflow_cents,
        expense_fixed_monthly_cents: r.expense_fixed_monthly_cents,
        savings_cents: r.savings_cents,
        goal: r.goal,
        draft_step: r.draft_step,
        questionnaire_completed: r.questionnaire_completed,
    }))
}

/// 整行 upsert。service 层负责把「这一步的答案」合并进既有行,这里只管写。
pub async fn upsert(pool: &PgPool, user_id: Uuid, row: &ProfileRow) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO profiles (
            id, user_id, horizon, drawdown_response, income_stability,
            has_social_security, has_commercial_insurance, mortgage_balance_cents,
            dependents, inflow_cents, expense_fixed_monthly_cents, savings_cents,
            goal, draft_step, questionnaire_completed
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
        ON CONFLICT (user_id) DO UPDATE SET
            horizon = EXCLUDED.horizon,
            drawdown_response = EXCLUDED.drawdown_response,
            income_stability = EXCLUDED.income_stability,
            has_social_security = EXCLUDED.has_social_security,
            has_commercial_insurance = EXCLUDED.has_commercial_insurance,
            mortgage_balance_cents = EXCLUDED.mortgage_balance_cents,
            dependents = EXCLUDED.dependents,
            inflow_cents = EXCLUDED.inflow_cents,
            expense_fixed_monthly_cents = EXCLUDED.expense_fixed_monthly_cents,
            savings_cents = EXCLUDED.savings_cents,
            goal = EXCLUDED.goal,
            draft_step = EXCLUDED.draft_step,
            questionnaire_completed = EXCLUDED.questionnaire_completed,
            updated_at = now()
        "#,
        Uuid::now_v7(),
        user_id,
        row.horizon,
        row.drawdown_response,
        row.income_stability,
        row.has_social_security,
        row.has_commercial_insurance,
        row.mortgage_balance_cents,
        row.dependents,
        row.inflow_cents,
        row.expense_fixed_monthly_cents,
        row.savings_cents,
        row.goal,
        row.draft_step,
        row.questionnaire_completed,
    )
    .execute(pool)
    .await?;
    Ok(())
}
