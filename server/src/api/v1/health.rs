//! health handler(协议转换层,只做取 service 结果 → 包信封)。

use crate::error::ApiOk;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;

pub use crate::services::health_service::HealthData;

/// GET /health 与 GET /api/v1/health 双挂(arch-a.md §4)。
///
/// ADR-A-003:服务进程存活即 HTTP 200;数据库状态由 data.db 表达(ok|error)。
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    summary = "服务与数据库连通探测(ADR-A-003:进程存活恒 200)",
    responses(
        (status = 200, description = "统一信封;data.db=ok|error 表达数据库状态", body = crate::error::Envelope<HealthData>)
    )
)]
pub async fn health_handler(
    State(state): State<AppState>,
) -> (StatusCode, ApiOk<HealthData>) {
    let data =
        crate::services::health_service::check(&state.pool, state.version).await;
    (StatusCode::OK, ApiOk(data))
}
