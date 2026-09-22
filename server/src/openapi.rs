//! utoipa OpenAPI 装配(arch 基线 §6.6 codegen 源)。
//! 每新增端点:在 `paths(...)` 挂 handler,在 `components(schemas(...))` 注册 DTO。

use crate::api::v1::analytics::__path_record_event;
use crate::api::v1::health::__path_health_handler;
use crate::api::v1::modes::__path_list_modes;
use crate::api::v1::plans::__path_active_plan;
use crate::api::v1::plans::__path_generate_plan;
use crate::api::v1::profiles::__path_get_profile;
use crate::api::v1::profiles::__path_save_step;
use crate::api::v1::snapshots::__path_delete_snapshot;
use crate::api::v1::snapshots::__path_export_plan_csv;
use crate::api::v1::snapshots::__path_export_snapshots_csv;
use crate::api::v1::snapshots::__path_list_snapshots;
use crate::api::v1::snapshots::__path_upsert_snapshot;
use crate::dto::analytics::{ClientEventRequest, EventAck};
use crate::dto::mode::{L2ClassView, L2PreviewView, ModeCardView, ModesView};
use crate::dto::plan::{BucketView, GeneratePlanRequest, PlanView};
use crate::dto::profile::{ProfileView, StepRequest};
use crate::dto::snapshot::{
    DeviationView, EmergencyGapView, LatestDeviationsView, SnapshotDeleted,
    SnapshotMutationResponse, SnapshotView, SnapshotsResponse, TrackingSummaryView,
    UpsertSnapshotRequest,
};
use crate::domain::l2::L2Allocation;
use crate::domain::mode::{Credibility, L2Class};
use crate::domain::profile::{DrawdownResponse, Goal, Horizon, IncomeStability};
use crate::domain::{EmergencyStatus, Notice};
use crate::services::health_service::HealthData;
use utoipa::OpenApi;

// 注册面按本文件顶部的规则逐个补。B 期遗留的存量缺口(profiles / plans / modes
// 五个端点已注解未注册)经 E 期 arch-e §3 议定在本期补挂 —— codegen 链路自此完整。
#[derive(OpenApi)]
#[openapi(
    info(title = "wise-wealth API", version = "0.1.0"),
    paths(
        health_handler,
        record_event,
        get_profile,
        save_step,
        generate_plan,
        active_plan,
        list_modes,
        list_snapshots,
        upsert_snapshot,
        delete_snapshot,
        export_snapshots_csv,
        export_plan_csv
    ),
    components(
        schemas(
            HealthData,
            ClientEventRequest,
            EventAck,
            ProfileView,
            StepRequest,
            GeneratePlanRequest,
            PlanView,
            BucketView,
            ModesView,
            ModeCardView,
            L2PreviewView,
            L2ClassView,
            L2Allocation,
            L2Class,
            EmergencyStatus,
            Notice,
            Horizon,
            DrawdownResponse,
            IncomeStability,
            Goal,
            Credibility,
            SnapshotsResponse,
            SnapshotView,
            UpsertSnapshotRequest,
            SnapshotMutationResponse,
            SnapshotDeleted,
            LatestDeviationsView,
            DeviationView,
            TrackingSummaryView,
            EmergencyGapView
        )
    )
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
