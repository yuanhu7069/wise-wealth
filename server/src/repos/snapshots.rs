//! snapshots 数据访问(ADR-E-001)。
//!
//! 月份在 SQL 里以 `DATE` 存储、以 `to_char` 的 `YYYY-MM` 字符串取回 —— 与
//! plans 的 `created_date` 同一取舍(方案页/追踪页只需要「2026-09」这一句,
//! 为它引入日期类型映射不划算);比较仍走原始列,字符串只在边界外流动。

use serde_json::Value as Json;
use sqlx::PgPool;
use uuid::Uuid;

/// 一条快照的落库形态。
#[derive(Debug, Clone)]
pub struct SnapshotRow {
    /// 快照 id
    pub id: Uuid,
    /// 自然月(YYYY-MM)
    pub month: String,
    /// 提交时的方案版本 id(RULE-028:冻结;方案版本本身永存)
    pub plan_id: Uuid,
    /// 各桶余额(JSONB:键 = 桶 id,值 = 分)
    pub balances: Json,
    /// 本月特殊(RULE-024)
    pub special_month: bool,
}

/// 提交或覆盖当月快照,返回「是否为新插入」(埋点 is_overwrite 的依据)。
///
/// **事务 + 先查后写**,而不是 `ON CONFLICT` 单语句:覆盖与否是上报给埋点的
/// 业务事实,必须来自明确的判断,不能押在 `xmax` 这类实现细节上。
pub async fn upsert(
    pool: &PgPool,
    user_id: Uuid,
    plan_id: Uuid,
    month: &str,
    balances: &Json,
    special_month: bool,
) -> Result<(SnapshotRow, bool), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM snapshots
            WHERE user_id = $1 AND month = to_date($2 || '-01', 'YYYY-MM-DD')
        ) AS "exists!"
        "#,
        user_id,
        month
    )
    .fetch_one(&mut *tx)
    .await?;

    // 两个 `query!` 各生成独立的匿名行类型,分支内立即映射成 [`SnapshotRow`],
    // 不让它们在 if/else 两侧相遇。
    let row = if exists {
        let r = sqlx::query!(
            r#"
            UPDATE snapshots
            SET plan_id = $3, balances = $4, special_month = $5, updated_at = now()
            WHERE user_id = $1 AND month = to_date($2 || '-01', 'YYYY-MM-DD')
            RETURNING id, to_char(month, 'YYYY-MM') AS "month!", plan_id, balances, special_month
            "#,
            user_id,
            month,
            plan_id,
            balances,
            special_month
        )
        .fetch_one(&mut *tx)
        .await?;
        SnapshotRow {
            id: r.id,
            month: r.month,
            plan_id: r.plan_id,
            balances: r.balances,
            special_month: r.special_month,
        }
    } else {
        let r = sqlx::query!(
            r#"
            INSERT INTO snapshots (id, user_id, plan_id, month, balances, special_month)
            VALUES ($1, $2, $3, to_date($4 || '-01', 'YYYY-MM-DD'), $5, $6)
            RETURNING id, to_char(month, 'YYYY-MM') AS "month!", plan_id, balances, special_month
            "#,
            Uuid::now_v7(),
            user_id,
            plan_id,
            month,
            balances,
            special_month
        )
        .fetch_one(&mut *tx)
        .await?;
        SnapshotRow {
            id: r.id,
            month: r.month,
            plan_id: r.plan_id,
            balances: r.balances,
            special_month: r.special_month,
        }
    };

    tx.commit().await?;
    Ok((row, !exists))
}

/// 最新一条快照(没有则 None)。
pub async fn latest(pool: &PgPool, user_id: Uuid) -> Result<Option<SnapshotRow>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, to_char(month, 'YYYY-MM') AS "month!", plan_id, balances, special_month
        FROM snapshots WHERE user_id = $1
        ORDER BY month DESC LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| SnapshotRow {
        id: r.id,
        month: r.month,
        plan_id: r.plan_id,
        balances: r.balances,
        special_month: r.special_month,
    }))
}

