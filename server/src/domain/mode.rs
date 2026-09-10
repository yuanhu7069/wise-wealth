//! 模式库:配置驱动的 L1/L2 模式定义(ADR-B-001)。
//!
//! 「加一个模式 = 加一个 TOML 文件,不改代码」是本模块存在的全部理由。配置文件在**构建期**
//! 由 `build.rs` 扫描并 `include_str!` 嵌进二进制,启动期解析并校验 —— 单二进制自包含,
//! 且坏配置在启动期就暴露(而非等用户点到那一步才炸)。

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// 配置来源清单由构建期扫描 `config/modes/` 生成(见 `build.rs`)。
// 用行注释而非文档注释:rustdoc 不会给宏调用生成文档。
include!(concat!(env!("OUT_DIR"), "/mode_sources.rs"));

/// 出处可信度(产品 PRD §4.1.2)。用枚举而非字符串:写错成 "vrefied" 应在解析期被拒,
/// 而不是静默加载成一个没人认识的档位。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Credibility {
    /// 已考证:有明确出版物/机构来源
    Verified,
    /// 存疑:广泛流传但出处不成立
    Disputed,
    /// 谨慎:来源真实但个人不可复制,或存在应用边界
    Caution,
}

/// 规则桶可用的规则。枚举而非字符串白名单 —— 新规则必然要写实现,让它在编译期被强制。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    /// 应急金规则(RULE-009 / RULE-011 / RULE-012)
    EmergencyFund,
}

/// 桶的份额类型。TOML 里写成内部标签形式:`share = { type = "pct", basis_points = 3000 }`。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShareType {
    /// 固定支出桶:金额 = 月固定支出
    FixedExpenses,
    /// 比例桶:金额 = 税后月收入 × basis_points / 10000
    Pct {
        /// 万分比(3000 = 30%),整数避免浮点误差
        basis_points: i64,
    },
    /// 规则桶:金额由规则计算
    Rule {
        /// 使用的规则
        rule: RuleKind,
    },
    /// 余量桶:吃掉其余桶分配完毕后的剩余;一个模式至多一个
    Remainder,
}

/// 一个桶的定义。
#[derive(Debug, Clone, Deserialize)]
pub struct BucketSpec {
    /// 稳定标识,进数据库与 API
    pub id: String,
    /// 展示名(如「工资账户」)
    pub name: String,
    /// 用途一句话,直接展示在方案页的分配总览里
    #[serde(default)]
    pub purpose: String,
    /// 份额类型
    pub share: ShareType,
    /// 是否计入「必要月支出」(应急金目标的基数)。四账户 = 工资桶 + 消费桶。
    #[serde(default)]
    pub is_necessary: bool,
    /// L2 大类配置是否作用于此桶;一个模式至多一个
    #[serde(default)]
    pub is_investable: bool,
}

/// 应急金规则(RULE-009/011/012)。无规则桶的模式(如 50/30/20)同样需要它 ——
/// 达标标注与覆盖月数展示对两种模式都成立,区别只在于有没有一个桶真的按它转钱。
#[derive(Debug, Clone, Deserialize)]
pub struct EmergencyFundRule {
    /// 收入「很稳」对应的应急月数
    pub stable: i32,
    /// 收入「一般」
    pub normal: i32,
    /// 收入「波动大」
    pub volatile: i32,
    /// 收入「自由职业」
    pub freelance: i32,
    /// 需赡养人数 > 0 时上浮的档数(3→6→9→12 中的「一档」)
    #[serde(default = "one")]
    pub dependents_bump: i32,
    /// 应急月数上限
    #[serde(default = "twelve")]
    pub max_months: i32,
    /// 补齐缺口的规划月数(缺口 ÷ 本值 = 规则桶月转入)
    #[serde(default = "twenty_four")]
    pub pacing_months: i64,
}

fn one() -> i32 {
    1
}
fn twelve() -> i32 {
    12
}
fn twenty_four() -> i64 {
    24
}

