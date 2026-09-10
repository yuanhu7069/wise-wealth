//! AppState(PgPool + 模式库 + 配置 + 登录限流)。PgPoolOptions 显式最大连接数与超时(arch-a.md T2-2)。

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::domain::ModeLibrary;
use crate::services::login_limiter::LoginLimiter;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    /// 模式库:构建期内嵌、启动期解析校验(ADR-B-001)。
    /// 放进 AppState 而非每次现读:它是只读资产,且坏配置必须在启动期就拦下。
    pub library: ModeLibrary,
    /// 应用配置(JWT 密钥、会话有效期等在此取用)
    pub config: Arc<Config>,
    /// 登录失败限流(RULE-002)。进程内状态。
    pub login_limiter: LoginLimiter,
    /// 版本号(来自 CARGO_PKG_VERSION),health 响应携带
    pub version: &'static str,
}

impl AppState {
    /// 建池:显式 max_connections / acquire_timeout。
    /// 使用惰性连接(connect_lazy_with):启动时数据库不可达**不阻塞启动**,
    /// 运行期库故障由 /health 探测如实反映 db=error(arch-a.md §6「服务进程不退出」、ADR-A-003)。
    pub fn new(config: &Config, library: ModeLibrary) -> Result<Self, sqlx::Error> {
        // 显式连接参数,避免 URL 中口令含未编码特殊字符时解析失败
        let opts: PgConnectOptions = config.database_url.parse()?;
        let pool = PgPoolOptions::new()
            .max_connections(5) // 单人使用,小而够;并发提升时按需调
            .min_connections(0)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(300))
            .max_lifetime(Duration::from_secs(1800))
            .connect_lazy_with(opts);
        Ok(Self {
            pool,
            library,
            config: Arc::new(config.clone()),
            login_limiter: LoginLimiter::default(),
            version: env!("CARGO_PKG_VERSION"),
        })
    }
}