/// `month` 之前最近的一条非特殊快照(RULE-025 的基准查询;无则 None)。
///
/// 「跳过特殊月」在 SQL 里一行 `AND NOT special_month` 说完 —— 域层的
/// [`crate::domain::tracking::deviations`] 只管「给了基准怎么比」,「谁是基准」归这里。
pub async fn prev_non_special_before(
    pool: &PgPool,
    user_id: Uuid,
    month: &str,
) -> Result<Option<SnapshotRow>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, to_char(month, 'YYYY-MM') AS "month!", plan_id, balances, special_month
        FROM snapshots
        WHERE user_id = $1
          AND month < to_date($2 || '-01', 'YYYY-MM-DD')
          AND NOT special_month
        ORDER BY month DESC LIMIT 1
        "#,
        user_id,
        month
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| SnapshotRow {
        id: r.id,
        month: r.month,
        plan_id: r.plan_id,
        balances: r.balances,
        special_month: r.special_month,
    }))
}

/// 该用户有快照的月数(RULE-027 的数据来源;UNIQUE 约束保证月份不重,count 即去重结果)。
pub async fn count(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    let n = sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!" FROM snapshots WHERE user_id = $1"#,
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(n)
}

/// 历史快照,按月倒序、分页(红线 11:列表接口不得全表返回)。
pub async fn list_desc(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<SnapshotRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, to_char(month, 'YYYY-MM') AS "month!", plan_id, balances, special_month
        FROM snapshots WHERE user_id = $1
        ORDER BY month DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| SnapshotRow {
            id: r.id,
            month: r.month,
            plan_id: r.plan_id,
            balances: r.balances,
            special_month: r.special_month,
        })
        .collect())
}

/// 删除某月快照(仅当月可删的边界在 service;这里只管删,返回影响行数)。
pub async fn delete_month(
    pool: &PgPool,
    user_id: Uuid,
    month: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM snapshots
        WHERE user_id = $1 AND month = to_date($2 || '-01', 'YYYY-MM-DD')
        "#,
        user_id,
        month
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// CSV 导出长表行(RULE-030:每行 = 月 × 桶;桶名取该快照所属方案版本的冻结桶名)。
///
/// `jsonb_each_text` 把余额对象展开成行;桶名经 plan_buckets 关联 ——
/// 不同方案版本的桶名可能不同(模式配置改过名),按各自版本的叫法导出才叫历史。
pub struct ExportRow {
    /// 自然月(YYYY-MM)
    pub month: String,
    /// 方案版本号
    pub version: i32,
    /// 桶 id
    pub bucket_id: String,
    /// 桶名(理论上必非空:balances 键集在提交时校验过 == 该方案桶集;
    /// LEFT JOIN 兜底 None 时由调用方回落到桶 id)
    pub bucket_name: Option<String>,
    /// 余额(分)
    pub cents: i64,
    /// 本月特殊
    pub special_month: bool,
}

pub async fn export_rows(pool: &PgPool, user_id: Uuid) -> Result<Vec<ExportRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT to_char(s.month, 'YYYY-MM') AS "month!",
               p.version,
               e.key AS "bucket_id!",
               pb.name AS "bucket_name: Option<String>",
               (e.value)::bigint AS "cents!",
               s.special_month AS "special_month!"
        FROM snapshots s
        JOIN plans p ON p.id = s.plan_id
        CROSS JOIN LATERAL jsonb_each_text(s.balances) AS e(key, value)
        LEFT JOIN plan_buckets pb ON pb.plan_id = s.plan_id AND pb.bucket_id = e.key
        WHERE s.user_id = $1
        ORDER BY s.month, pb.sort_order
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ExportRow {
            month: r.month,
            version: r.version,
            bucket_id: r.bucket_id,
            bucket_name: r.bucket_name,
            cents: r.cents,
            special_month: r.special_month,
        })
        .collect())
}