/// 一个 L1 分账模式的完整定义。
#[derive(Debug, Clone, Deserialize)]
pub struct ModeConfig {
    /// 稳定标识(如 `four_accounts`)
    pub id: String,
    /// 展示名
    pub name: String,
    /// 一句话核心理念
    #[serde(default)]
    pub tagline: String,
    /// 出处可信度徽章
    pub credibility: Credibility,
    /// 适合人群标签
    #[serde(default)]
    pub fit_for: Vec<String>,
    /// 桶列表,顺序即求解顺序的展示顺序
    pub buckets: Vec<BucketSpec>,
    /// 应急金规则
    pub emergency_fund: EmergencyFundRule,
}

impl ModeConfig {
    /// 该模式的余量桶(至多一个,已在装载期校验)。
    pub fn remainder_bucket(&self) -> Option<&BucketSpec> {
        self.buckets
            .iter()
            .find(|b| matches!(b.share, ShareType::Remainder))
    }

    /// 该模式的规则桶。
    pub fn rule_bucket(&self) -> Option<&BucketSpec> {
        self.buckets
            .iter()
            .find(|b| matches!(b.share, ShareType::Rule { .. }))
    }

    /// 该模式的投资桶(L2 作用点)。
    pub fn investable_bucket(&self) -> Option<&BucketSpec> {
        self.buckets.iter().find(|b| b.is_investable)
    }
}

/// 一个 L2 大类配置。
#[derive(Debug, Clone, Deserialize)]
pub struct L2Config {
    /// 稳定标识(如 `sixty_forty`)
    pub id: String,
    /// 展示名
    pub name: String,
    /// 大类配置项,万分比之和必须为 10000
    pub classes: Vec<L2Class>,
    /// 附注(如核心-卫星的「卫星单个主题 ≤5%」约束)
    #[serde(default)]
    pub note: Option<String>,
}

/// L2 里的一个资产大类。**只到大类**,不出现任何具体产品(合规红线,产品 PRD §九)。
///
/// 同时实现 `Serialize`:匹配结果要随方案快照一起冻结进库,配置日后改动不影响历史方案。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct L2Class {
    /// 大类名(权益类/债券类/黄金/现金类…)
    pub name: String,
    /// 万分比
    pub basis_points: i64,
}

/// 模式库装载错误。启动期即失败(模式库是编译期资产,没有降级一说)。
#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    /// TOML 解析失败
    #[error("模式配置解析失败({id}): {detail}")]
    Parse {
        /// 出错的配置来源
        id: String,
        /// 解析器给出的细节
        detail: String,
    },
    /// 配置本身非法(校验不通过)
    #[error("模式配置非法({id}): {reason}")]
    Invalid {
        /// 出错的配置来源
        id: String,
        /// 违反的约束
        reason: String,
    },
}

/// 模式库:全部 L1 模式与 L2 配置的集合。
#[derive(Debug, Clone)]
pub struct ModeLibrary {
    modes: Vec<ModeConfig>,
    l2: Vec<L2Config>,
}

impl ModeLibrary {
    /// 装载内嵌配置。启动期调用一次,失败即启动失败。
    ///
    /// 除逐份校验外,还交叉校验「引擎引用的 L2 配置必须齐全」—— 否则要等用户点
    /// 「生成方案」才会发现缺档,而那时错误已经远离了它的成因。
    pub fn load_embedded() -> Result<Self, LibraryError> {
        let lib = Self::from_sources(EMBEDDED_MODES, EMBEDDED_L2)?;
        for id in crate::domain::l2::required_l2_ids() {
            if lib.l2(id).is_none() {
                return Err(LibraryError::Invalid {
                    id: "l2".to_string(),
                    reason: format!("引擎需要的 L2 配置缺失: {id}"),
                });
            }
        }
        Ok(lib)
    }

