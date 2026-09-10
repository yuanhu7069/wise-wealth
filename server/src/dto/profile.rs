//! 问卷档案 DTO(RULE-003/004/005)。
//!
//! 分步提交的入参是一个**扁平结构 + 步号**:每步只填自己那几个字段,其余留空。
//! 用 `Option` 而非每步一个请求体,是为了让「这一步该有哪些字段」这条规则集中在
//! service 的校验里,而不是散落在多个 DTO 类型上。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::profile::{DrawdownResponse, Goal, Horizon, IncomeStability};

/// 保存某一步答案的请求。
#[derive(Debug, Deserialize, ToSchema)]
pub struct StepRequest {
    /// 步号 1-6(1 久期 / 2 回撤反应 / 3 收入稳定性 / 4 保障与负债 / 5 财务数字 / 6 推荐)
    pub step: i16,
    /// 步 1
    pub horizon: Option<Horizon>,
    /// 步 2
    pub drawdown_response: Option<DrawdownResponse>,
    /// 步 3
    pub income_stability: Option<IncomeStability>,
    /// 步 4:社保
    pub has_social_security: Option<bool>,
    /// 步 4:商业保险
    pub has_commercial_insurance: Option<bool>,
    /// 步 4:房贷余额(分,选填;本期仅记录)
    pub mortgage_balance_cents: Option<i64>,
    /// 步 4:需赡养人数(0-20)
    pub dependents: Option<i16>,
    /// 步 5:税后月收入(分)
    pub inflow_cents: Option<i64>,
    /// 步 5:月固定支出(分)
    pub expense_fixed_monthly_cents: Option<i64>,
    /// 步 5:现有存款(分,选填;缺省视为 0)
    pub savings_cents: Option<i64>,
    /// 步 5:理财目标
    pub goal: Option<Goal>,
}

/// 档案回读(供问卷页恢复草稿与后续推荐/方案使用)。
#[derive(Debug, Serialize, ToSchema)]
pub struct ProfileView {
    /// 下一步该答的步号
    pub draft_step: i16,
    /// 问卷是否已完成
    pub questionnaire_completed: bool,
    /// 步 1
    pub horizon: Option<Horizon>,
    /// 步 2
    pub drawdown_response: Option<DrawdownResponse>,
    /// 步 3
    pub income_stability: Option<IncomeStability>,
    /// 步 4
    pub has_social_security: Option<bool>,
    /// 步 4
    pub has_commercial_insurance: Option<bool>,
    /// 步 4
    pub mortgage_balance_cents: Option<i64>,
    /// 步 4
    pub dependents: Option<i16>,
    /// 步 5
    pub inflow_cents: Option<i64>,
    /// 步 5
    pub expense_fixed_monthly_cents: Option<i64>,
    /// 步 5
    pub savings_cents: Option<i64>,
    /// 步 5
    pub goal: Option<Goal>,
}
