//! 模式库与推荐的 DTO。

use serde::Serialize;
use utoipa::ToSchema;

use crate::domain::Credibility;

/// 一张模式卡。**全部字段来自 TOML 配置** —— 页面不硬编码任何模式信息,
/// 加模式只加一个配置文件。
#[derive(Debug, Serialize, ToSchema)]
pub struct ModeCardView {
    /// 模式 id
    pub id: String,
    /// 展示名
    pub name: String,
    /// 一句话理念
    pub tagline: String,
    /// 出处可信度
    pub credibility: Credibility,
    /// 适合人群标签
    pub fit_for: Vec<String>,
    /// 是否是本次的主推模式
    pub is_recommended: bool,
}

/// 投资桶的一个大类(只到大类,不出现任何具体产品)
#[derive(Debug, Serialize, ToSchema)]
pub struct L2ClassView {
    /// 大类名
    pub name: String,
    /// 万分比
    pub basis_points: i64,
}

/// L2 预览:选定模式后投资部分会怎么配。
#[derive(Debug, Serialize, ToSchema)]
pub struct L2PreviewView {
    /// 配置名(如「60/40」)
    pub name: String,
    /// 匹配理由
    pub reason: String,
    /// 大类与占比
    pub classes: Vec<L2ClassView>,
}

/// GET /api/v1/modes 的响应。
#[derive(Debug, Serialize, ToSchema)]
pub struct ModesView {
    /// 全部 L1 模式(顺序与配置目录一致)
    pub items: Vec<ModeCardView>,
    /// 主推模式 id;档案不足以推荐时为 null
    pub recommended_id: Option<String>,
    /// 主推理由(一句人话)
    pub recommendation_reason: String,
    /// L2 预览
    pub l2: L2PreviewView,
}