    /// 从给定文本装载(供测试注入自造配置)。
    pub fn from_sources(
        mode_sources: &[(&str, &str)],
        l2_sources: &[(&str, &str)],
    ) -> Result<Self, LibraryError> {
        let mut modes = Vec::with_capacity(mode_sources.len());
        for (id, text) in mode_sources {
            let mode: ModeConfig = toml::from_str(text).map_err(|e| LibraryError::Parse {
                id: (*id).to_string(),
                detail: e.to_string(),
            })?;
            validate_mode(&mode)?;
            modes.push(mode);
        }

        let mut l2 = Vec::with_capacity(l2_sources.len());
        for (id, text) in l2_sources {
            let cfg: L2Config = toml::from_str(text).map_err(|e| LibraryError::Parse {
                id: (*id).to_string(),
                detail: e.to_string(),
            })?;
            validate_l2(&cfg)?;
            l2.push(cfg);
        }

        Ok(Self { modes, l2 })
    }

    /// 全部 L1 模式(供模式列表与推荐使用)。
    pub fn modes(&self) -> &[ModeConfig] {
        &self.modes
    }

    /// 按 id 取 L1 模式。
    pub fn mode(&self, id: &str) -> Option<&ModeConfig> {
        self.modes.iter().find(|m| m.id == id)
    }

    /// 按 id 取 L2 配置。
    pub fn l2(&self, id: &str) -> Option<&L2Config> {
        self.l2.iter().find(|c| c.id == id)
    }
}

fn validate_mode(mode: &ModeConfig) -> Result<(), LibraryError> {
    let invalid = |reason: String| LibraryError::Invalid {
        id: mode.id.clone(),
        reason,
    };

    if mode.buckets.is_empty() {
        return Err(invalid("至少需要一个桶".into()));
    }

    let mut seen_ids = HashSet::new();
    for b in &mode.buckets {
        if !seen_ids.insert(b.id.as_str()) {
            return Err(invalid(format!("桶 id 重复: {}", b.id)));
        }
        if let ShareType::Pct { basis_points } = b.share
            && !(0..10_000).contains(&basis_points)
        {
            return Err(invalid(format!(
                "比例桶 {} 的 basis_points 必须在 (0, 10000) 内,实为 {basis_points}",
                b.id
            )));
        }
    }

    let remainders = mode
        .buckets
        .iter()
        .filter(|b| matches!(b.share, ShareType::Remainder))
        .count();
    if remainders > 1 {
        return Err(invalid(format!(
            "余量桶至多一个,实为 {remainders} 个 —— 两个余量桶无法唯一求解"
        )));
    }

    // 规则桶的金额要从余量中让位。没有余量桶就没有来源:比例桶已占满 100% 时,
    // 规则桶仍会拿到非零金额,分配总额超过收入(safety_first 整段被跳过)。
    let has_rule_bucket = mode
        .buckets
        .iter()
        .any(|b| matches!(b.share, ShareType::Rule { .. }));
    if has_rule_bucket && remainders == 0 {
        return Err(invalid(
            "有规则桶的模式必须同时有余量桶 —— 规则桶的金额从余量中让位,无余量则会超配收入".into(),
        ));
    }

    let investables = mode.buckets.iter().filter(|b| b.is_investable).count();
    if investables > 1 {
        return Err(invalid(format!(
            "标注 is_investable 的桶至多一个,实为 {investables} 个"
        )));
    }

    // 比例桶之和必须小于 100%,余量桶才有正份额可吃;无余量桶时允许恰好 100%。
    let pct_sum: i64 = mode
        .buckets
        .iter()
        .filter_map(|b| match b.share {
            ShareType::Pct { basis_points } => Some(basis_points),
            _ => None,
        })
        .sum();
    if remainders == 0 && pct_sum != 10_000 {
        return Err(invalid(format!(
            "无余量桶时比例桶之和必须为 10000,实为 {pct_sum}"
        )));
    }
    if remainders == 1 && pct_sum >= 10_000 {
        return Err(invalid(format!(
            "有余量桶时比例桶之和必须小于 10000,实为 {pct_sum}"
        )));
    }

    let rule = &mode.emergency_fund;
    for (name, v) in [
        ("stable", rule.stable),
        ("normal", rule.normal),
        ("volatile", rule.volatile),
        ("freelance", rule.freelance),
    ] {
        if v <= 0 {
            return Err(invalid(format!("应急月数 {name} 必须为正,实为 {v}")));
        }
    }
    if rule.pacing_months <= 0 {
        return Err(invalid(format!(
            "pacing_months 必须为正,实为 {}",
            rule.pacing_months
        )));
    }
    if rule.max_months < rule.stable {
        return Err(invalid("max_months 不可小于起始月数".into()));
    }

    Ok(())
}

