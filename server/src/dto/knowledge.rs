//! 知识库 DTO(G 期 RULE-038):列表只给元数据,详情才带 sections ——
//! P07 板块卡与全文阅读态分两次取数,列表响应不背全量正文。

use serde::Serialize;
use utoipa::ToSchema;

use crate::domain::knowledge::{KnowledgeArticle, KnowledgeKind, Section};
use crate::domain::Credibility;

/// 文章元数据(列表项;不含正文)。
#[derive(Debug, Serialize, ToSchema)]
pub struct KnowledgeListItemView {
    /// 全库唯一 id(深链 `/knowledge?id=`)
    pub id: String,
    /// 板块
    pub kind: KnowledgeKind,
    /// 标题
    pub title: String,
    /// 关联的 L1 模式 id(仅解读类携带)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_mode: Option<String>,
    /// 一句话导语
    pub summary: String,
    /// 关联模式可信度(仅解读类有值,服务端从模式库富化;卡片徽章直接用)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credibility: Option<Credibility>,
}

/// 一节正文。
#[derive(Debug, Serialize, ToSchema)]
pub struct SectionView {
    /// 小标题
    pub heading: String,
    /// 段落列表
    pub paragraphs: Vec<String>,
}

/// 文章全文(详情)。
#[derive(Debug, Serialize, ToSchema)]
pub struct KnowledgeArticleView {
    pub id: String,
    pub kind: KnowledgeKind,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_mode: Option<String>,
    pub summary: String,
    /// 关联模式可信度(仅解读类有值,详情页徽章直接用)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credibility: Option<Credibility>,
    pub sections: Vec<SectionView>,
}

/// GET /api/v1/knowledge 的响应。
#[derive(Debug, Serialize, ToSchema)]
pub struct KnowledgeListView {
    /// 全部文章(按配置目录顺序;前端按 kind 分组)
    pub items: Vec<KnowledgeListItemView>,
}

impl KnowledgeListItemView {
    /// 元数据 + 关联模式可信度富化(解读类;service 装配时传模式库取值函数)。
    pub fn of(a: &KnowledgeArticle, credibility_of: impl Fn(&str) -> Option<Credibility>) -> Self {
        Self {
            id: a.id.clone(),
            kind: a.kind,
            title: a.title.clone(),
            related_mode: a.related_mode.clone(),
            summary: a.summary.clone(),
            credibility: a.related_mode.as_deref().and_then(credibility_of),
        }
    }
}

/// 详情装配:附上关联模式的可信度(解读类),供详情页徽章。
pub fn article_view(a: &KnowledgeArticle, credibility_of: impl Fn(&str) -> Option<Credibility>) -> KnowledgeArticleView {
    let credibility = a.related_mode.as_deref().and_then(credibility_of);
    KnowledgeArticleView {
        id: a.id.clone(),
        kind: a.kind,
        title: a.title.clone(),
        related_mode: a.related_mode.clone(),
        summary: a.summary.clone(),
        credibility,
        sections: a
            .sections
            .iter()
            .map(|s: &Section| SectionView {
                heading: s.heading.clone(),
                paragraphs: s.paragraphs.clone(),
            })
            .collect(),
    }
}
