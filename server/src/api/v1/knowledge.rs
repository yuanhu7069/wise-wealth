//! 知识库端点(受保护组,G 期):列表(元数据)+ 详情(全文)。
//!
//! 列表不含 sections:板块卡只需要导语,正文随阅读态按 id 二次取 ——
//! 同一次会话里「不读的文章」不会被反复推给浏览器。

use axum::extract::{Path, State};
use axum::Extension;

use crate::api::middleware::CurrentUser;
use crate::dto::knowledge::{article_view, KnowledgeArticleView, KnowledgeListView, KnowledgeListItemView};
use crate::error::{ApiOk, AppError};
use crate::state::AppState;

/// GET /api/v1/knowledge —— 文章元数据列表(RULE-038;前端按 kind 分组)。
#[utoipa::path(
    get,
    path = "/api/v1/knowledge",
    tag = "knowledge",
    summary = "列出知识库文章元数据(不含正文)",
    responses(
        (status = 200, description = "文章列表", body = crate::error::Envelope<KnowledgeListView>),
        (status = 401, description = "未登录或会话过期")
    )
)]
pub async fn list_knowledge(
    State(state): State<AppState>,
    Extension(_user): Extension<CurrentUser>,
) -> Result<ApiOk<KnowledgeListView>, AppError> {
    let items = state
        .knowledge
        .articles()
        .iter()
        .map(KnowledgeListItemView::from)
        .collect();
    Ok(ApiOk(KnowledgeListView { items }))
}

/// GET /api/v1/knowledge/:id —— 文章全文;未知 id → 404(AC-12)。
#[utoipa::path(
    get,
    path = "/api/v1/knowledge/{id}",
    tag = "knowledge",
    summary = "读取知识库文章全文",
    params(("id" = String, Path, description = "文章 id")),
    responses(
        (status = 200, description = "文章全文", body = crate::error::Envelope<KnowledgeArticleView>),
        (status = 401, description = "未登录或会话过期"),
        (status = 404, description = "文章不存在")
    )
)]
pub async fn get_knowledge(
    State(state): State<AppState>,
    Extension(_user): Extension<CurrentUser>,
    Path(id): Path<String>,
) -> Result<ApiOk<KnowledgeArticleView>, AppError> {
    let article = state
        .knowledge
        .article(&id)
        .ok_or_else(|| AppError::NotFound("文章不存在".into()))?;
    let view = article_view(article, |mode_id| {
        state.library.mode(mode_id).map(|m| m.credibility)
    });
    Ok(ApiOk(view))
}
