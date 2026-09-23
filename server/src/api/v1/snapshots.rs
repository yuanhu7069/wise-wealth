//! 快照端点(受保护组, RULE-021 ~ RULE-031):列表+摘要 / 录入覆盖 / 删除 / 双 CSV 导出。
//!
//! 两个导出端点是统一信封的**显式例外**(ADR-E-004):`text/csv` 直出给浏览器下载,
//! 前端不以 JSON 消费它们。

use std::collections::BTreeMap;

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::middleware::CurrentUser;
use crate::domain::csv::render_csv;
use crate::dto::snapshot::{
    self, EmergencyGapView, LatestDeviationsView, SnapshotDeleted, SnapshotMutationResponse,
    SnapshotView, SnapshotsResponse, TrackingSummaryView, UpsertSnapshotRequest,
};
use crate::error::{ApiOk, AppError};
use crate::repos;
use crate::repos::snapshots::SnapshotRow;
use crate::services::analytics_service::{self, Event, ExportKind};
use crate::services::snapshot_service::{self, SnapshotError, TrackingSummary};
use crate::state::AppState;

impl From<SnapshotError> for AppError {
    fn from(e: SnapshotError) -> Self {
        match e {
            // 用户能自己修的:422,文案照实给
            SnapshotError::NoActivePlan
            | SnapshotError::BucketSetMismatch(_)
            | SnapshotError::MonthNotAllowed { .. }
            | SnapshotError::NotLatestMonth => AppError::Validation(e.to_string()),
            SnapshotError::CorruptedSnapshot(_) => {
                AppError::Internal(anyhow::anyhow!("{e}"))
            }
            SnapshotError::Db(inner) => AppError::Database(inner),
        }
    }
}

/// repo 行 → 视图。版本号随行携带(list/upsert 查询已 JOIN 或冻结),
/// 不再逐行 by_id —— 每页 24 行 × 12 列方案的 N+1(评审发现 #9)。
fn to_view(row: &SnapshotRow) -> Result<SnapshotView, AppError> {
    let balances: BTreeMap<String, i64> = serde_json::from_value(row.balances.clone())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("快照余额对象损坏: {e}")))?;
    Ok(SnapshotView {
        id: row.id,
        month: row.month.clone(),
        plan_version: row.plan_version,
        balances,
        special_month: row.special_month,
    })
}

/// 域摘要 → API 摘要视图。
fn to_summary_view(summary: TrackingSummary) -> TrackingSummaryView {
    TrackingSummaryView {
        persisted_months: summary.persisted_months,
        latest: summary.latest.map(|l| LatestDeviationsView {
            month: l.month,
            deviations: l
                .deviations
                .map(|ds| ds.iter().map(Into::into).collect()),
        }),
        emergency: summary.emergency.map(|e| EmergencyGapView {
            target_cents: e.target_cents,
            balance_cents: e.balance_cents,
            gap_cents: e.gap.gap_cents,
            gap_months_tenths: e.gap.gap_months_tenths,
            met: e.gap.met,
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    /// 每页条数,缺省 24(红线 11:列表必须分页)
    limit: Option<i64>,
    offset: Option<i64>,
}

/// GET /api/v1/snapshots —— 历史快照(倒序分页)+ 追踪摘要。
#[utoipa::path(
    get,
    path = "/api/v1/snapshots",
    tag = "snapshots",
    summary = "历史快照与追踪摘要(已坚持月数 / 应急缺口 / 最新偏离)",
    params(("limit" = Option<i64>, Query, description = "每页条数,缺省 24"), ("offset" = Option<i64>, Query, description = "偏移")),
    responses(
        (status = 200, description = "快照列表与摘要", body = crate::error::Envelope<SnapshotsResponse>),
        (status = 401, description = "未登录或会话过期")
    )
)]
pub async fn list_snapshots(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Query(q): Query<ListQuery>,
) -> Result<ApiOk<SnapshotsResponse>, AppError> {
    let user_id = parse_user_id(&user)?;
    let limit = q.limit.unwrap_or(24).clamp(1, 120);
    let offset = q.offset.unwrap_or(0).max(0);

    let rows = repos::snapshots::list_desc(&state.pool, user_id, limit, offset).await?;
    let items: Vec<SnapshotView> = rows.iter().map(to_view).collect::<Result<_, _>>()?;
    let summary = snapshot_service::summary(
        &state.pool,
        &state.library,
        state.config.snapshot_deviation_threshold_bp,
        user_id,
    )
    .await?;

    Ok(ApiOk(SnapshotsResponse {
        items,
        summary: to_summary_view(summary),
        // 当前自然月的**后端权威值**(评审发现 #4):前端不再各算各的月份,
        // 打卡目标月与「本月还没打卡」判定以此为准
        current_month: snapshot_service::current_month(),
    }))
}

/// PUT /api/v1/snapshots/{month} —— 录入或覆盖**当月**快照(RULE-021/028:历史月不可变)。
#[utoipa::path(
    put,
    path = "/api/v1/snapshots/{month}",
    tag = "snapshots",
    summary = "录入/覆盖当月快照(仅限当前自然月;全桶必填;同月再提交为覆盖)",
    params(("month" = String, Path, description = "自然月 YYYY-MM,必须等于服务器当前月")),
    request_body = UpsertSnapshotRequest,
    responses(
        (status = 200, description = "落库后的快照与偏离结论", body = crate::error::Envelope<SnapshotMutationResponse>),
        (status = 401, description = "未登录或会话过期"),
        (status = 422, description = "金额/月份/桶集合校验失败,或还没有方案")
    )
)]
pub async fn upsert_snapshot(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(month): Path<String>,
    Json(req): Json<UpsertSnapshotRequest>,
) -> Result<ApiOk<SnapshotMutationResponse>, AppError> {
    let user_id = parse_user_id(&user)?;
    snapshot::validate_month(&month).map_err(AppError::Validation)?;

    // 元字符串 → 分(RULE-022);哪个桶不合法就说哪个桶
    let mut balances: BTreeMap<String, i64> = BTreeMap::new();
    for (bucket_id, raw) in &req.balances {
        let cents = snapshot::parse_yuan_to_cents(raw)
            .map_err(|msg| AppError::Validation(format!("{bucket_id}:{msg}")))?;
        balances.insert(bucket_id.clone(), cents);
    }

    let (row, inserted, deviations) = snapshot_service::upsert(
        &state.pool,
        state.config.snapshot_deviation_threshold_bp,
        user_id,
        &month,
        balances,
        req.special_month,
    )
    .await?;

    // 埋点(prd-e §9.5):首次快照率核心事件;载荷只有两个布尔,金额不入埋点
    analytics_service::record(
        &state.pool,
        &Event::SnapshotSubmit {
            is_overwrite: !inserted,
            is_special: req.special_month,
        },
    )
    .await;
    tracing::info!(
        user = %user.username,
        month = %month,
        overwrite = !inserted,
        "快照已记录(金额不入日志)"
    );

    Ok(ApiOk(SnapshotMutationResponse {
        snapshot: to_view(&row)?,
        deviations: deviations.map(|ds| ds.iter().map(Into::into).collect()),
    }))
}

