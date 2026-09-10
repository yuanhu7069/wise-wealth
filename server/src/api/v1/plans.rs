//! 方案端点(受保护组):生成方案 / 读当前方案(RULE-011)。

use axum::extract::State;
use axum::{Extension, Json};
use uuid::Uuid;

use crate::api::middleware::CurrentUser;
use crate::api::v1::profiles::to_domain_profile;
use crate::domain::engine::{EmergencyStatus, Notice};
use crate::domain::l2::L2Allocation;
use crate::dto::plan::{BucketView, GeneratePlanRequest, PlanView};
use crate::error::{ApiOk, AppError};
use crate::repos;
use crate::repos::plans::{BucketRow, PlanRecord};
use crate::services::plan_service::{self, PlanError};
use crate::state::AppState;

impl From<PlanError> for AppError {
    fn from(e: PlanError) -> Self {
        match e {
            // 用户能自己修:让前端按 422 给「请先完成问卷」
            PlanError::ProfileIncomplete | PlanError::UnknownMode(_) => {
                AppError::Validation(e.to_string())
            }
            PlanError::Engine(inner) => AppError::Validation(inner.to_string()),
            PlanError::Db(inner) => AppError::Database(inner),
        }
    }
}

/// 方案行 + 桶 → 视图。快照字段反序列化成强类型(前端类型由 OpenAPI 生成)。
fn to_view(
    state: &AppState,
    plan: &PlanRecord,
    buckets: Vec<BucketRow>,
) -> Result<PlanView, AppError> {
    let l2: L2Allocation = serde_json::from_value(plan.l2_allocation.clone())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("L2 快照损坏: {e}")))?;
    let emergency: EmergencyStatus = serde_json::from_value(plan.emergency.clone())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("应急金快照损坏: {e}")))?;
    let notices: Vec<Notice> = serde_json::from_value(plan.notices.clone())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("提示快照损坏: {e}")))?;

    let l1_mode_name = state
        .library
        .mode(&plan.l1_mode)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| plan.l1_mode.clone());

    Ok(PlanView {
        id: plan.id,
        version: plan.version,
        created_date: plan.created_date.clone(),
        l1_mode: plan.l1_mode.clone(),
        l1_mode_name,
        buckets: buckets
            .into_iter()
            .map(|b| BucketView {
                bucket_id: b.bucket_id,
                name: b.name,
                purpose: b.purpose,
                amount_monthly_cents: b.amount_monthly_cents,
                target_cents: b.target_cents,
            })
            .collect(),
        l2,
        emergency,
        notices,
    })
}

/// POST /api/v1/plans —— 按选定的 L1 模式生成方案并落库(版本化)。
#[utoipa::path(
    post,
    path = "/api/v1/plans",
    tag = "plans",
    summary = "生成方案(重新生成会新增版本,旧版本保留)",
    request_body = GeneratePlanRequest,
    responses(
        (status = 200, description = "生成的方案", body = crate::error::Envelope<PlanView>),
        (status = 401, description = "未登录或会话过期"),
        (status = 422, description = "问卷未完成或模式不存在")
    )
)]
pub async fn generate_plan(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<GeneratePlanRequest>,
) -> Result<ApiOk<PlanView>, AppError> {
    let user_id = Uuid::parse_str(&user.id).map_err(|_| AppError::Unauthorized)?;

    let row = repos::profiles::get(&state.pool, user_id)
        .await?
        .ok_or(PlanError::ProfileIncomplete)?;
    if !row.is_complete() {
        return Err(PlanError::ProfileIncomplete.into());
    }
    let profile = to_domain_profile(&row).ok_or(PlanError::ProfileIncomplete)?;

    let generated =
        plan_service::generate(&state.pool, &state.library, user_id, &req.l1_mode, &profile)
            .await?;

    // 基线 §11.1:生成方案是关键业务操作,留审计日志(只记模式与版本,金额不入日志)
    tracing::info!(
        user = %user.username,
        mode = %generated.plan.l1_mode,
        version = generated.plan.version,
        "方案已生成"
    );

    let view = to_view(&state, &generated.plan, generated.buckets)?;
    Ok(ApiOk(view))
}

/// GET /api/v1/plans/active —— 读当前方案;没有生成过则 404。
#[utoipa::path(
    get,
    path = "/api/v1/plans/active",
    tag = "plans",
    summary = "读取当前方案",
    responses(
        (status = 200, description = "当前方案", body = crate::error::Envelope<PlanView>),
        (status = 401, description = "未登录或会话过期"),
        (status = 404, description = "还没有生成过方案")
    )
)]
pub async fn active_plan(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<ApiOk<PlanView>, AppError> {
    let user_id = Uuid::parse_str(&user.id).map_err(|_| AppError::Unauthorized)?;

    let plan = repos::plans::active_for_user(&state.pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("还没有生成过方案".into()))?;
    let buckets = repos::plans::buckets_of(&state.pool, plan.id).await?;

    Ok(ApiOk(to_view(&state, &plan, buckets)?))
}
