//! 应用配置(RULE-002:启动期校验必需环境变量,缺失即拒绝启动并逐条列出键名)。
//!
//! 脱敏要求(prd-a.md §11 / arch-a.md §8):配置摘要只输出「已配置/缺失」状态,
//! 禁止输出任何环境变量值(数据库口令、连接串不得进入日志)。

use std::env;

/// 应用配置。字段值禁止 Debug/Display 打印(脱敏红线),仅提供 `summary()` 安全摘要。
#[derive(Debug, Clone)]
pub struct Config {
    /// dev / prod(基线 §12)
    pub app_env: AppEnv,
    /// 后端监听端口
    pub app_port: u16,
    /// 当前环境对应的数据库连接串(本结构体内流转,禁入日志)
    pub database_url: String,
    /// API_BASE_URL(前端 BFF 指向后端,后端仅读取不校验值)
    pub api_base_url_configured: bool,
    /// RUST_LOG(缺省 info)
    pub rust_log: String,
    /// JWT 签名密钥(B 期,ADR-B-002)。长度不足 32 字符即拒绝启动 —— 过短的签名密钥等于没有。
    pub jwt_secret: String,
    /// 会话有效期(天),缺省 30
    pub session_ttl_days: i64,
    /// 种子账号用户名(可选:仅 `seed-user` 子命令消费,不设置不影响服务启动)
    pub seed_username: Option<String>,
    /// 种子账号口令(可选:同上;只进 argon2 哈希,禁入日志)
    pub seed_password: Option<String>,
    /// 偏离提醒阈值(万分比,2000 = 20%;ADR-E-003)。缺省 2000,
    /// 非法值按配置错误拒绝启动 —— 显式写错的配置不该被静默吞成默认值。
    pub snapshot_deviation_threshold_bp: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Dev,
    Prod,
}

impl std::fmt::Display for AppEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppEnv::Dev => write!(f, "dev"),
            AppEnv::Prod => write!(f, "prod"),
        }
    }
}

/// RULE-002 校验失败:携带全部缺失/违规键名,由 main.rs 逐条输出后退出。
#[derive(Debug, thiserror::Error)]
#[error("配置校验失败")]
pub struct ConfigError {
    pub problems: Vec<String>,
}

