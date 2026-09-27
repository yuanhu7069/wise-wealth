//! 方案端点(受保护组):生成方案 / 读当前方案(RULE-011)。

use axum::extract::State;
use axum::{Extension, Json};
use uuid::Uuid;

use crate::api::middleware::CurrentUser;
use crate::api::v1::profiles::to_domain_profile;
use crate::domain::Tier;
use crate::domain::engine::{EmergencyStatus, Notice, Trace};
use crate::domain::l2::L2Allocation;
use crate::domain::profile::Profile;
use crate::domain::{ModeConfig, ModeLibrary};
use crate::dto::mode::BucketOverviewView;
use crate::dto::plan::{
    BucketView, GeneratePlanRequest, PlanEntry, PlanView, PreviewBucketView, PreviewItemView,
    PreviewRequest, PreviewResponse, PreviewSolutionView, TraceView,
};
use crate::error::{ApiOk, AppError};
use crate::repos;
use crate::repos::plans::{BucketRow, PlanRecord};
use crate::services::analytics_service::{self, Event};
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
///
/// `traces` 是随方案冻结的推理链:条数恒下发,内容**仅 plus 会话**装配
/// (ADR-H-001 服务侧闸门 —— free 分支根本不把链条放响应体,而非前端遮罩)。
fn to_view(
    state: &AppState,
    plan: &PlanRecord,
    buckets: Vec<BucketRow>,
    tier: Tier,
) -> Result<PlanView, AppError> {
    let l2: L2Allocation = serde_json::from_value(plan.l2_allocation.clone())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("L2 快照损坏: {e}")))?;
    let emergency: EmergencyStatus = serde_json::from_value(plan.emergency.clone())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("应急金快照损坏: {e}")))?;
    let notices: Vec<Notice> = serde_json::from_value(plan.notices.clone())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("提示快照损坏: {e}")))?;
    let parsed_traces: Vec<Trace> = serde_json::from_value(plan.traces.clone())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("推理链快照损坏: {e}")))?;
    let trace_count = parsed_traces.len();
    let traces = if tier.is_plus() {
        Some(parsed_traces.into_iter().map(TraceView::from).collect())
    } else {
        // RULE-045:free 响应无内容可偷。这里的 None 是「不给」,不是「没有」。
        None
    };

    let l1_mode_name = state
        .library
        .mode(&plan.l1_mode)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| plan.l1_mode.clone());

    // 可信度/出处按读取时解析(ADR-F-002):评级描述模式的知识状态,随配置更新;
    // 模式已下架 → None,前端不渲染提示条(降级安全)
    let mode_config = state.library.mode(&plan.l1_mode);
    let l1_credibility = mode_config.map(|m| m.credibility);
    let l1_source = mode_config.and_then(|m| m.source.clone());

    Ok(PlanView {
        id: plan.id,
        version: plan.version,
        created_date: plan.created_date.clone(),
        l1_mode: plan.l1_mode.clone(),
        l1_mode_name,
        l1_credibility,
        l1_source,
        investable_monthly_cents: plan.investable_monthly_cents,
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
        trace_count,
        traces,
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

    // 埋点(prd-v1 §9.5 / prd-f §9.5):生成率、模式偏好与入口分布。
    // 载荷只带模式 id、版本号与入口枚举,金额不入埋点。
    analytics_service::record(
        &state.pool,
        &Event::PlanGenerated {
            l1_mode: generated.plan.l1_mode.clone(),
            plan_version: generated.plan.version,
            entry: req.entry.unwrap_or(PlanEntry::Questionnaire),
        },
    )
    .await;

    // 分层实时读取(RULE-046):生成响应与读取响应同一闸门口径
    let tier = repos::users::tier_of(&state.pool, user_id).await?;
    let view = to_view(&state, &generated.plan, generated.buckets, tier)?;
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
    // 分层实时读取(ADR-H-001/RULE-046):置 plus 后无需重新生成,下一个请求即得全链
    let tier = repos::users::tier_of(&state.pool, user_id).await?;

    Ok(ApiOk(to_view(&state, &plan, buckets, tier)?))
}

/// 试算条目装配(纯函数,可对账单测):同档案同引擎,输出与真实生成逐字段同源(ADR-G-002)。
fn build_preview_item(
    profile: &Profile,
    mode: &ModeConfig,
    library: &ModeLibrary,
) -> PreviewItemView {
    let buckets_meta = mode
        .buckets
        .iter()
        .map(|b| BucketOverviewView {
            name: b.name.clone(),
            purpose: b.purpose.clone(),
            share_desc: b.share_desc(),
        })
        .collect();

    match crate::domain::solve(profile, mode, library) {
        Ok(r) => PreviewItemView {
            mode_id: mode.id.clone(),
            name: mode.name.clone(),
            credibility: mode.credibility,
            source: mode.source.clone(),
            tagline: mode.tagline.clone(),
            fit_for: mode.fit_for.clone(),
            buckets_meta,
            solution: Some(PreviewSolutionView {
                buckets: r
                    .buckets
                    .iter()
                    .map(|b| PreviewBucketView {
                        name: b.name.clone(),
                        amount_monthly_cents: b.amount_monthly_cents,
                    })
                    .collect(),
                necessary_monthly_cents: r.emergency.necessary_monthly_cents,
                investable_monthly_cents: r
                    .buckets
                    .iter()
                    .find(|b| Some(b.id.as_str()) == r.l2_bucket_id.as_deref())
                    .map(|b| b.amount_monthly_cents)
                    .unwrap_or(0),
                l2_name: r.l2.name.clone(),
                notices: r.notices,
                emergency_met: r.emergency.is_met,
                emergency_months_to_fill: r.emergency.months_to_fill,
            }),
            reason: None,
        },
        Err(e) => PreviewItemView {
            mode_id: mode.id.clone(),
            name: mode.name.clone(),
            credibility: mode.credibility,
            source: mode.source.clone(),
            tagline: mode.tagline.clone(),
            fit_for: mode.fit_for.clone(),
            buckets_meta,
            solution: None,
            reason: Some(e.to_string()),
        },
    }
}

/// POST /api/v1/plans/preview —— 试算:对 ≤3 个模式以当前档案只读跑引擎(RULE-041/042)。
///
/// 不落库、不产生版本、不改 active、不影响推荐 —— 与生成的唯一共享是 solve 纯函数本身。
/// POST 用于计算是记录在案的显式例外(ADR-G-002):请求体携带模式列表,GET 语义别扭。
#[utoipa::path(
    post,
    path = "/api/v1/plans/preview",
    tag = "plans",
    summary = "按当前档案试算若干模式(不落库)",
    request_body = PreviewRequest,
    responses(
        (status = 200, description = "逐模式试算结果", body = crate::error::Envelope<PreviewResponse>),
        (status = 401, description = "未登录或会话过期"),
        (status = 422, description = "问卷未完成 / 模式数量或 id 非法")
    )
)]
pub async fn preview_plans(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<PreviewRequest>,
) -> Result<ApiOk<PreviewResponse>, AppError> {
    let user_id = Uuid::parse_str(&user.id).map_err(|_| AppError::Unauthorized)?;
    if req.mode_ids.is_empty() || req.mode_ids.len() > 3 {
        return Err(AppError::Validation("最多对比 3 个模式,至少选 1 个".into()));
    }

    let row = repos::profiles::get(&state.pool, user_id)
        .await?
        .ok_or(PlanError::ProfileIncomplete)?;
    if !row.is_complete() {
        return Err(PlanError::ProfileIncomplete.into());
    }
    let profile = to_domain_profile(&row).ok_or(PlanError::ProfileIncomplete)?;

    let mut items = Vec::with_capacity(req.mode_ids.len());
    for id in &req.mode_ids {
        let mode = state
            .library
            .mode(id)
            .ok_or_else(|| AppError::Validation(format!("未知模式:{id}")))?;
        items.push(build_preview_item(&profile, mode, &state.library));
    }
    Ok(ApiOk(PreviewResponse { items }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::{DrawdownResponse, Goal, Horizon, IncomeStability};

    /// 与 engine.rs 金例 A 同档案:四账户口径已在那里锁定,这里锁「preview == 生成」。
    fn profile_a() -> Profile {
        Profile {
            horizon: Horizon::Y5to10,
            drawdown_response: DrawdownResponse::Hold,
            income_stability: IncomeStability::Volatile,
            dependents: 1,
            inflow_cents: 1_200_000,
            expense_fixed_monthly_cents: 450_000,
            savings_cents: 2_400_000,
            goal: Goal::Wealth,
        }
    }

    #[test]
    fn 试算与生成逐字段一致_对账() {
        // AC-6 的自动化底座:preview 装配走的就是 solve,这里锁死「装配不改数字」
        let lib = crate::domain::ModeLibrary::load_embedded().unwrap();
        let p = profile_a();
        for mode_id in ["four_accounts", "fifty_30_20", "snp_quadrant", "four_pots"] {
            let mode = lib.mode(mode_id).unwrap();
            let direct = crate::domain::solve(&p, mode, &lib).unwrap();
            let item = build_preview_item(&p, mode, &lib);
            let s = item.solution.as_ref().expect("金例档案应可求解");

            assert_eq!(s.buckets.len(), direct.buckets.len(), "{mode_id} 桶数");
            for (pb, db) in s.buckets.iter().zip(&direct.buckets) {
                assert_eq!(pb.name, db.name, "{mode_id} 桶名");
                assert_eq!(
                    pb.amount_monthly_cents, db.amount_monthly_cents,
                    "{mode_id} 桶金额"
                );
            }
            assert_eq!(s.necessary_monthly_cents, direct.emergency.necessary_monthly_cents);
            assert_eq!(s.emergency_met, direct.emergency.is_met);
            assert_eq!(s.notices, direct.notices);
            let invest_direct = direct
                .buckets
                .iter()
                .find(|b| Some(b.id.as_str()) == direct.l2_bucket_id.as_deref())
                .map(|b| b.amount_monthly_cents)
                .unwrap_or(0);
            assert_eq!(s.investable_monthly_cents, invest_direct, "{mode_id} 投资桶");
        }
    }

    #[test]
    fn 试算条目携带元数据_含出处与桶口径() {
        let lib = crate::domain::ModeLibrary::load_embedded().unwrap();
        let p = profile_a();
        let snp = lib.mode("snp_quadrant").unwrap();
        let item = build_preview_item(&p, snp, &lib);
        assert_eq!(item.credibility, crate::domain::Credibility::Disputed);
        assert!(item.source.as_deref().unwrap_or("").contains("从未发布"));
        assert_eq!(item.buckets_meta.len(), 4);
        assert!(item.buckets_meta.iter().all(|b| !b.share_desc.is_empty()));
    }

    // ── H 期:推理链闸门(ADR-H-001/RULE-045;to_view 纯函数,AppState 只用模式库)──

    fn plan_record_with_traces(traces: &[crate::domain::Trace]) -> PlanRecord {
        PlanRecord {
            id: uuid::Uuid::now_v7(),
            l1_mode: "four_accounts".into(),
            l2_mode: "sixty_forty".into(),
            version: 1,
            is_active: true,
            investable_monthly_cents: 80_000,
            created_date: "2026-09-27".into(),
            profile_snapshot: serde_json::Value::Null,
            l2_allocation: serde_json::json!({
                "id": "sixty_forty", "name": "60/40",
                "classes": [{"name": "权益类", "basis_points": 6000}, {"name": "债券类", "basis_points": 4000}],
                "note": null,
                "reason": "久期 5-10 年,回撤反应 = 不动 → 60/40"
            }),
            emergency: serde_json::json!({
                "months": 9, "necessary_monthly_cents": 1_080_000, "target_cents": 9_720_000,
                "gap_cents": 7_320_000, "monthly_toward_emergency_cents": 310_000,
                "months_to_fill": 31, "coverage_tenths": 22, "surplus_cents": 0, "is_met": false
            }),
            notices: serde_json::json!([]),
            traces: serde_json::to_value(traces).unwrap(),
        }
    }

    #[tokio::test]
    async fn 闸门双向_free_响应无_traces_键_plus_全量() {
        let state = crate::api::tests_support::state();
        let trace = crate::domain::Trace {
            rule_id: "emergency_fund_months".into(),
            output: 9,
            unit: crate::domain::TraceUnit::Months,
            rationale: "收入稳定性=波动大 → 9 个月".into(),
        };
        let plan = plan_record_with_traces(std::slice::from_ref(&trace));

        for (tier, expect_key) in [(Tier::Free, false), (Tier::Plus, true)] {
            let view = to_view(&state, &plan, vec![], tier).expect("视图装配不应失败");
            assert_eq!(view.trace_count, 1, "{tier:?}:条数恒下发");
            let json = serde_json::to_value(&view).expect("序列化不应失败");
            assert_eq!(
                json.get("traces").is_some(),
                expect_key,
                "{tier:?}:traces 键{}存在",
                if expect_key { "必须" } else { "不得" }
            );
            if expect_key {
                assert_eq!(json["traces"][0]["rule_id"], "emergency_fund_months");
            }
        }
    }
}
