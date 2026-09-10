//! 埋点上报端点(受保护组)。
//!
//! 只收**客户端才知道**的两类事件:页面触达与问卷开始。后端自己知道的三类
//! (每步保存、问卷答全、方案生成)直接在各自 handler 里记,不走本端点 ——
//! 同一事实两个来源迟早会不一致(与 prd-v1 §9.5 的事件定义一一对应)。

use axum::extract::State;
use axum::Json;

use crate::dto::analytics::{ClientEventRequest, EventAck};
use crate::error::{ApiOk, AppError};
use crate::services::analytics_service::{self, Event, EventError};
use crate::state::AppState;

impl From<EventError> for AppError {
    fn from(e: EventError) -> Self {
        // 白名单拒收属于「请求本身不合法」,422 而不是 400:前端能按信封展示 message
        AppError::Validation(e.to_string())
    }
}

/// POST /api/v1/analytics/events —— 记录一条客户端埋点。
///
/// 成功恒 200:埋点的写失败在 service 层就被吞掉(RULE-019,只留告警日志),
/// 上报端拿不到「写没写进去」—— 否则调用方迟早会为它加分支,那就成了主流程的依赖。
#[utoipa::path(
    post,
    path = "/api/v1/analytics/events",
    tag = "analytics",
    summary = "上报埋点事件(仅 page_view / questionnaire_start)",
    request_body = ClientEventRequest,
    responses(
        (status = 200, description = "已受理", body = crate::error::Envelope<EventAck>),
        (status = 401, description = "未登录或会话过期"),
        (status = 422, description = "事件名不在白名单,或 page_id 与该事件不匹配")
    )
)]
pub async fn record_event(
    State(state): State<AppState>,
    Json(req): Json<ClientEventRequest>,
) -> Result<ApiOk<EventAck>, AppError> {
    let event = Event::from_client(&req.event, req.page_id.as_deref())?;
    analytics_service::record(&state.pool, &event).await;
    Ok(ApiOk(EventAck { accepted: true }))
}