impl Config {
    /// 从进程环境读取配置并校验(RULE-002)。
    ///
    /// - `APP_ENV` 必填,取值 dev/prod;
    /// - `APP_PORT` 必填,u16;
    /// - `APP_ENV=dev` 时 `DATABASE_URL_DEV` 必填,prod 时 `DATABASE_URL_PROD` 必填;
    /// - 两键同时配置时必须指向不同数据库(开发库与正式库分离)。
    pub fn load() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv(); // .env 缺失不算错误:环境变量也可来自 shell
        Self::from_env(env::vars().collect())
    }

    /// 纯函数入口,便于单测(缺单键 / 缺多键 / dev-prod 同库 三用例,AC-2)。
    pub fn from_env(vars: std::collections::HashMap<String, String>) -> Result<Self, ConfigError> {
        let mut problems: Vec<String> = Vec::new();

        let get = |k: &str| -> Option<String> {
            vars.get(k)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };

        let app_env_str = get("APP_ENV");
        let app_port_str = get("APP_PORT");
        let db_dev = get("DATABASE_URL_DEV");
        let db_prod = get("DATABASE_URL_PROD");
        let rust_log = get("RUST_LOG").unwrap_or_else(|| "info".to_string());
        let api_base_configured = get("API_BASE_URL").is_some();
        let jwt_secret_raw = get("JWT_SECRET");
        let session_ttl_days = get("SESSION_TTL_DAYS")
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|d| *d > 0)
            .unwrap_or(30);
        let seed_username = get("SEED_USERNAME");
        let seed_password = get("SEED_PASSWORD");

        // E 期(ADR-E-003):偏离阈值,万分比整数,缺省 2000 = 20%。
        // 上限 10000(= 100%):超过它的阈值在语义上等于「永不提醒」,
        // 与其当成合法配置,不如启动期就拦下。
        let snapshot_deviation_threshold_bp = match get("SNAPSHOT_DEVIATION_THRESHOLD_BP") {
            Some(raw) => match raw.parse::<i64>() {
                Ok(bp) if (1..=10_000).contains(&bp) => bp,
                Ok(bp) => {
                    problems.push(format!(
                        "SNAPSHOT_DEVIATION_THRESHOLD_BP(必须在 1..=10000 内,实为 {bp})"
                    ));
                    2_000
                }
                Err(_) => {
                    problems.push(
                        "SNAPSHOT_DEVIATION_THRESHOLD_BP(必须是整数万分比,如 2000 = 20%)"
                            .to_string(),
                    );
                    2_000
                }
            },
            None => 2_000,
        };

        // 1. APP_ENV
        let app_env = match app_env_str.as_deref() {
            Some("dev") => Some(AppEnv::Dev),
            Some("prod") => Some(AppEnv::Prod),
            Some(other) => {
                problems.push(format!("APP_ENV(值非法:「{other}」,允许 dev|prod)"));
                None
            }
            None => {
                problems.push("APP_ENV(缺失)".to_string());
                None
            }
        };

        // 2. APP_PORT
        let app_port = match app_port_str
            .as_deref()
            .map(str::parse::<u16>)
        {
            Some(Ok(p)) if p > 0 => Some(p),
            Some(Ok(_)) => {
                problems.push("APP_PORT(端口不能为 0)".to_string());
                None
            }
            Some(Err(e)) => {
                problems.push(format!("APP_PORT(不是合法端口号:{e})"));
                None
            }
            None => {
                problems.push("APP_PORT(缺失)".to_string());
                None
            }
        };

        // 3. JWT_SECRET(B 期新增,ADR-B-002):会话签名密钥,短于 32 字符即视为未配置。
        let jwt_secret = match jwt_secret_raw {
            Some(s) if s.chars().count() >= 32 => Some(s),
            Some(_) => {
                problems.push("JWT_SECRET(长度不足 32 字符:签名密钥过短等于没有)".to_string());
                None
            }
            None => {
                problems.push("JWT_SECRET(缺失)".to_string());
                None
            }
        };

        // 4. dev/prod 同库检测(两键都配置时强制不同库;仅配置一键则跳过)
        // 必须在 move 之前以引用完成判定
        let db_same = matches!((&db_dev, &db_prod), (Some(d), Some(p)) if same_database(d, p));
        if db_same {
            problems.push(
                "DATABASE_URL_DEV / DATABASE_URL_PROD 指向同一个数据库(RULE-002:开发库与正式库必须分离)"
                    .to_string(),
            );
        }

        // 3. 环境对应库名必填(此处 move db_dev/db_prod)。
        // APP_ENV 无法确定时(dev/prod 两分支都不成立),仍检查 DATABASE_URL_DEV:
        // 一次性把可判定的缺失键都列出来(AC-2:逐条列出缺失键名)。
        let database_url = match app_env {
            Some(AppEnv::Dev) => {
                match db_dev {
                    Some(url) => Some(url),
                    None => {
                        problems.push("DATABASE_URL_DEV(APP_ENV=dev 时必填)".to_string());
                        None
                    }
                }
            }
            Some(AppEnv::Prod) => {
                match db_prod {
                    Some(url) => Some(url),
                    None => {
                        problems.push("DATABASE_URL_PROD(APP_ENV=prod 时必填)".to_string());
                        None
                    }
                }
            }
            None => {
                if db_dev.is_none() {
                    problems.push("DATABASE_URL_DEV(APP_ENV=dev 时必填)".to_string());
                }
                None
            }
        };

        if !problems.is_empty() {
            return Err(ConfigError { problems });
        }

        // APP_ENV/APP_PORT 缺失时已归入 problems 并提前返回;此处 match 归一化
        // (基线 §16.6:handler/启动路径禁止 unwrap,构造路径用穷尽 match 替代 expect)。
        let app_env = match app_env {
            Some(env) => env,
            None => {
                problems.push("APP_ENV(缺失)".to_string());
                return Err(ConfigError { problems });
            }
        };
        let app_port = match app_port {
            Some(port) => port,
            None => {
                problems.push("APP_PORT(缺失)".to_string());
                return Err(ConfigError { problems });
            }
        };
        let database_url = match database_url {
            Some(url) => url,
            None => {
                problems.push("DATABASE_URL(APP_ENV 对应库名未配置)".to_string());
                return Err(ConfigError { problems });
            }
        };
        let jwt_secret = match jwt_secret {
            Some(s) => s,
            None => {
                problems.push("JWT_SECRET(缺失)".to_string());
                return Err(ConfigError { problems });
            }
        };

        Ok(Self {
            app_env,
            app_port,
            database_url,
            api_base_url_configured: api_base_configured,
            rust_log,
            jwt_secret,
            session_ttl_days,
            seed_username,
            seed_password,
            snapshot_deviation_threshold_bp,
        })
    }

    /// 启动摘要:只输出「已配置/缺失」状态,不含任何值(基线红线 2/9)。
    pub fn summary(&self) -> String {
        format!(
            "配置摘要: APP_ENV={} | APP_PORT=已配置 | DATABASE_URL_{}=已配置 | API_BASE_URL={} | JWT_SECRET=已配置 | 会话有效期={}天 | 种子账号={} | RUST_LOG=已配置(值不打印)",
            self.app_env,
            if self.app_env == AppEnv::Dev { "DEV" } else { "PROD" },
            if self.api_base_url_configured { "已配置" } else { "缺失(可选)" },
            self.session_ttl_days,
            if self.seed_username.is_some() { "已配置" } else { "未配置(可选)" },
        )
    }
}

