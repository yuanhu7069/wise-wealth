//! handler 层路由装配(只做协议转换,业务在 services 层)。

pub mod v1 {
    pub mod health;
}

use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

use crate::state::AppState;

/// 全部路由:/health 与 /api/v1/health 同 handler 双挂;/api/v1/openapi.json 供 codegen。
pub fn routes(state: AppState) -> Router {
    let api_v1 = Router::new()
        .route("/health", get(crate::api::v1::health::health_handler))
        .route("/openapi.json", get(openapi_json_handler));

    Router::new()
        .route("/health", get(crate::api::v1::health::health_handler))
        .nest("/api/v1", api_v1)
        .with_state(state)
}

async fn openapi_json_handler() -> axum::response::Response {
    let doc = crate::openapi::openapi_json();
    // RULE-xxx 标注:handler 路径禁止 unwrap/expect(基线 §16.6)。
    // OpenAPI 文档为进程内纯内存数据,序列化失败仅可能来自程序缺陷,无运行期触发条件;
    // 失败时按 RULE-003 返回 INTERNAL_ERROR 信封,不 panic。
    let json = match serde_json::to_value(&doc) {
        Ok(v) => v,
        Err(_) => serde_json::json!({
            "success": false,
            "data": null,
            "errorCode": "INTERNAL_ERROR",
            "message": "OpenAPI 文档序列化失败"
        }),
    };
    (
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        axum::Json(json),
    )
        .into_response()
}
