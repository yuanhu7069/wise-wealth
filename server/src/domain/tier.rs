//! 产品分层(RULE-045/046,ADR-H-001):「怎么做」免费,「为什么」Plus。
//!
//! 分层字段自迁移 0002 预留,H 期启用。写入方本期只有运维脚本
//! (`scripts/set-tier.sh`,产品内无购买路径);读取方是 plans service 的
//! **实时**闸门 —— 与方案版本无关,置 plus 后刷新即得推理链。

use serde::{Deserialize, Serialize};

/// 账号分层。未知库值按 `Free` 降级(fail-safe 方向 = 给得少,ADR-H-001)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    /// 免费层:方案数字可见,推理链只有条数与占位
    Free,
    /// Plus 层:推理链全量(本期唯一开通方式 = 运维脚本)
    Plus,
}

impl Tier {
    /// 库值 → 分层。识别面与 0002 迁移 COMMENT 的预留语义一致。
    pub fn parse(raw: &str) -> Self {
        match raw {
            "plus" => Tier::Plus,
            _ => Tier::Free,
        }
    }

    pub const fn is_plus(self) -> bool {
        matches!(self, Tier::Plus)
    }

    /// 库里的字面值(诊断日志用,不进 API)。
    pub const fn as_str(self) -> &'static str {
        match self {
            Tier::Free => "free",
            Tier::Plus => "plus",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 库值解析且未知值降级为_free() {
        assert_eq!(Tier::parse("free"), Tier::Free);
        assert_eq!(Tier::parse("plus"), Tier::Plus);
        // fail-safe:库值再离谱也只当 free(给得少),不报错不猜测
        assert_eq!(Tier::parse("enterprise"), Tier::Free);
        assert_eq!(Tier::parse(""), Tier::Free);
    }

    #[test]
    fn is_plus_只对_plus_为真() {
        assert!(Tier::Plus.is_plus());
        assert!(!Tier::Free.is_plus());
    }
}
