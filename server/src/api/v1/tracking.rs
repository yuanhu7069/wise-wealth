//! 季度复盘端点(受保护组,H 期 RULE-049 ~ RULE-051,ADR-H-003)。
//!
//! 复盘免费全量:本端点**不读 tier**(RULE-051)—— 分层闸门只存在于推理链,
//! 追踪口径对两种分层完全一致。纯读聚合,响应单请求渲染 P09。

use axum::extract::{Query, State};
use axum::Extension;
use serde::Deserialize;
use uuid::Uuid;

use crate::api::middleware::CurrentUser;
use crate::dto::snapshot::{BucketDeltaView, EmergencyConvergenceView, ReviewView};
use crate::error::{ApiOk, AppError};
use crate::services::snapshot_service;
use crate::state::AppState;

/// `?quarter=YYYY-QN`(缺省 = 服务器当前季)。
#[derive(Debug, Deserialize)]
pub struct ReviewQuery {
    /// 季度标识;缺省取服务器当前季(前端切换器只产合法值,直链非法值 → 422)
    pub quarter: Option<String>,
}

/// GET /api/v1/tracking/review —— 季度复盘(RULE-049/050)。
#[utoipa::path(
    get,
    path = "/api/v1/tracking/review",
    tag = "tracking",
    summary = "季度复盘(坚持月数 / 各桶季内变化 / 应急缺口收敛;免费全量)",
    params(("quarter" = Option<String>, Query, description = "季度 YYYY-QN,缺省当前季")),
    responses(
        (status = 200, description = "复盘聚合", body = crate::error::Envelope<ReviewView>),
        (status = 401, description = "未登录或会话过期"),
        (status = 422, description = "季度参数非法")
    )
)]
pub async fn quarter_review(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Query(q): Query<ReviewQuery>,
) -> Result<ApiOk<ReviewView>, AppError> {
    let user_id = Uuid::parse_str(&user.id).map_err(|_| AppError::Unauthorized)?;

    let data =
        snapshot_service::quarter_review(&state.pool, &state.library, user_id, q.quarter.as_deref())
            .await?;
    let r = data.review;

    Ok(ApiOk(ReviewView {
        quarter: r.quarter,
        persisted_months: r.persisted_months as i64,
        quarter_snapshot_count: r.quarter_snapshot_count as i64,
        comparable: r.comparable,
        buckets: r
            .buckets
            .into_iter()
            .map(|b| {
                // 缺名回落桶 id(桶结构漂移的防御;正常路径 names 已覆盖)
                let name = data
                    .names
                    .get(&b.bucket_id)
                    .cloned()
                    .unwrap_or_else(|| b.bucket_id.clone());
                BucketDeltaView {
                    bucket_id: b.bucket_id,
                    name,
                    quarter_start_cents: b.start_cents,
                    latest_cents: b.latest_cents,
                    delta_cents: b.delta_cents,
                }
            })
            .collect(),
        emergency: r.emergency.map(|e| EmergencyConvergenceView {
            target_cents: e.target_cents,
            start_gap_cents: e.start_gap_cents,
            current_gap_cents: e.current_gap_cents,
            avg_monthly_convergence_cents: e.avg_monthly_convergence_cents,
            months_to_goal: e.months_to_goal,
            met: e.met,
        }),
        special_months: r.special_months,
        // 当前季的后端权威值:切换器的边界判定以此为准(承 E 期评审发现 #4 立场)
        current_quarter: snapshot_service::current_quarter(),
    }))
}
