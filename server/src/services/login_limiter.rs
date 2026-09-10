//! 登录失败限流(RULE-002)。
//!
//! 按**用户名**计数而非来源 IP:本项目前端经 Next 的 BFF 代理转发,后端看到的对端地址
//! 恒为 BFF 自身 —— 按 IP 计数等于把所有请求算作同一个来源,限流形同虚设。
//! 单用户场景下,有意义的轴是「正在被尝试的账号」。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 窗口内允许的失败次数上限;第 6 次尝试直接拒绝,不比对口令。
const MAX_FAILURES: usize = 5;
/// 计数窗口
const WINDOW: Duration = Duration::from_secs(60);

/// 内存滑动窗口计数器。进程内状态:重启即清空(可接受 —— 重启本身已经打断了爆破节奏)。
#[derive(Clone, Default)]
pub struct LoginLimiter {
    inner: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl LoginLimiter {
    /// 窗口内失败次数是否已达上限。
    pub fn is_blocked(&self, username: &str) -> bool {
        self.with_entry(username, |times| {
            times.retain(|t| t.elapsed() < WINDOW);
            times.len() >= MAX_FAILURES
        })
    }

    /// 记录一次失败尝试。
    pub fn record_failure(&self, username: &str) {
        self.with_entry(username, |times| {
            times.retain(|t| t.elapsed() < WINDOW);
            times.push(Instant::now());
        });
    }

    /// 登录成功后清零该账号的失败计数。
    pub fn clear(&self, username: &str) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.remove(username);
        }
    }

    /// 取锁、修剪窗口、执行闭包。锁中毒时取回内部值继续 —— 限流计数不该因
    /// 别处 panic 而永久失效。
    fn with_entry<T>(&self, username: &str, f: impl FnOnce(&mut Vec<Instant>) -> T) -> T {
        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let entry = guard.entry(username.to_string()).or_default();
        f(entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 五次失败后封锁() {
        let limiter = LoginLimiter::default();
        assert!(!limiter.is_blocked("苑问"));
        for _ in 0..MAX_FAILURES {
            limiter.record_failure("苑问");
        }
        assert!(limiter.is_blocked("苑问"), "第 6 次尝试应被拒绝");
    }

    #[test]
    fn 成功登录清零计数() {
        let limiter = LoginLimiter::default();
        for _ in 0..MAX_FAILURES {
            limiter.record_failure("苑问");
        }
        assert!(limiter.is_blocked("苑问"));
        limiter.clear("苑问");
        assert!(!limiter.is_blocked("苑问"), "成功登录后应重新计数");
    }

    #[test]
    fn 计数按账号隔离() {
        let limiter = LoginLimiter::default();
        for _ in 0..MAX_FAILURES {
            limiter.record_failure("甲");
        }
        assert!(limiter.is_blocked("甲"));
        assert!(!limiter.is_blocked("乙"), "别的账号不该被连坐");
    }
}
