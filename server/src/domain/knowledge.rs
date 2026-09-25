//! 知识库:配置驱动的文章内容(ADR-G-001)。
//!
//! 与模式库同一立场:「**加一篇内容 = 加一个 TOML 文件,不改代码**」。构建期由
//! `build.rs` 扫描 `config/knowledge/` 并 `include_str!` 内嵌,启动期解析并校验 ——
//! 坏内容(缺局限性、指向不存在的模式)在启动期就暴露,而不是等用户读到一半。
//!
//! 内容与引擎严格分离(RULE-040):`non_implementable` 条目永远只是文章,
//! 不会出现在模式列表、推荐与试算里。

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::mode::ModeLibrary;

// 配置来源清单由构建期扫描 `config/knowledge/` 生成(见 `build.rs`)。
// 用行注释而非文档注释:rustdoc 不会给宏调用生成文档。
include!(concat!(env!("OUT_DIR"), "/knowledge_sources.rs"));

/// 文章板块(总 PRD §4.5 四板块;「对比文章」不做,见 prd-g OUT-001)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeKind {
    /// 模式解读:每模式一篇,四节(局限性必写,RULE-039)
    ModeInterpretation,
    /// 出处考据:disputed 模式的深挖文章
    Verification,
    /// 理财百科:术语词条
    Encyclopedia,
    /// 不可落地专区:仅供理解,不进引擎(RULE-040)
    NonImplementable,
}

impl KnowledgeKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            KnowledgeKind::ModeInterpretation => "mode_interpretation",
            KnowledgeKind::Verification => "verification",
            KnowledgeKind::Encyclopedia => "encyclopedia",
            KnowledgeKind::NonImplementable => "non_implementable",
        }
    }
}

/// 文章的一节:小标题 + 若干段落。前端按此结构排版(ADR-G-001:不用 markdown 渲染器)。
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct Section {
    /// 小标题(如「局限性」「为什么不建议照搬」)
    pub heading: String,
    /// 段落列表;空段落在装载期被拒(防手滑留下半截排版)
    pub paragraphs: Vec<String>,
}

/// 一篇知识文章。
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct KnowledgeArticle {
    /// 全库唯一标识(如 `ki-four-accounts`)
    pub id: String,
    /// 板块
    pub kind: KnowledgeKind,
    /// 标题
    pub title: String,
    /// 关联的 L1 模式 id;**仅 mode_interpretation 允许携带**(RULE-039)
    #[serde(default)]
    pub related_mode: Option<String>,
    /// 一句话导语(卡片上展示)
    pub summary: String,
    /// 正文小节
    pub sections: Vec<Section>,
}

/// 知识库装载错误。启动期即失败(内容是编译期资产,没有降级一说)。
#[derive(Debug, thiserror::Error)]
pub enum KnowledgeError {
    /// TOML 解析失败
    #[error("知识内容解析失败({id}): {detail}")]
    Parse {
        /// 出错的配置来源
        id: String,
        /// 解析器给出的细节
        detail: String,
    },
    /// 内容非法(校验不通过)
    #[error("知识内容非法({id}): {reason}")]
    Invalid {
        /// 出错的配置来源
        id: String,
        /// 违反的约束
        reason: String,
    },
}

/// 知识库:全部文章的集合。
#[derive(Debug, Clone, Default)]
pub struct KnowledgeLibrary {
    articles: Vec<KnowledgeArticle>,
}

impl KnowledgeLibrary {
    /// 装载内嵌配置。启动期调用一次(需先有模式库,校验 related_mode 用);失败即启动失败。
    pub fn load_embedded(mode_library: &ModeLibrary) -> Result<Self, KnowledgeError> {
        Self::from_sources(EMBEDDED_KNOWLEDGE, mode_library)
    }

    /// 从给定文本装载(供测试注入自造内容)。
    pub fn from_sources(
        sources: &[(&str, &str)],
        mode_library: &ModeLibrary,
    ) -> Result<Self, KnowledgeError> {
        let mut articles = Vec::with_capacity(sources.len());
        let mut seen_ids = HashSet::new();
        for (id, text) in sources {
            let article: KnowledgeArticle = toml::from_str(text).map_err(|e| KnowledgeError::Parse {
                id: (*id).to_string(),
                detail: e.to_string(),
            })?;
            validate(&article, mode_library)?;
            if !seen_ids.insert(article.id.clone()) {
                return Err(KnowledgeError::Invalid {
                    id: article.id.clone(),
                    reason: "文章 id 重复 —— id 是深链与 API 的唯一键".into(),
                });
            }
            articles.push(article);
        }
        Ok(Self { articles })
    }

