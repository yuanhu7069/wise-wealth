//! utoipa OpenAPI 装配(arch 基线 §6.6 codegen 源)。
//! 每新增端点:在 `paths(...)` 挂 handler,在 `components(schemas(...))` 注册 DTO。

use crate::api::v1::analytics::__path_record_event;
use crate::api::v1::health::__path_health_handler;
use crate::dto::analytics::{ClientEventRequest, EventAck};
use crate::services::health_service::HealthData;
use utoipa::OpenApi;

// 注册面按本文件顶部的规则逐个补:本 ticket 新增的埋点端点挂上,
// 04-06 落地的业务端点(profiles / plans / modes)尚未注册,是已知的存量缺口
// (见 ticket 07 完成记录的「遗留」),不在这里顺带补 —— 那是另一件事。
#[derive(OpenApi)]
#[openapi(
    info(title = "wise-wealth API", version = "0.1.0"),
    paths(health_handler, record_event),
    components(schemas(HealthData, ClientEventRequest, EventAck))
)]
struct ApiDoc;

/// 构建完整 OpenAPI 文档(供 /api/v1/openapi.json 与 gen-types.sh 消费)。
/// utoipa 按 handler 的 `path = "/health"` 注解生成;`/api/v1/health` 与 `/health`
/// 双挂同一 handler,codegen 以 /api/v1/health 为准,故在此补挂同义路径。
pub fn openapi_json() -> utoipa::openapi::OpenApi {
    let mut doc = ApiDoc::openapi();
    if let Some(health) = doc.paths.paths.get("/health").cloned() {
        // 镜像路径的 operationId 必须与原路径不同——否则 openapi-typescript 会在
        // operations 接口里生成重名成员(TS2300)。
        let mut mirror = health;
        if let Some(op) = mirror.get.as_mut() {
            op.operation_id = Some("health_handler_alias".to_string());
        }
        doc.paths
            .paths
            .insert("/api/v1/health".to_string(), mirror);
    }
    doc
}
