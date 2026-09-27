//! users 表数据访问(B 期 · 单用户账号,ADR-B-002;H 期启用分层字段,ADR-H-001)。

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::Tier;

/// 账号记录。`password_hash` 只在本层与 auth service 之间流转,**永不进 API 响应**。
#[derive(Debug, Clone)]
pub struct UserRecord {
    /// 主键
    pub id: Uuid,
    /// 用户名
    pub username: String,
    /// argon2 哈希
    pub password_hash: String,
}

/// 按用户名查账号。返回 `None` 表示该账号不存在。
pub async fn find_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT id, username, password_hash FROM users WHERE username = $1"#,
        username
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| UserRecord {
        id: r.id,
        username: r.username,
        password_hash: r.password_hash,
    }))
}

/// 读账号当前分层(RULE-046 实时语义的数据源:每次请求现读,不进 JWT、不做缓存 ——
/// 置 plus 后下一个请求即生效)。账号不存在按 Free 降级(写入方只有运维脚本)。
pub async fn tier_of(pool: &PgPool, user_id: Uuid) -> Result<Tier, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT tier AS "tier!" FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map_or(Tier::Free, |r| Tier::parse(&r.tier)))
}

/// 幂等写入账号:不存在则建,存在则只更新口令哈希。
///
/// 种子脚本与将来的改密共用这一条路径 —— 两处各写一份 upsert 迟早会走岔。
pub async fn upsert(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        INSERT INTO users (id, username, password_hash)
        VALUES ($1, $2, $3)
        ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash
        RETURNING id
        "#,
        Uuid::now_v7(),
        username,
        password_hash
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}
