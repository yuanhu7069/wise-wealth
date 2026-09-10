//! 鉴权 service(ADR-B-002):argon2 口令哈希 + JWT 会话签发与校验。
//!
//! 只做密码学与令牌,不碰数据库、不碰 HTTP —— 数据访问在 `repos::users`,
//! 协议转换在 `api::v1::auth`。

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

/// 鉴权失败原因。区分「口令不符」与「令牌无效」只为日志可读,
/// 对外一律映射为 401 UNAUTHORIZED(不给攻击者区分依据)。
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// argon2 哈希或校验失败
    #[error("口令处理失败")]
    Password,
    /// JWT 签发失败
    #[error("会话签发失败")]
    Issue,
    /// JWT 校验失败(过期/伪造/格式错)
    #[error("会话无效或已过期")]
    Token,
}

/// 会话载荷。`sub` 放用户 id,`name` 放用户名:
/// 前端只需要用户名,但服务端校验资源归属时需要 id。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionClaims {
    /// 用户 id(UUID 字符串)
    pub sub: String,
    /// 用户名
    pub name: String,
    /// 过期时间(Unix 秒)
    pub exp: i64,
    /// 签发时间(Unix 秒)
    pub iat: i64,
}

/// 口令哈希(argon2 默认参数)。**禁止明文、禁止弱哈希**(基线 §4.7-1)。
pub fn hash_password(plain: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AuthError::Password)
}

/// 校验口令。哈希串本身异常时按「不匹配」处理,不向上抛 —— 调用方只关心是与否。
pub fn verify_password(plain: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// 签发会话令牌(HS256)。`ttl_days` 由配置给,缺省 30 天。
pub fn issue_token(
    secret: &str,
    user_id: &str,
    username: &str,
    ttl_days: i64,
) -> Result<String, AuthError> {
    let now = unix_now();
    let claims = SessionClaims {
        sub: user_id.to_string(),
        name: username.to_string(),
        iat: now,
        exp: now + ttl_days * 24 * 60 * 60,
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AuthError::Issue)
}

/// 校验会话令牌。过期由 jsonwebtoken 的 exp 校验负责。
pub fn verify_token(secret: &str, token: &str) -> Result<SessionClaims, AuthError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 5; // 容忍 5 秒时钟偏移
    decode::<SessionClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| AuthError::Token)
}

/// 当前 Unix 时间(秒)。用标准库而非引入时间库:这里只需要一个整数。
fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-at-least-32-characters-long";

    #[test]
    fn 口令哈希可校验且不存明文() {
        let hash = hash_password("wiseWealth#123").unwrap();
        assert!(!hash.contains("wiseWealth"), "哈希串不得包含明文");
        assert!(hash.starts_with("$argon2"), "应使用 argon2: {hash}");
        assert!(verify_password("wiseWealth#123", &hash));
        assert!(!verify_password("wrong-password", &hash));
    }

    #[test]
    fn 同一口令两次哈希结果不同因加盐() {
        let a = hash_password("same-password").unwrap();
        let b = hash_password("same-password").unwrap();
        assert_ne!(a, b, "加盐后两次哈希必须不同");
        assert!(verify_password("same-password", &a));
        assert!(verify_password("same-password", &b));
    }

    #[test]
    fn 哈希串损坏时校验为假而非崩溃() {
        assert!(!verify_password("any", "这不是一个合法哈希"));
        assert!(!verify_password("any", ""));
    }

    #[test]
    fn 令牌可签发可校验() {
        let token = issue_token(SECRET, "user-1", "苑问", 30).unwrap();
        let claims = verify_token(SECRET, &token).unwrap();
        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.name, "苑问");
        assert!(claims.exp > claims.iat, "过期时间必须晚于签发时间");
        // 30 天有效期(留 5 秒余量)
        let span = claims.exp - claims.iat;
        assert!((30 * 24 * 60 * 60 - span).abs() <= 5, "有效期应为 30 天");
    }

    #[test]
    fn 换密钥后旧令牌失效() {
        let token = issue_token(SECRET, "user-1", "苑问", 30).unwrap();
        assert!(verify_token("another-secret-also-32-chars-long!!", &token).is_err());
    }

    #[test]
    fn 伪造令牌被拒() {
        assert!(verify_token(SECRET, "not.a.jwt").is_err());
        assert!(verify_token(SECRET, "").is_err());
    }

    #[test]
    fn 过期令牌被拒() {
        // ttl 为负 → exp 早于现在,应立即失效
        let token = issue_token(SECRET, "user-1", "苑问", -1).unwrap();
        assert!(verify_token(SECRET, &token).is_err(), "过期令牌必须被拒");
    }
}
