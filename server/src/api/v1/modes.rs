//! 模式列表与推荐端点(受保护组)。
//!
//! 一个端点同时给出「有哪些模式」与「主推哪个」:界面上的两张卡与主推标记
//! 来自同一份数据,分两次请求只会让它们有机会不一致。

use axum::extract::State;
use axum::Extension;
use uuid::Uuid;

use crate::api::middleware::CurrentUser;
use crate::api::v1::profiles::to_domain_profile;
use crate::domain::l2;
use crate::dto::mode::{L2ClassView, L2PreviewView, ModeCardView, ModesView};
use crate::error::{ApiOk, AppError};
use crate::repos;
use crate::services::recommend_service;
use crate::state::AppState;

/// GET /api/v1/modes —— 模式列表 + 本次主推 + L2 预览(RULE-006 / RULE-015)。
#[utoipa::path(
    get,
    path = "/api/v1/modes",
    tag = "modes",
    summary = "列出 L1 模式并给出基于当前档案的推荐",
    responses(
        (status = 200, description = "模式列表与推荐", body = crate::error::Envelope<ModesView>),
        (status = 401, description = "未登录或会话过期"),
        (status = 422, description = "问卷尚未完成,无法给出推荐")
    )
)]
pub async fn list_modes(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> Result<ApiOk<ModesView>, AppError> {
    let user_id = Uuid::parse_str(&user.id).map_err(|_| AppError::Unauthorized)?;
    let row = repos::profiles::get(&state.pool, user_id)
        .await?
        .ok_or_else(|| AppError::Validation("请先完成问卷".into()))?;

    if !row.questionnaire_completed {
        return Err(AppError::Validation("请先完成问卷".into()));
    }

    // 档案 → 领域结构;任一必填项解析失败即视为档案不完整
    let profile = to_domain_profile(&row)
        .ok_or_else(|| AppError::Validation("档案不完整,请重新作答问卷".into()))?;

    let recommendation = recommend_service::recommend(&profile, &state.library);
    let recommended_id = recommendation.as_ref().map(|r| r.recommended_id.clone());

    let items = recommend_service::list_modes(&state.library)
        .into_iter()
        .map(|c| ModeCardView {
            is_recommended: recommended_id.as_deref() == Some(c.id.as_str()),
            id: c.id,
            name: c.name,
            tagline: c.tagline,
            credibility: c.credibility,
            fit_for: c.fit_for,
        })
        .collect();

    // 引擎在模式库已校验的前提下不会缺配置;真出错就如实报,不编一个假配置糊过去
    let allocation = l2::match_l2(&profile, &state.library)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("L2 匹配失败: {e}")))?;

    Ok(ApiOk(ModesView {
        items,
        recommended_id,
        recommendation_reason: recommendation.map(|r| r.reason).unwrap_or_default(),
        l2: L2PreviewView {
            name: allocation.name,
            reason: allocation.reason,
            classes: allocation
                .classes
                .into_iter()
                .map(|c| L2ClassView {
                    name: c.name,
                    basis_points: c.basis_points,
                })
                .collect(),
        },
    }))
}
