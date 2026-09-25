//! handler 层路由装配(只做协议转换,业务在 services 层)。

pub mod middleware;
pub mod v1 {
    pub mod analytics;
    pub mod auth;
    pub mod health;
    pub mod modes;
    pub mod plans;
    pub mod profiles;
    pub mod snapshots;
}

use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::Router;

use crate::state::AppState;

/// 全部路由。
///
/// 分两组,边界即**信任边界**(基线 §4.7-4「默认拒绝」):
/// - **公开组**:健康探测、OpenAPI、鉴权三端点(登录 / 退出 / 会话探测)
/// - **受保护组**:业务端点。B 期的业务端点自 ticket 03 起逐个挂入
///   `protected`(`/api/v1/profiles/...`、`/plans...`、`/modes`),
///   统一由 `middleware::require_auth` 兜底 —— 新增端点时忘记加鉴权是不可能发生的,
///   因为它必须显式挂到哪一组里。
pub fn routes(state: AppState) -> Router {
    let public = Router::new()
        .route("/health", get(crate::api::v1::health::health_handler))
        .route("/openapi.json", get(openapi_json_handler))
        .route("/auth/login", post(crate::api::v1::auth::login))
        .route("/auth/logout", post(crate::api::v1::auth::logout))
        .route("/auth/me", get(crate::api::v1::auth::me));

    // 受保护组:默认拒绝。首个业务端点(档案读写)已经挂上,
    // 后续端点(方案生成 / 模式列表 / 埋点)按同样方式追加即可。
    let protected = Router::new()
        .route(
            "/profiles/me",
            get(crate::api::v1::profiles::get_profile),
        )
        .route(
            "/profiles/me/step",
            put(crate::api::v1::profiles::save_step),
        )
        .route("/modes", get(crate::api::v1::modes::list_modes))
        .route("/plans", post(crate::api::v1::plans::generate_plan))
        .route("/plans/active", get(crate::api::v1::plans::active_plan))
        .route(
            "/plans/active/export",
            get(crate::api::v1::snapshots::export_plan_csv),
        )
        .route(
            "/snapshots",
            get(crate::api::v1::snapshots::list_snapshots),
        )
        .route(
            "/snapshots/export",
            get(crate::api::v1::snapshots::export_snapshots_csv),
        )
        .route(
            "/snapshots/{month}",
            axum::routing::put(crate::api::v1::snapshots::upsert_snapshot)
                .delete(crate::api::v1::snapshots::delete_snapshot),
        )
        .route(
            "/analytics/events",
            post(crate::api::v1::analytics::record_event),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::api::middleware::require_auth,
        ));

    Router::new()
        .route("/health", get(crate::api::v1::health::health_handler))
        .nest("/api/v1", public.merge(protected))
        .with_state(state)
}

async fn openapi_json_handler() -> axum::response::Response {
    let doc = crate::openapi::openapi_json();
    // RULE-xxx 标注:handler 路径禁止 unwrap/expect(基线 §16.6)。
    // OpenAPI 文档为进程内纯内存数据,序列化失败仅可能来自程序缺陷,无运行期触发条件;
    // 失败时按 RULE-003 返回 INTERNAL_ERROR 信封,不 panic。
    let json = match serde_json::to_value(&doc) {
        Ok(v) => v,
        Err(_) => serde_json::json!({
            "success": false,
            "data": null,
            "errorCode": "INTERNAL_ERROR",
            "message": "OpenAPI 文档序列化失败"
        }),
    };
    (
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/json")],
        axum::Json(json),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use crate::api::middleware::{CurrentUser, SESSION_COOKIE, require_auth};
    use crate::config::{AppEnv, Config};
    use crate::state::AppState;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let vars = std::collections::HashMap::from([
            ("APP_ENV".to_string(), "dev".to_string()),
            ("APP_PORT".to_string(), "8080".to_string()),
            (
                "DATABASE_URL_DEV".to_string(),
                "postgres://u:p@127.0.0.1:5432/wise_wealth_dev".to_string(),
            ),
            (
                "JWT_SECRET".to_string(),
                "test-secret-at-least-32-characters-long".to_string(),
            ),
        ]);
        let config = Config::from_env(vars).expect("测试配置应合法");
        let library = crate::domain::ModeLibrary::load_embedded().unwrap();
        AppState::new(&config, library, crate::domain::KnowledgeLibrary::default())
            .expect("惰性连接池不应失败")
    }

    /// 受保护组的最小复现:一个受 `require_auth` 保护的业务端点。
    fn protected_router(state: AppState) -> Router {
        Router::new()
            .route(
                "/business",
                get(|axum::Extension(u): axum::Extension<CurrentUser>| async move {
                    format!("hello {}", u.username)
                }),
            )
            .route_layer(axum::middleware::from_fn_with_state(
                state.clone(),
                require_auth,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn 无_cookie_访问受保护端点返回_401_信封() {
        let resp = protected_router(test_state())
            .oneshot(
                Request::builder()
                    .uri("/business")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["errorCode"], "UNAUTHORIZED");
        assert_eq!(json["data"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn 伪造_cookie_返回_401() {
        let resp = protected_router(test_state())
            .oneshot(
                Request::builder()
                    .uri("/business")
                    .header(header::COOKIE, format!("{SESSION_COOKIE}=forged.token.value"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn 有效会话放行且注入当前用户() {
        let state = test_state();
        let token = crate::services::auth_service::issue_token(
            &state.config.jwt_secret,
            "11111111-1111-7111-8111-111111111111",
            "苑问",
            30,
        )
        .unwrap();

        let resp = protected_router(state)
            .oneshot(
                Request::builder()
                    .uri("/business")
                    .header(header::COOKIE, format!("{SESSION_COOKIE}={token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
        assert_eq!(String::from_utf8_lossy(&body), "hello 苑问");
    }

    #[test]
    fn 配置可构造测试态() {
        // 顺带锁定:JWT_SECRET 属必需项,缺失时 from_env 必须失败
        let mut vars = std::collections::HashMap::from([
            ("APP_ENV".to_string(), "dev".to_string()),
            ("APP_PORT".to_string(), "8080".to_string()),
            (
                "DATABASE_URL_DEV".to_string(),
                "postgres://u:p@127.0.0.1:5432/db".to_string(),
            ),
        ]);
        let err = Config::from_env(vars.clone()).unwrap_err();
        assert!(
            err.problems.iter().any(|p| p.contains("JWT_SECRET")),
            "缺 JWT_SECRET 应被列出: {:?}",
            err.problems
        );

        // 过短的密钥同样被拒
        vars.insert("JWT_SECRET".to_string(), "short".to_string());
        let err = Config::from_env(vars).unwrap_err();
        assert!(
            err.problems.iter().any(|p| p.contains("32")),
            "过短密钥应被拒: {:?}",
            err.problems
        );
        let _ = AppEnv::Dev;
    }
}
