//! 档案端点(受保护组):读取草稿 / 保存单步答案(RULE-003)。

use axum::extract::State;
use axum::{Extension, Json};

use crate::api::middleware::CurrentUser;
use crate::domain::profile::{DrawdownResponse, Goal, Horizon, IncomeStability};
use crate::dto::profile::{ProfileView, StepRequest};
use crate::error::{ApiOk, AppError};
use crate::repos;
use crate::repos::profiles::ProfileRow;
use crate::services::profile_service::{self, StepError};
use crate::state::AppState;

/// 文本列 → 领域枚举。写路径已锁定格式,读路径遇到无法识别的值按「未填」处理,
/// 不让一条脏数据把整个档案接口打成 500。
fn parse<T: serde::de::DeserializeOwned>(raw: &Option<String>) -> Option<T> {
    raw.as_ref()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.clone())).ok())
}

/// 行 → 视图(DTO 与表结构的翻译层)
fn to_view(row: &ProfileRow) -> ProfileView {
    ProfileView {
        draft_step: row.draft_step,
        questionnaire_completed: row.questionnaire_completed,
        horizon: parse::<Horizon>(&row.horizon),
        drawdown_response: parse::<DrawdownResponse>(&row.drawdown_response),
        income_stability: parse::<IncomeStability>(&row.income_stability),
        has_social_security: Some(row.has_social_security),
        has_commercial_insurance: Some(row.has_commercial_insurance),
        mortgage_balance_cents: row.mortgage_balance_cents,
        dependents: row.dependents,
        inflow_cents: row.inflow_cents,
        expense_fixed_monthly_cents: row.expense_fixed_monthly_cents,
        savings_cents: row.savings_cents,
        goal: parse::<Goal>(&row.goal),
    }
}

impl From<StepError> for AppError {
    fn from(e: StepError) -> Self {
        AppError::Validation(e.to_string())
    }
}

/// GET /api/v1/profiles/me —— 读取档案(含草稿步号,供问卷页断点恢复)。
#[utoipa::path(
    get,
    path = "/api/v1/profiles/me",
    tag = "profile",
    summary = "读取问卷档案与草稿进度",
    responses(
        (status = 200, description = "档案(未答过则返回全空 + draft_step=1)", body = crate::error::Envelope<ProfileView>),
        (status = 401, description = "未登录或会话过期")
    )
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<ApiOk<ProfileView>, AppError> {
    let user_id = parse_user_id(&user)?;
    let row = repos::profiles::get(&state.pool, user_id)
        .await?
        .unwrap_or_default();
    Ok(ApiOk(to_view(&row)))
}

/// PUT /api/v1/profiles/me/step —— 保存某一步答案(分步校验;RULE-004/005)。
#[utoipa::path(
    put,
    path = "/api/v1/profiles/me/step",
    tag = "profile",
    summary = "保存问卷某一步答案并推进草稿步号",
    request_body = StepRequest,
    responses(
        (status = 200, description = "保存后的档案", body = crate::error::Envelope<ProfileView>),
        (status = 401, description = "未登录或会话过期"),
        (status = 422, description = "字段校验失败(缺必填项或取值越界)")
    )
)]
pub async fn save_step(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<StepRequest>,
) -> Result<ApiOk<ProfileView>, AppError> {
    let user_id = parse_user_id(&user)?;

    // 读 → 合并(纯函数,规则都在这里) → 写。单用户场景无需担心并发覆盖。
    let current = repos::profiles::get(&state.pool, user_id)
        .await?
        .unwrap_or_default();
    let merged = profile_service::apply_step(&current, &req)?;
    repos::profiles::upsert(&state.pool, user_id, &merged).await?;

    // 基线 §11.1:关键业务操作留审计日志。问卷步骤承载收入/存款等财务字段,
    // 属于「金额变更」一类,写入必须留痕(只记步号与用户名,金额不入日志)。
    tracing::info!(user = %user.username, step = req.step, "问卷步骤已保存");

    Ok(ApiOk(to_view(&merged)))
}

/// 会话里的用户 id 是 UUID 字符串。会话由本服务签发,格式已锁定;
/// 解析失败意味着令牌被篡改过,按未认证处理。
fn parse_user_id(user: &CurrentUser) -> Result<uuid::Uuid, AppError> {
    uuid::Uuid::parse_str(&user.id).map_err(|_| AppError::Unauthorized)
}
