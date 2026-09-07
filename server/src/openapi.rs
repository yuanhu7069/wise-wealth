//! utoipa OpenAPI 装配(arch 基线 §6.6 codegen 源)。
//! 每新增端点:在 `paths(...)` 挂 handler,在 `components(schemas(...))` 注册 DTO。

use crate::api::v1::health::__path_health_handler;
use crate::services::health_service::HealthData;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "wise-wealth API", version = "0.1.0"),
    paths(health_handler),
    components(schemas(HealthData))
)]
struct ApiDoc;

/// 构建完整 OpenAPI 文档(供 /api/v1/openapi.json 与 gen-types.sh 消费)。
/// utoipa 按 handler 的 `path = "/health"` 注解生成;`/api/v1/health` 与 `/health`
/// 双挂同一 handler,codegen 以 /api/v1/health 为准,故在此补挂同义路径。
pub fn openapi_json() -> utoipa::openapi::OpenApi {
    let mut doc = ApiDoc::openapi();
    if let Some(health) = doc.paths.paths.get("/health").cloned() {
        doc.paths
            .paths
            .insert("/api/v1/health".to_string(), health);
    }
    doc
}
