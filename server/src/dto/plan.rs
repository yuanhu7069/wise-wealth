//! 方案 DTO(RULE-011):五段式方案页的数据面。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::Credibility;
use crate::domain::engine::{EmergencyStatus, Notice};
use crate::domain::l2::L2Allocation;

/// 生成入口(prd-f §9.5:区分问卷路径与模式库手动路径,仅用于埋点口径)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlanEntry {
    /// 问卷步 6 选定模式后生成
    Questionnaire,
    /// P06 模式库页「用此模式生成方案」(RULE-034)
    ModeLib,
}

/// 生成方案的请求。
#[derive(Debug, Deserialize, ToSchema)]
pub struct GeneratePlanRequest {
    /// 用户选定的 L1 模式 id
    pub l1_mode: String,
    /// 生成入口;缺省按 questionnaire(老客户端兼容)
    #[serde(default)]
    pub entry: Option<PlanEntry>,
}

/// 方案里的一个桶。
#[derive(Debug, Serialize, ToSchema)]
pub struct BucketView {
    /// 桶 id
    pub bucket_id: String,
    /// 展示名(如「工资账户」)
    pub name: String,
    /// 用途一句话
    pub purpose: String,
    /// 每月转入(分)。展示层转元并加千分位(ADR-004)
    pub amount_monthly_cents: i64,
    /// 目标金额(分,仅规则桶且未达标时有值)
    pub target_cents: Option<i64>,
}

/// 一份方案(五段式方案页的全部数据)。
#[derive(Debug, Serialize, ToSchema)]
pub struct PlanView {
    /// 方案 id
    pub id: Uuid,
    /// 版本号(重新生成会 +1)
    pub version: i32,
    /// 生成日期 YYYY-MM-DD
    pub created_date: String,
    /// L1 模式 id
    pub l1_mode: String,
    /// L1 模式展示名
    pub l1_mode_name: String,
    /// 模式可信度(读取时解析,ADR-F-002;模式已下架 → null,前端不渲染提示条)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l1_credibility: Option<Credibility>,
    /// 模式出处(读取时解析,ADR-F-002;disputed 方案页警示条文案用)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l1_source: Option<String>,
    /// 投资桶的每月转入(分,生成当时冻结):首页摘要的「每月可投资」
    pub investable_monthly_cents: i64,
    /// 各桶金额
    pub buckets: Vec<BucketView>,
    /// 投资桶的大类配置(快照回读)
    pub l2: L2Allocation,
    /// 应急金状态(快照回读)
    pub emergency: EmergencyStatus,
    /// 提示(缺口 / 固定支出超额 / 短久期)
    pub notices: Vec<Notice>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_缺省按问卷路径() {
        let req: GeneratePlanRequest =
            serde_json::from_str(r#"{"l1_mode":"four_pots"}"#).unwrap();
        assert_eq!(req.entry, None, "缺省不带 entry,handler 按 questionnaire 口径");
    }

    #[test]
    fn entry_两种入口可解析() {
        let req: GeneratePlanRequest =
            serde_json::from_str(r#"{"l1_mode":"four_pots","entry":"mode_lib"}"#).unwrap();
        assert_eq!(req.entry, Some(PlanEntry::ModeLib));
        let req: GeneratePlanRequest =
            serde_json::from_str(r#"{"l1_mode":"four_pots","entry":"questionnaire"}"#).unwrap();
        assert_eq!(req.entry, Some(PlanEntry::Questionnaire));
    }

    #[test]
    fn entry_未知枚举被拒() {
        // 越界值在 Json 提取期被拒(handler 侧表现为 422,arch-f §5)
        let r: Result<GeneratePlanRequest, _> =
            serde_json::from_str(r#"{"l1_mode":"four_pots","entry":"elsewhere"}"#);
        assert!(r.is_err(), "未知 entry 必须被拒绝");
    }
}
