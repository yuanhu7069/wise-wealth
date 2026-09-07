//! 入口:装配 State、路由、tracing(request_id)、优雅退出、迁移执行。
//!
//! RULE-002:配置校验失败 → 逐条列出缺失键名(ERR-003 终端文案)→ 退出码 1。

mod api;
mod config;
mod error;
mod openapi;
pub mod services;
mod state;

use axum::http::HeaderValue;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // 1. 配置校验(RULE-002):失败即逐条列出问题并退出
    let config = match config::Config::load() {
        Ok(c) => c,
        Err(e) => {
            // ERR-003 终端文案(prd-a.md §8.4)
            eprintln!("缺少环境变量或有非法配置:");
            for p in &e.problems {
                eprintln!("  - {p}");
            }
            eprintln!("复制 .env.example 为 .env 并填写后重新启动。");
            std::process::exit(1);
        }
    };

    // 2. 日志初始化(RUST_LOG 缺省 info);配置摘要只输出「已配置/缺失」状态(脱敏红线)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_new(&config.rust_log).unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
    tracing::info!("{}", config.summary());

    // 3. 数据库连接池(惰性:库不可达不阻塞启动,health 如实反映 db=error)
    let app_state = state::AppState::new(&config)
        .unwrap_or_else(|e| {
            tracing::error!("连接参数解析失败(连接串不打印): {e:#}");
            std::process::exit(1);
        });

    // 4. 迁移(空基线迁移,打通链路;失败仅告警不退出,health 会如实反映 db=error)
    run_migrations(&app_state.pool).await;

    // 5. 路由 + 安全响应头 + tracing(含 request_id)
    let app = api::routes(app_state)
        .layer(
            tower_http::set_header::SetResponseHeaderLayer::overriding(
                axum::http::header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            ),
        )
        .layer(TraceLayer::new_for_http());

    // 6. 监听 + 优雅退出
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], config.app_port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("端口 {} 绑定失败: {e}", config.app_port);
            std::process::exit(1);
        });
    tracing::info!("后端已监听 http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap_or_else(|e| {
            tracing::error!("服务运行异常退出: {e}");
            std::process::exit(1);
        });
}

async fn run_migrations(pool: &sqlx::PgPool) {
    let migrator = sqlx::migrate!("./migrations");
    match migrator.run(pool).await {
        Ok(_) => tracing::info!("数据库迁移:全部已应用"),
        Err(e) => tracing::warn!("数据库迁移失败(不阻塞启动,health 将反映 db=error): {e}"),
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        // 信号 handler 安装失败属进程初始化缺陷,记日志后退出(exit 非 panic,基线 §16.6)
        if tokio::signal::ctrl_c().await.is_err() {
            tracing::error!("安装 Ctrl+C handler 失败");
            std::process::exit(1);
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(e) => {
                tracing::error!("安装 SIGTERM handler 失败: {e}");
                std::process::exit(1);
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("收到 Ctrl+C,开始优雅退出"),
        _ = terminate => tracing::info!("收到 SIGTERM,开始优雅退出"),
    }
}
