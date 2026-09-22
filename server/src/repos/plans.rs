//! plans / plan_buckets 数据访问。
//!
//! 生成时间是格式化后取回的(`to_char`):方案页只需要「生成于 2026-09-10」这一句,
//! 为它引入时间库的日期类型不划算。ORDER BY 仍走原始列。

use serde_json::Value as Json;
use sqlx::PgPool;
use uuid::Uuid;

/// 一个桶的落库形态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BucketRow {
    /// 桶 id
    pub bucket_id: String,
    /// 展示名
    pub name: String,
    /// 用途一句话
    pub purpose: String,
    /// 月转入(分)
    pub amount_monthly_cents: i64,
    /// 目标金额(分,可空)
    pub target_cents: Option<i64>,
}

/// 一份方案的落库形态。
#[derive(Debug, Clone)]
pub struct PlanRecord {
    /// 方案 id
    pub id: Uuid,
    /// L1 模式 id
    pub l1_mode: String,
    /// L2 配置 id
    pub l2_mode: String,
    /// 版本号(用户内自增)
    pub version: i32,
    /// 是否当前方案
    pub is_active: bool,
    /// 生成当时投资桶的每月转入(分):首页摘要的「每月可投资」
    pub investable_monthly_cents: i64,
    /// 生成日期(YYYY-MM-DD)
    pub created_date: String,
    /// 档案快照
    pub profile_snapshot: Json,
    /// L2 配置快照
    pub l2_allocation: Json,
    /// 应急金状态快照
    pub emergency: Json,
    /// 提示快照
    pub notices: Json,
    /// 推理链快照
    pub traces: Json,
}

/// 待落库的一份完整快照。参数超过七八个之后,散着传必然传错序 —— 打包成结构。
pub struct NewPlan<'a> {
    /// 归属用户
    pub user_id: Uuid,
    /// L1 模式 id
    pub l1_mode: &'a str,
    /// L2 配置 id
    pub l2_mode: &'a str,
    /// 投资桶的每月转入(分,生成当时冻结)
    pub investable_monthly_cents: i64,
    /// 档案快照
    pub profile_snapshot: &'a Json,
    /// L2 配置快照
    pub l2_allocation: &'a Json,
    /// 应急金状态快照
    pub emergency: &'a Json,
    /// 提示快照
    pub notices: &'a Json,
    /// 推理链快照
    pub traces: &'a Json,
    /// 桶明细
    pub buckets: &'a [BucketRow],
}

/// 写入一份新方案,并把它设为该用户唯一的 active。
///
/// **事务**:先把旧方案置为非 active,再插入新方案 —— 部分唯一索引
/// `plans_one_active_per_user` 要求任一时刻至多一个 active,顺序颠倒会直接冲突。
/// 版本号在同一事务内取 `max+1`,并发下由索引兜底。
pub async fn insert_plan(pool: &PgPool, new: &NewPlan<'_>) -> Result<Uuid, sqlx::Error> {
    let NewPlan {
        user_id,
        l1_mode,
        l2_mode,
        investable_monthly_cents,
        profile_snapshot,
        l2_allocation,
        emergency,
        notices,
        traces,
        buckets,
    } = *new;
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "UPDATE plans SET is_active = FALSE WHERE user_id = $1 AND is_active",
        user_id
    )
    .execute(&mut *tx)
    .await?;

    let plan_id = Uuid::now_v7();
    sqlx::query!(
        r#"
        INSERT INTO plans (
            id, user_id, l1_mode, l2_mode, version, is_active,
            investable_monthly_cents,
            profile_snapshot_json, l2_allocation_json, emergency_json,
            notices_json, traces_json
        )
        VALUES (
            $1, $2, $3, $4,
            (SELECT COALESCE(MAX(version), 0) + 1 FROM plans WHERE user_id = $2),
            TRUE, $5, $6, $7, $8, $9, $10
        )
        "#,
        plan_id,
        user_id,
        l1_mode,
        l2_mode,
        investable_monthly_cents,
        profile_snapshot,
        l2_allocation,
        emergency,
        notices,
        traces
    )
    .execute(&mut *tx)
    .await?;

    for (i, b) in buckets.iter().enumerate() {
        sqlx::query!(
            r#"
            INSERT INTO plan_buckets (
                id, plan_id, bucket_id, name, purpose, amount_monthly_cents, target_cents, sort_order
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            Uuid::now_v7(),
            plan_id,
            b.bucket_id,
            b.name,
            b.purpose,
            b.amount_monthly_cents,
            b.target_cents,
            i as i16
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(plan_id)
}

/// 读当前 active 方案(没有则 None)。
pub async fn active_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<PlanRecord>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, l1_mode, l2_mode, version, is_active, investable_monthly_cents,
               to_char(created_at, 'YYYY-MM-DD') AS "created_date!",
               profile_snapshot_json, l2_allocation_json, emergency_json,
               notices_json, traces_json
        FROM plans
        WHERE user_id = $1 AND is_active
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| PlanRecord {
        id: r.id,
        l1_mode: r.l1_mode,
        l2_mode: r.l2_mode,
        version: r.version,
        is_active: r.is_active,
        investable_monthly_cents: r.investable_monthly_cents,
        created_date: r.created_date,
        profile_snapshot: r.profile_snapshot_json,
        l2_allocation: r.l2_allocation_json,
        emergency: r.emergency_json,
        notices: r.notices_json,
        traces: r.traces_json,
    }))
}

/// 按 id 读方案(E 期:快照回溯所属版本用;快照里只存了 plan_id,RULE-028)。
pub async fn by_id(pool: &PgPool, id: Uuid) -> Result<Option<PlanRecord>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, l1_mode, l2_mode, version, is_active, investable_monthly_cents,
               to_char(created_at, 'YYYY-MM-DD') AS "created_date!",
               profile_snapshot_json, l2_allocation_json, emergency_json,
               notices_json, traces_json
        FROM plans
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| PlanRecord {
        id: r.id,
        l1_mode: r.l1_mode,
        l2_mode: r.l2_mode,
        version: r.version,
        is_active: r.is_active,
        investable_monthly_cents: r.investable_monthly_cents,
        created_date: r.created_date,
        profile_snapshot: r.profile_snapshot_json,
        l2_allocation: r.l2_allocation_json,
        emergency: r.emergency_json,
        notices: r.notices_json,
        traces: r.traces_json,
    }))
}

/// 读一份方案的全部桶(按写入顺序)。
pub async fn buckets_of(pool: &PgPool, plan_id: Uuid) -> Result<Vec<BucketRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT bucket_id, name, purpose, amount_monthly_cents, target_cents
        FROM plan_buckets WHERE plan_id = $1 ORDER BY sort_order
        "#,
        plan_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| BucketRow {
            bucket_id: r.bucket_id,
            name: r.name,
            purpose: r.purpose,
            amount_monthly_cents: r.amount_monthly_cents,
            target_cents: r.target_cents,
        })
        .collect())
}