fn validate_l2(cfg: &L2Config) -> Result<(), LibraryError> {
    let invalid = |reason: String| LibraryError::Invalid {
        id: cfg.id.clone(),
        reason,
    };
    if cfg.classes.is_empty() {
        return Err(invalid("至少需要一个大类".into()));
    }
    if cfg.classes.iter().any(|c| c.basis_points < 0) {
        return Err(invalid("大类比例不可为负".into()));
    }
    let sum: i64 = cfg.classes.iter().map(|c| c.basis_points).sum();
    if sum != 10_000 {
        return Err(invalid(format!("大类比例之和必须为 10000,实为 {sum}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 内嵌配置可装载且含两个模式() {
        let lib = ModeLibrary::load_embedded().expect("内嵌配置必须可装载");
        let ids: Vec<&str> = lib.modes().iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains(&"four_accounts"), "缺四账户模式: {ids:?}");
        assert!(ids.contains(&"fifty_30_20"), "缺 50/30/20 模式: {ids:?}");
    }

    #[test]
    fn 四账户的桶结构与语义角色正确() {
        let lib = ModeLibrary::load_embedded().unwrap();
        let m = lib.mode("four_accounts").unwrap();

        assert_eq!(m.remainder_bucket().map(|b| b.id.as_str()), Some("invest"));
        assert_eq!(m.rule_bucket().map(|b| b.id.as_str()), Some("reserve"));
        assert_eq!(m.investable_bucket().map(|b| b.id.as_str()), Some("invest"));
        assert_eq!(m.credibility, Credibility::Verified);

        // 必要月支出 = 工资桶 + 消费桶
        let necessary: Vec<&str> = m
            .buckets
            .iter()
            .filter(|b| b.is_necessary)
            .map(|b| b.id.as_str())
            .collect();
        assert_eq!(necessary, vec!["salary", "spend"]);
    }

    #[test]
    fn 五十三十二十无余量桶且储蓄桶兼作投资桶() {
        let lib = ModeLibrary::load_embedded().unwrap();
        let m = lib.mode("fifty_30_20").unwrap();
        assert!(m.remainder_bucket().is_none(), "50/30/20 不应有余量桶");
        assert!(
            m.rule_bucket().is_none(),
            "50/30/20 不应有规则桶(储蓄桶是比例桶)"
        );
        assert_eq!(m.investable_bucket().map(|b| b.id.as_str()), Some("savings"));
    }

    #[test]
    fn 四个_l2_配置齐全且比例合计为一万() {
        let lib = ModeLibrary::load_embedded().unwrap();
        for id in crate::domain::l2::required_l2_ids() {
            let c = lib.l2(id).unwrap_or_else(|| panic!("缺 L2 配置: {id}"));
            let sum: i64 = c.classes.iter().map(|x| x.basis_points).sum();
            assert_eq!(sum, 10_000, "{id} 比例合计应为 10000");
        }
    }

    #[test]
    fn 双余量桶被拒绝() {
        let bad = r#"
id = "two_remainders"
name = "坏模式"
credibility = "verified"
buckets = [
  { id = "a", name = "A", share = { type = "remainder" } },
  { id = "b", name = "B", share = { type = "remainder" } },
]
[emergency_fund]
stable = 3
normal = 6
volatile = 9
freelance = 9
"#;
        let err = ModeLibrary::from_sources(&[("two_remainders", bad)], &[]).unwrap_err();
        assert!(
            err.to_string().contains("余量桶至多一个"),
            "错误信息应指出余量桶问题: {err}"
        );
    }

    #[test]
    fn 规则桶而余量桶时被拒绝() {
        // 比例桶占满 100% + 有规则桶:规则桶的钱没有来源,会超配收入
        let bad = r#"
id = "rule_without_remainder"
name = "坏模式"
credibility = "verified"
buckets = [
  { id = "a", name = "A", share = { type = "pct", basis_points = 6000 }, is_necessary = true },
  { id = "b", name = "B", share = { type = "pct", basis_points = 4000 } },
  { id = "r", name = "R", share = { type = "rule", rule = "emergency_fund" } },
]
[emergency_fund]
stable = 3
normal = 6
volatile = 9
freelance = 9
"#;
        let err =
            ModeLibrary::from_sources(&[("rule_without_remainder", bad)], &[]).unwrap_err();
        assert!(
            err.to_string().contains("必须同时有余量桶"),
            "实际: {err}"
        );
    }

    #[test]
    fn 未知规则在解析期被拒() {
        let bad = r#"
id = "unknown_rule"
name = "坏模式"
credibility = "verified"
buckets = [
  { id = "a", name = "A", share = { type = "rule", rule = "no_such_rule" } },
  { id = "b", name = "B", share = { type = "remainder" } },
]
[emergency_fund]
stable = 3
normal = 6
volatile = 9
freelance = 9
"#;
        let err = ModeLibrary::from_sources(&[("unknown_rule", bad)], &[]).unwrap_err();
        match err {
            LibraryError::Parse { .. } => {}
            other => panic!("未知规则应在解析期被拒,实际: {other}"),
        }
    }

    #[test]
    fn 未知可信度在解析期被拒() {
        let bad = r#"
id = "bad_credibility"
name = "坏模式"
credibility = "vrefied"
buckets = [{ id = "a", name = "A", share = { type = "remainder" } }]
[emergency_fund]
stable = 3
normal = 6
volatile = 9
freelance = 9
"#;
        let err = ModeLibrary::from_sources(&[("bad_credibility", bad)], &[]).unwrap_err();
        assert!(
            matches!(err, LibraryError::Parse { .. }),
            "拼错的可信度应被解析期拒绝,实际: {err}"
        );
    }

    #[test]
    fn 无余量桶时比例之和必须满一百() {
        let bad = r#"
id = "short_pct"
name = "坏模式"
credibility = "verified"
buckets = [
  { id = "a", name = "A", share = { type = "pct", basis_points = 5000 } },
]
[emergency_fund]
stable = 3
normal = 6
volatile = 9
freelance = 9
"#;
        let err = ModeLibrary::from_sources(&[("short_pct", bad)], &[]).unwrap_err();
        assert!(err.to_string().contains("10000"), "实际: {err}");
    }

    #[test]
    fn l2_比例合计不为一万被拒绝() {
        let bad = r#"
id = "broken_l2"
name = "坏配置"
classes = [
  { name = "权益类", basis_points = 6000 },
]
"#;
        let err = ModeLibrary::from_sources(&[], &[("broken_l2", bad)]).unwrap_err();
        assert!(err.to_string().contains("10000"), "实际: {err}");
    }

    #[test]
    fn 配置损坏时给出可读错误而非崩溃() {
        let err = ModeLibrary::from_sources(&[("broken", "这不是 TOML {{{")], &[]).unwrap_err();
        match err {
            LibraryError::Parse { id, .. } => assert_eq!(id, "broken"),
            other => panic!("应报解析错误,实际: {other}"),
        }
    }
}
