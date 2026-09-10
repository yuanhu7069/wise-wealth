//! 方案 DTO(RULE-011):五段式方案页的数据面。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::engine::{EmergencyStatus, Notice};
use crate::domain::l2::L2Allocation;

/// 生成方案的请求。
#[derive(Debug, Deserialize, ToSchema)]
pub struct GeneratePlanRequest {
    /// 用户在步 6 选定的 L1 模式 id
    pub l1_mode: String,
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
    /// 各桶金额
    pub buckets: Vec<BucketView>,
    /// 投资桶的大类配置(快照回读)
    pub l2: L2Allocation,
    /// 应急金状态(快照回读)
    pub emergency: EmergencyStatus,
    /// 提示(缺口 / 固定支出超额 / 短久期)
    pub notices: Vec<Notice>,
}