/// DELETE /api/v1/snapshots/{month} —— 删除快照(RULE-029:仅最新月;录错恢复口)。
#[utoipa::path(
    delete,
    path = "/api/v1/snapshots/{month}",
    tag = "snapshots",
    summary = "删除某月快照(仅限最新月;删除后该月回到跳过月)",
    params(("month" = String, Path, description = "自然月 YYYY-MM")),
    responses(
        (status = 200, description = "已删除", body = crate::error::Envelope<SnapshotDeleted>),
        (status = 401, description = "未登录或会话过期"),
        (status = 422, description = "目标不是最新月(历史月不可删)")
    )
)]
pub async fn delete_snapshot(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(month): Path<String>,
) -> Result<ApiOk<SnapshotDeleted>, AppError> {
    let user_id = parse_user_id(&user)?;
    snapshot::validate_month(&month).map_err(AppError::Validation)?;

    snapshot_service::delete_month(&state.pool, user_id, &month).await?;
    analytics_service::record(&state.pool, &Event::SnapshotDelete).await;
    tracing::info!(user = %user.username, month = %month, "快照已删除");

    Ok(ApiOk(SnapshotDeleted { deleted: true }))
}

/// CSV 响应组装(ADR-E-004:统一信封的显式例外)。
fn csv_response(filename: &str, header: &[&str], rows: Vec<Vec<String>>) -> Response {
    let body = render_csv(header, &rows);
    (
        [
            (axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                axum::http::header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        body,
    )
        .into_response()
}

/// 文件名里的导出日期(本地时区;文件名只是给人看的,精确到日即可)。
fn export_date() -> String {
    chrono::Local::now().format("%Y%m%d").to_string()
}

/// GET /api/v1/snapshots/export —— 快照长表 CSV(RULE-030)。
#[utoipa::path(
    get,
    path = "/api/v1/snapshots/export",
    tag = "snapshots",
    summary = "导出全部快照(CSV 长表:月 × 桶一行;UTF-8 带 BOM)",
    responses(
        (status = 200, description = "CSV 文件下载", content_type = "text/csv", body = String),
        (status = 401, description = "未登录或会话过期")
    )
)]
pub async fn export_snapshots_csv(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Response, AppError> {
    let user_id = parse_user_id(&user)?;
    let (header, rows) = snapshot_service::export_csv_rows(&state.pool, user_id).await?;
    analytics_service::record(
        &state.pool,
        &Event::ExportCsv {
            export_type: ExportKind::Snapshot,
        },
    )
    .await;
    Ok(csv_response(
        &format!("snapshot_export_{}.csv", export_date()),
        &header,
        rows,
    ))
}

/// GET /api/v1/plans/active/export —— 当前方案 CSV(RULE-030;P05 导出区第二颗按钮)。
#[utoipa::path(
    get,
    path = "/api/v1/plans/active/export",
    tag = "snapshots",
    summary = "导出当前方案与各桶明细(CSV;UTF-8 带 BOM)",
    responses(
        (status = 200, description = "CSV 文件下载", content_type = "text/csv", body = String),
        (status = 401, description = "未登录或会话过期"),
        (status = 404, description = "还没有生成过方案")
    )
)]
pub async fn export_plan_csv(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<Response, AppError> {
    let user_id = parse_user_id(&user)?;
    let plan = repos::plans::active_for_user(&state.pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("还没有生成过方案".into()))?;
    let buckets = repos::plans::buckets_of(&state.pool, plan.id).await?;
    let l1_mode_name = state
        .library
        .mode(&plan.l1_mode)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| plan.l1_mode.clone());
    let l2_mode_name = state
        .library
        .l2(&plan.l2_mode)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| plan.l2_mode.clone());

    let rows: Vec<Vec<String>> = buckets
        .iter()
        .map(|b| {
            vec![
                plan.version.to_string(),
                l1_mode_name.clone(),
                l2_mode_name.clone(),
                plan.created_date.clone(),
                b.bucket_id.clone(),
                b.name.clone(),
                crate::domain::csv::yuan_string(b.amount_monthly_cents),
                b.target_cents
                    .map(crate::domain::csv::yuan_string)
                    .unwrap_or_default(),
            ]
        })
        .collect();

    analytics_service::record(
        &state.pool,
        &Event::ExportCsv { export_type: ExportKind::Plan },
    )
    .await;
    Ok(csv_response(
        &format!("plan_export_{}.csv", export_date()),
        &[
            "方案版本",
            "L1模式",
            "L2配置",
            "生成日期",
            "桶ID",
            "桶名",
            "月转入(元)",
            "目标金额(元)",
        ],
        rows,
    ))
}

