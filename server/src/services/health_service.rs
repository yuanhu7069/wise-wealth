//! 健康检查 service(RULE-001):data 仅 status/db/version 三字段。

use sqlx::PgPool;
use std::time::Duration;

/// 给 Future 加超时的轻量扩展(避免引入额外依赖,超时语义同 tokio::time::timeout)
trait TimeoutExt: Future + Sized {
    async fn timeout(self, d: Duration) -> Result<Self::Output, std::io::Error>;
}

impl<F: Future> TimeoutExt for F {
    async fn timeout(self, d: Duration) -> Result<F::Output, std::io::Error> {
        match tokio::time::timeout(d, self).await {
            Ok(v) => Ok(v),
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "db probe timeout",
            )),
        }
    }
}

/// 健康数据(RULE-001 白名单:恰好三个字段,不含连接串/主机名/内部路径)。
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, PartialEq)]
pub struct HealthData {
    /// 恒 "ok" — 服务进程存活即可应答(ADR-A-003)
    pub status: String,
    /// "ok" | "error" — 数据库可用性
    pub db: String,
    /// 后端版本号
    pub version: String,
}

/// 探测数据库连通性。查询失败/超时一律归为 db=error,不上抛、不崩溃。
pub async fn check(pool: &PgPool, version: &str) -> HealthData {
    let db = match probe(pool).await {
        Ok(()) => "ok".to_string(),
        Err(_) => "error".to_string(),
    };
    HealthData {
        status: "ok".to_string(),
        db,
        version: version.to_string(),
    }
}

async fn probe(pool: &PgPool) -> Result<(), sqlx::Error> {
    // 单连接短超时:库不可达时快速返回 error 而非长时间挂起。
    // query_scalar! 为编译期校验宏(sqlx 优先策略,基线 §4.4);离线缓存由 cargo sqlx prepare 维护。
    let mut conn = pool.acquire().await?;
    let fut = sqlx::query_scalar!("SELECT 1").fetch_one(&mut *conn);
    match fut.timeout(Duration::from_secs(2)).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(io_err) => Err(sqlx::Error::Io(io_err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RULE-001 字段白名单:序列化后恰好 3 个键
    #[test]
    fn health_data_has_exactly_three_fields() {
        let d = HealthData {
            status: "ok".into(),
            db: "ok".into(),
            version: "0.1.0".into(),
        };
        let v = serde_json::to_value(&d).unwrap();
        let keys = v.as_object().unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains_key("status"));
        assert!(keys.contains_key("db"));
        assert!(keys.contains_key("version"));
    }

    /// RULE-001:db 取值只能是 ok|error(此处验证 error 形态)
    #[test]
    fn health_data_error_shape() {
        let d = HealthData {
            status: "ok".into(),
            db: "error".into(),
            version: "0.1.0".into(),
        };
        assert_eq!(d.status, "ok");
        assert_eq!(d.db, "error");
    }
}
