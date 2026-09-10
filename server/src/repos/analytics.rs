//! analytics_events 数据访问。
//!
//! 写入端只此一处。**没有面向展示的查询接口** —— 本期埋点的消费方式是人工 SQL 查验
//! (ADR-B-005 自用期无看板)。唯一的读函数是 [`count_of_type`]:服务层用它判断
//! 一次性事件(问卷开始)是否已经记过,不是为了展示。

use serde_json::Value as Json;
use sqlx::PgPool;
use uuid::Uuid;

/// 追加一条事件。时间由数据库 `now()` 落,避免应用进程时钟与库时钟不一致时
/// 「事件时间」与「方案生成时间」两个来源打架。
pub async fn insert(pool: &PgPool, event_type: &str, payload: &Json) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO analytics_events (id, event_type, payload_json)
        VALUES ($1, $2, $3)
        "#,
        Uuid::now_v7(),
        event_type,
        payload
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// 某类事件已有多少条。
///
/// 只给「一次性事件是否已记过」这一个判据用(见 `analytics_service::record`);
/// **不要**拿它去做统计报表 —— 那类查询直接走 SQL。`count(*)` 在自用期
/// (日均 < 50 条)不必优化。
pub async fn count_of_type(pool: &PgPool, event_type: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!" FROM analytics_events WHERE event_type = $1"#,
        event_type
    )
    .fetch_one(pool)
    .await
}
