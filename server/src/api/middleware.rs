//! 鉴权中间件(RULE-001:默认拒绝)。

use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;

use crate::error::AppError;
use crate::services::auth_service;
use crate::state::AppState;

/// 会话 Cookie 名
pub const SESSION_COOKIE: &str = "ww_session";

/// 当前用户:由 `require_auth` 写入请求扩展,handler 用 `Extension<CurrentUser>` 取用。
///
/// 这解决了「中间件校验了,handler 却不知道是谁」的问题 —— 校验与使用同一份解析结果。
#[derive(Debug, Clone)]
pub struct CurrentUser {
    /// 用户 id(UUID 字符串)
    pub id: String,
    /// 用户名
    pub username: String,
}

/// 从 Cookie 头取指定名字的值。
///
/// 手写解析:只读一个名字,不值得为此引入 cookie 依赖(基线 §8.5「依赖越少越好」)。
pub fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let raw = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    raw.split(';').find_map(|part| {
        let (k, v) = part.split_once('=')?;
        (k.trim() == name).then(|| v.trim())
    })
}

/// RULE-001:挂在受保护路由组上的鉴权层 —— **默认拒绝**,无有效会话一律 401 信封。
///
/// 前端隐藏按钮不算权限(基线 ADR-005),这里是唯一的信任边界。
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = cookie_value(req.headers(), SESSION_COOKIE).ok_or(AppError::Unauthorized)?;
    let claims = auth_service::verify_token(&state.config.jwt_secret, token)
        .map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(CurrentUser {
        id: claims.sub,
        username: claims.name,
    });
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, header};

    fn headers_with_cookie(raw: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(header::COOKIE, HeaderValue::from_str(raw).unwrap());
        h
    }

    #[test]
    fn 从_cookie_头取出会话值() {
        let h = headers_with_cookie("other=1; ww_session=abc.def.ghi; trailing=2");
        assert_eq!(cookie_value(&h, SESSION_COOKIE), Some("abc.def.ghi"));
    }

    #[test]
    fn 无_cookie_头或名字不匹配时返回_none() {
        let empty = HeaderMap::new();
        assert_eq!(cookie_value(&empty, SESSION_COOKIE), None);

        let other = headers_with_cookie("foo=bar");
        assert_eq!(cookie_value(&other, SESSION_COOKIE), None);
    }

    #[test]
    fn 名字前缀相同但不相等时不误匹配() {
        // 防止 ww_session_x 被当成 ww_session
        let h = headers_with_cookie("ww_session_x=evil");
        assert_eq!(cookie_value(&h, SESSION_COOKIE), None);
    }
}