/// 判断两条连接串是否指向同一数据库:
/// 归一化比较 scheme://host:port/db(大小写不敏感,忽略查询参数、口令差异与末尾斜杠)。
/// 若任一串无法解析,退化为整串精确比较(保守判定)。
fn same_database(a: &str, b: &str) -> bool {
    match (parse_pg_target(a), parse_pg_target(b)) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

/// 从 postgres://user:pass@host:port/dbname?params 提取 (host, port, dbname)。
fn parse_pg_target(url: &str) -> Option<(String, u16, String)> {
    let rest = url
        .strip_prefix("postgres://")
        .or_else(|| url.strip_prefix("postgresql://"))?;
    let after_at = rest.rsplit_once('@')?.1;
    let (hostport, path) = after_at.split_once('/')?;
    let dbname = path.split('?').next()?.trim_end_matches('/');
    let (host, port) = match hostport.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().ok()?),
        None => (hostport, 5432),
    };
    Some((host.to_ascii_lowercase(), port, dbname.to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_vars() -> std::collections::HashMap<String, String> {
        std::collections::HashMap::from([
            ("APP_ENV".to_string(), "dev".to_string()),
            ("APP_PORT".to_string(), "8080".to_string()),
            (
                "DATABASE_URL_DEV".to_string(),
                "postgres://u:p@db.example.com:5432/wise_wealth_dev".to_string(),
            ),
            ("API_BASE_URL".to_string(), "http://127.0.0.1:8080".to_string()),
            // B 期新增的必填项:会话签名密钥(ADR-B-002)
            (
                "JWT_SECRET".to_string(),
                "test-secret-at-least-32-characters-long".to_string(),
            ),
        ])
    }

    /// AC-2 / RULE-002:缺单键(删掉 DATABASE_URL_DEV)→ 启动失败且错误信息含键名
    #[test]
    fn missing_single_key_fails_with_key_name() {
        let mut vars = base_vars();
        vars.remove("DATABASE_URL_DEV");
        let err = Config::from_env(vars).unwrap_err();
        assert!(err.problems.iter().any(|p| p.contains("DATABASE_URL_DEV")));
    }

    /// AC-2 / RULE-002:缺多键(APP_ENV、APP_PORT、DATABASE_URL_DEV 全缺)→ 一次性列出全部缺失键名
    #[test]
    fn missing_multiple_keys_lists_all() {
        let mut vars = base_vars();
        vars.remove("APP_ENV");
        vars.remove("APP_PORT");
        vars.remove("DATABASE_URL_DEV");
        let err = Config::from_env(vars).unwrap_err();
        assert!(err.problems.iter().any(|p| p.contains("APP_ENV")));
        assert!(err.problems.iter().any(|p| p.contains("APP_PORT")));
        assert!(err.problems.iter().any(|p| p.contains("DATABASE_URL_DEV")));
        assert_eq!(err.problems.len(), 3);
    }

    /// AC-2 / RULE-002:dev 与 prod 指向同一库 → 拒绝启动
    #[test]
    fn same_database_for_dev_and_prod_rejected() {
        let mut vars = base_vars();
        vars.insert("APP_ENV".to_string(), "prod".to_string());
        vars.insert(
            "DATABASE_URL_PROD".to_string(),
            "postgres://u2:p2@db.example.com:5432/wise_wealth_dev".to_string(),
        );
        let err = Config::from_env(vars).unwrap_err();
        assert!(err
            .problems
            .iter()
            .any(|p| p.contains("同一个数据库") || p.contains("分离")));
    }

    /// 合法配置应通过(顺带覆盖 prod 分支)
    #[test]
    fn valid_prod_config_loads() {
        let mut vars = base_vars();
        vars.insert("APP_ENV".to_string(), "prod".to_string());
        vars.insert(
            "DATABASE_URL_PROD".to_string(),
            "postgres://u2:p2@db.example.com:5432/wise_wealth_prod".to_string(),
        );
        let cfg = Config::from_env(vars).expect("应通过");
        assert_eq!(cfg.app_env, AppEnv::Prod);
        assert_eq!(cfg.app_port, 8080);
    }

    /// same_database:同库不同口令/大小写应判定为同库;不同库名判定为不同库
    #[test]
    fn same_database_normalization() {
        assert!(same_database(
            "postgres://u:p@DB.example.com:5432/wise_wealth_dev?sslmode=disable",
            "postgres://other:other@db.example.com:5432/Wise_Wealth_Dev"
        ));
        assert!(!same_database(
            "postgres://u:p@db.example.com:5432/wise_wealth_dev",
            "postgres://u:p@db.example.com:5432/wise_wealth_prod"
        ));
    }

    // ── E 期:偏离阈值(ADR-E-003)──

    #[test]
    fn 偏离阈值缺省两千() {
        let cfg = Config::from_env(base_vars()).expect("应通过");
        assert_eq!(cfg.snapshot_deviation_threshold_bp, 2_000);
    }

    #[test]
    fn 偏离阈值合法自定义生效() {
        let mut vars = base_vars();
        vars.insert("SNAPSHOT_DEVIATION_THRESHOLD_BP".to_string(), "1000".to_string());
        let cfg = Config::from_env(vars).expect("应通过");
        assert_eq!(cfg.snapshot_deviation_threshold_bp, 1_000);
    }

    #[test]
    fn 偏离阈值越界被拒且列出键名() {
        for bad in ["0", "10001", "-500", "twenty"] {
            let mut vars = base_vars();
            vars.insert("SNAPSHOT_DEVIATION_THRESHOLD_BP".to_string(), bad.to_string());
            let err = Config::from_env(vars).unwrap_err();
            assert!(
                err.problems.iter().any(|p| p.contains("SNAPSHOT_DEVIATION_THRESHOLD_BP")),
                "非法值 {bad} 应被列出: {:?}",
                err.problems
            );
        }
    }
}
