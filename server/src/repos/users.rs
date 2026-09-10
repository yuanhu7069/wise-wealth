//! users 表数据访问(B 期 · 单用户账号,ADR-B-002)。

use sqlx::PgPool;
use uuid::Uuid;

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