    /// 全部文章(排列顺序 = 配置目录扫描顺序,前端按 kind 分组)。
    pub fn articles(&self) -> &[KnowledgeArticle] {
        &self.articles
    }

    /// 按 id 取文章。
    pub fn article(&self, id: &str) -> Option<&KnowledgeArticle> {
        self.articles.iter().find(|a| a.id == id)
    }

    /// 某板块的全部文章。
    pub fn by_kind(&self, kind: KnowledgeKind) -> impl Iterator<Item = &KnowledgeArticle> {
        self.articles.iter().filter(move |a| a.kind == kind)
    }
}

fn validate(article: &KnowledgeArticle, mode_library: &ModeLibrary) -> Result<(), KnowledgeError> {
    let invalid = |reason: String| KnowledgeError::Invalid {
        id: article.id.clone(),
        reason,
    };

    if article.id.trim().is_empty() {
        return Err(invalid("id 不能为空".into()));
    }
    if article.title.trim().is_empty() {
        return Err(invalid("标题不能为空".into()));
    }
    if article.summary.trim().is_empty() {
        return Err(invalid("导语不能为空".into()));
    }
    if article.sections.is_empty() {
        return Err(invalid("至少需要一个小节".into()));
    }

    // related_mode 的板块归属(RULE-039:仅解读类携带;考据/词条/不可落地不带)
    match article.kind {
        KnowledgeKind::ModeInterpretation => {}
        _ if article.related_mode.is_some() => {
            return Err(invalid(format!(
                "kind={} 不允许携带 related_mode(仅模式解读携带)",
                article.kind.as_str()
            )));
        }
        _ => {}
    }

    // sections 逐节校验:非空标题 + 非空段落;内容合规(零链接,RULE-044 的机械部分)
    for section in &article.sections {
        if section.heading.trim().is_empty() {
            return Err(invalid("小节标题不能为空".into()));
        }
        if section.paragraphs.is_empty() || section.paragraphs.iter().any(|p| p.trim().is_empty()) {
            return Err(invalid(format!(
                "小节「{}」的段落为空 —— 半截排版在装载期就该拦下",
                section.heading
            )));
        }
        for p in &section.paragraphs {
            if p.contains("http://") || p.contains("https://") {
                return Err(invalid(format!(
                    "小节「{}」含链接 —— 知识内容零外链(RULE-044)",
                    section.heading
                )));
            }
        }
    }

    match article.kind {
        KnowledgeKind::ModeInterpretation => {
            // RULE-039:必须关联真实存在的模式
            let Some(related) = article.related_mode.as_deref() else {
                return Err(invalid("模式解读必须携带 related_mode".into()));
            };
            if mode_library.mode(related).is_none() {
                return Err(invalid(format!(
                    "related_mode={related} 在模式库中不存在"
                )));
            }
            // RULE-039:局限性必须写 —— 这是可信度来源的一半
            if !article.sections.iter().any(|s| s.heading.trim() == "局限性") {
                return Err(invalid(
                    "模式解读必须包含「局限性」一节(总 PRD §4.5)".into(),
                ));
            }
        }
        KnowledgeKind::NonImplementable => {
            // RULE-040:「仅供理解」必须有原因
            if !article
                .sections
                .iter()
                .any(|s| s.heading.trim() == "为什么不建议照搬")
            {
                return Err(invalid(
                    "不可落地条目必须包含「为什么不建议照搬」一节(RULE-040)".into(),
                ));
            }
        }
        KnowledgeKind::Verification | KnowledgeKind::Encyclopedia => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ModeLibrary;

    fn lib() -> ModeLibrary {
        ModeLibrary::load_embedded().unwrap()
    }

    fn article_toml(id: &str, kind: &str, extra: &str, sections: &str) -> String {
        format!(
            r#"
id = "{id}"
kind = "{kind}"
title = "测试文章"
summary = "一句导语"
{extra}
[[sections]]
{sections}
"#
        )
    }

    const MODE_INTERP_SECTIONS: &str = r#"
heading = "核心逻辑"
paragraphs = ["切四堆。"]
[[sections]]
heading = "局限性"
paragraphs = ["不适用高负债家庭。"]
"#;

    #[test]
    fn 内嵌知识装载且四板块计数正确() {
        let lib = KnowledgeLibrary::load_embedded(&lib()).expect("内嵌知识必须可装载");
        let count = |k: KnowledgeKind| lib.by_kind(k).count();
        assert_eq!(count(KnowledgeKind::ModeInterpretation), 4, "解读 4 篇");
        assert_eq!(count(KnowledgeKind::Verification), 1, "考据 1 篇");
        assert_eq!(count(KnowledgeKind::Encyclopedia), 8, "百科 8 词条");
        assert_eq!(count(KnowledgeKind::NonImplementable), 3, "不可落地 3 篇");
        assert_eq!(lib.articles().len(), 16);
    }

    #[test]
    fn 内嵌解读全部关联真实模式且含局限性() {
        let modes = lib();
        let lib = KnowledgeLibrary::load_embedded(&modes).unwrap();
        for a in lib.by_kind(KnowledgeKind::ModeInterpretation) {
            let related = a.related_mode.as_deref().unwrap_or_else(|| {
                panic!("{} 是解读但缺 related_mode", a.id)
            });
            assert!(
                modes.mode(related).is_some(),
                "{} 的 related_mode={related} 不在模式库",
                a.id
            );
            assert!(
                a.sections.iter().any(|s| s.heading == "局限性"),
                "{} 缺「局限性」节",
                a.id
            );
        }
    }

    #[test]
    fn 不可落地条目含不建议照搬且不带关联模式() {
        let lib = KnowledgeLibrary::load_embedded(&lib()).unwrap();
        for a in lib.by_kind(KnowledgeKind::NonImplementable) {
            assert!(a.related_mode.is_none(), "不可落地条目不得携带 related_mode");
            assert!(
                a.sections.iter().any(|s| s.heading == "为什么不建议照搬"),
                "{} 缺「为什么不建议照搬」节",
                a.id
            );
        }
    }

    #[test]
    fn 解读缺局限性被拒绝() {
        let bad = article_toml(
            "ki-bad",
            "mode_interpretation",
            r#"related_mode = "four_accounts""#,
            r#"heading = "核心逻辑"
paragraphs = ["切四堆。"]"#,
        );
        let err = KnowledgeLibrary::from_sources(&[("ki-bad", &bad)], &lib()).unwrap_err();
        assert!(err.to_string().contains("局限性"), "实际: {err}");
    }

    #[test]
    fn 解读指向不存在的模式被拒绝() {
        let bad = article_toml(
            "ki-bad",
            "mode_interpretation",
            r#"related_mode = "no_such_mode""#,
            MODE_INTERP_SECTIONS,
        );
        let err = KnowledgeLibrary::from_sources(&[("ki-bad", &bad)], &lib()).unwrap_err();
        assert!(err.to_string().contains("不存在"), "实际: {err}");
    }

    #[test]
    fn 非解读类携带关联模式被拒绝() {
        let bad = article_toml(
            "ec-bad",
            "encyclopedia",
            r#"related_mode = "four_accounts""#,
            r#"heading = "是什么"
paragraphs = ["测试。"]"#,
        );
        let err = KnowledgeLibrary::from_sources(&[("ec-bad", &bad)], &lib()).unwrap_err();
        assert!(err.to_string().contains("related_mode"), "实际: {err}");
    }

    #[test]
    fn 不可落地缺不建议照搬被拒绝() {
        let bad = article_toml(
            "ni-bad",
            "non_implementable",
            "",
            r#"heading = "它说什么"
paragraphs = ["测试。"]"#,
        );
        let err = KnowledgeLibrary::from_sources(&[("ni-bad", &bad)], &lib()).unwrap_err();
        assert!(err.to_string().contains("为什么不建议照搬"), "实际: {err}");
    }

    #[test]
    fn 内容含链接被拒绝_rule044() {
        let bad = article_toml(
            "ec-link",
            "encyclopedia",
            "",
            r#"heading = "是什么"
paragraphs = ["详见 https://example.com。"]"#,
        );
        let err = KnowledgeLibrary::from_sources(&[("ec-link", &bad)], &lib()).unwrap_err();
        assert!(err.to_string().contains("链接"), "实际: {err}");
    }

    #[test]
    fn 文章id重复被拒绝() {
        let ok1 = article_toml("ki-dup", "encyclopedia", "", r#"heading = "是什么"
paragraphs = ["A。"]"#);
        let ok2 = ok1.clone();
        let err = KnowledgeLibrary::from_sources(
            &[("ki-dup", &ok1), ("ki-dup-2", &ok2)],
            &lib(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("重复"), "实际: {err}");
    }

    #[test]
    fn 按板块取文与按id取文() {
        let lib = KnowledgeLibrary::load_embedded(&lib()).unwrap();
        let first = lib.article("ec-rebalancing").expect("百科词条应可按 id 取");
        assert_eq!(first.kind, KnowledgeKind::Encyclopedia);
        assert!(lib.article("no-such").is_none());
    }
}