/// 会话里的用户 id 是 UUID 字符串(与 profiles/plans 同一解析纪律)。
fn parse_user_id(user: &CurrentUser) -> Result<Uuid, AppError> {
    Uuid::parse_str(&user.id).map_err(|_| AppError::Unauthorized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let vars = std::collections::HashMap::from([
            ("APP_ENV".to_string(), "dev".to_string()),
            ("APP_PORT".to_string(), "8080".to_string()),
            (
                "DATABASE_URL_DEV".to_string(),
                "postgres://u:p@127.0.0.1:5432/wise_wealth_dev".to_string(),
            ),
            (
                "JWT_SECRET".to_string(),
                "test-secret-at-least-32-characters-long".to_string(),
            ),
        ]);
        let config = crate::config::Config::from_env(vars).expect("测试配置应合法");
        let library = crate::domain::ModeLibrary::load_embedded().unwrap();
        AppState::new(&config, library).expect("惰性连接池不应失败")
    }

    fn app() -> axum::Router {
        crate::api::routes(test_state())
    }

    fn cookie() -> String {
        let token = crate::services::auth_service::issue_token(
            "test-secret-at-least-32-characters-long",
            "11111111-1111-7111-8111-111111111111",
            "苑问",
            30,
        )
        .unwrap();
        format!("ww_session={token}")
    }

    // ── 鉴权边界:五个新端点全部落在受保护组,无 Cookie 一律 401(工单 02 DoD)──

    #[tokio::test]
    async fn 新端点无_cookie_一律_401_信封() {
        let cases = [
            ("GET", "/api/v1/snapshots", None),
            ("PUT", "/api/v1/snapshots/2026-09", Some(r#"{"balances":{}}"#)),
            ("DELETE", "/api/v1/snapshots/2026-09", None),
            ("GET", "/api/v1/snapshots/export", None),
            ("GET", "/api/v1/plans/active/export", None),
        ];
        for (method, uri, body) in cases {
            let req = Request::builder()
                .method(method)
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json");
            let req = match body {
                Some(b) => req.body(Body::from(b.to_string())).unwrap(),
                None => req.body(Body::empty()).unwrap(),
            };
            let resp = app().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED, "{method} {uri}");
            let bytes = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
            let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(json["errorCode"], "UNAUTHORIZED", "{method} {uri}");
        }
    }

    // ── 未来月份拒绝(RULE-022 边界;发生在任何数据库访问之前)──

    #[tokio::test]
    async fn 未来月份被_422_拒绝且不触库() {
        let req = Request::builder()
            .method("PUT")
            .uri("/api/v1/snapshots/2099-01")
            .header(header::COOKIE, cookie())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"balances": {"reserve": "100.00"}}"#))
            .unwrap();
        let resp = app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let bytes = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["errorCode"], "VALIDATION_ERROR");
        assert!(
            json["message"].as_str().unwrap().contains("2099-01"),
            "文案应点名被拒的月份: {json}"
        );
    }

    // ── 月份格式(手滑防线;同在触库之前)──

    #[tokio::test]
    async fn 非法月份格式被_422_拒绝() {
        let req = Request::builder()
            .method("DELETE")
            .uri("/api/v1/snapshots/2099-13")
            .header(header::COOKIE, cookie())
            .body(Body::empty())
            .unwrap();
        let resp = app().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
