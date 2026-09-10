//! 鉴权端点:登录 / 退出 / 会话探测(ADR-B-002)。
//!
//! 三处 401 的区分方式:登录失败与「会话过期」同为 UNAUTHORIZED 错误码,
//! 前端按**请求位置**区分文案(登录页上收到的 401 = 凭证不符;别处收到的 = 会话过期),
//! 因此不需要为二者新增错误码(基线 §6.2 的错误码表是封闭的)。

use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::Json;
use validator::Validate;

use crate::api::middleware::{SESSION_COOKIE, cookie_value};
use crate::config::AppEnv;
use crate::dto::auth::{LoginRequest, SessionInfo};
use crate::error::{ApiOk, AppError, first_validation_message};
use crate::repos;
use crate::services::auth_service;
use crate::state::AppState;

/// POST /api/v1/auth/login
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    summary = "登录并签发 HttpOnly 会话 Cookie",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "登录成功,Set-Cookie 携带会话", body = crate::error::Envelope<SessionInfo>),
        (status = 401, description = "凭证不符"),
        (status = 429, description = "失败次数过多,暂时限流")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Response, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(first_validation_message(&e)))?;

    // RULE-002:窗口内失败已达上限 → 直接拒绝,不再比对口令
    if state.login_limiter.is_blocked(&req.username) {
        return Err(AppError::RateLimited);
    }

    // 「账号不存在」与「口令不符」对外表现完全一致:不给攻击者枚举账号的依据。
    // 账号不存在时也记一次失败,避免用不存在的用户名绕过限流。
    let Some(user) = repos::users::find_by_username(&state.pool, &req.username).await? else {
        state.login_limiter.record_failure(&req.username);
        return Err(AppError::InvalidCredentials);
    };

    if !auth_service::verify_password(&req.password, &user.password_hash) {
        state.login_limiter.record_failure(&req.username);
        return Err(AppError::InvalidCredentials);
    }

    state.login_limiter.clear(&req.username);

    let token = auth_service::issue_token(
        &state.config.jwt_secret,
        &user.id.to_string(),
        &user.username,
        state.config.session_ttl_days,
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("会话签发失败: {e}")))?;

    // 口令与用户名不进日志(基线 §8.3)
    tracing::info!(user = %user.username, "登录成功");

    let mut resp = ApiOk(SessionInfo {
        username: user.username.clone(),
    })
    .into_response();
    resp.headers_mut().append(
        header::SET_COOKIE,
        session_cookie(&token, state.config.session_ttl_days, state.config.app_env),
    );
    *resp.status_mut() = StatusCode::OK;
    Ok(resp)
}

/// POST /api/v1/auth/logout
///
/// 放在公开组:会话已过期时点「退出」也必须能顺利清除 Cookie 并返回成功,
/// 否则用户会卡在一个退不掉的登录态里。
#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    summary = "清除会话 Cookie",
    responses((status = 200, description = "已退出"))
)]
pub async fn logout(State(state): State<AppState>) -> Response {
    let mut resp = ApiOk(crate::dto::auth::OkBody { ok: true }).into_response();
    resp.headers_mut().append(
        header::SET_COOKIE,
        cleared_cookie(state.config.app_env),
    );
    *resp.status_mut() = StatusCode::OK;
    resp
}

/// GET /api/v1/auth/me —— 会话探测。未登录返回 401 信封(前端据此跳登录)。
#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tag = "auth",
    summary = "读取当前会话",
    responses(
        (status = 200, description = "已登录", body = crate::error::Envelope<SessionInfo>),
        (status = 401, description = "未登录或会话过期")
    )
)]
pub async fn me(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<ApiOk<SessionInfo>, AppError> {
    // 本端点在公开组(未登录时也必须能被调用并返回 401 信封,而不是被中间件短路),
    // 故自行解析会话 —— 与 require_auth 用的是同一个校验函数,不存在两套判断。
    let token = cookie_value(&headers, SESSION_COOKIE).ok_or(AppError::Unauthorized)?;
    let claims = auth_service::verify_token(&state.config.jwt_secret, token)
        .map_err(|_| AppError::Unauthorized)?;
    Ok(ApiOk(SessionInfo {
        username: claims.name,
    }))
}

/// 会话 Cookie。HttpOnly + SameSite=Lax;生产环境补 Secure(基线 §4.7-2 / §8.4-3)。
fn session_cookie(token: &str, ttl_days: i64, env: AppEnv) -> header::HeaderValue {
    let max_age = ttl_days * 24 * 60 * 60;
    let secure = if env == AppEnv::Prod { "; Secure" } else { "" };
    header::HeaderValue::from_str(&format!(
        "{SESSION_COOKIE}={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age={max_age}{secure}"
    ))
    .unwrap_or_else(|_| header::HeaderValue::from_static(""))
}

/// 清除 Cookie(Max-Age=0)。
fn cleared_cookie(env: AppEnv) -> header::HeaderValue {
    let secure = if env == AppEnv::Prod { "; Secure" } else { "" };
    header::HeaderValue::from_str(&format!(
        "{SESSION_COOKIE}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0{secure}"
    ))
    .unwrap_or_else(|_| header::HeaderValue::from_static(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 会话_cookie_带安全属性() {
        let v = session_cookie("tok", 30, AppEnv::Dev);
        let s = v.to_str().unwrap();
        assert!(s.contains("HttpOnly"), "必须 HttpOnly: {s}");
        assert!(s.contains("SameSite=Lax"), "必须 SameSite: {s}");
        assert!(s.contains("Path=/"), "必须限定 Path: {s}");
        assert!(s.contains(&format!("Max-Age={}", 30 * 24 * 60 * 60)));
        assert!(!s.contains("Secure"), "dev 环境不加 Secure(本地为 http)");
    }

    #[test]
    fn 生产环境补_secure() {
        let s = session_cookie("tok", 7, AppEnv::Prod);
        let s = s.to_str().unwrap();
        assert!(s.contains("Secure"), "生产环境必须 Secure: {s}");
    }

    #[test]
    fn 退出时清除_cookie() {
        let s = cleared_cookie(AppEnv::Dev);
        let s = s.to_str().unwrap();
        assert!(s.contains("Max-Age=0"), "清除必须置 Max-Age=0: {s}");
        assert!(s.contains(&format!("{SESSION_COOKIE}=;")), "值应为空: {s}");
    }
}
