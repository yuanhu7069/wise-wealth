//! L2 大类配置匹配(RULE-015)。
//!
//! 匹配只在**已校验的模式库**上做:基础档由回撤反应决定,久期做降档修正,久期不足一年
//! 直接覆盖为现金类。收入稳定性不参与 L2 —— 它只影响应急月数(那是 L1 的事)。

use crate::domain::mode::{L2Class, L2Config, ModeLibrary};
use crate::domain::profile::{DrawdownResponse, Horizon, Profile};
use serde::{Deserialize, Serialize};

/// 权益占比从低到高的基础档阶梯。**顺序即语义**,降档就是在此数组里向前退一格。
///
/// 阶梯放在代码而非配置里,是因为它是 **RULE-015 这条业务规则**本身(回撤反应 → 档位),
/// 配置只提供每一档的具体比例。装载期会校验这些 id 在模式库中确实存在。
const TIER_LADDER: &[&str] = &["permanent_portfolio", "sixty_forty", "core_satellite"];

/// 久期不足一年时使用的覆盖档。
const CASH_OVER_L2_ID: &str = "cash_only";

/// 引擎引用的全部 L2 配置 id。`ModeLibrary::load_embedded` 据此校验库的完整性 ——
/// 缺一档就要等用户点「生成方案」才炸,而那时错误已远离成因。
pub fn required_l2_ids() -> impl Iterator<Item = &'static str> {
    TIER_LADDER
        .iter()
        .copied()
        .chain(std::iter::once(CASH_OVER_L2_ID))
}

/// 匹配结果:既供页面渲染,也整体进方案快照(方案一旦生成,配置再改也不影响历史)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct L2Allocation {
    /// 配置 id
    pub id: String,
    /// 展示名
    pub name: String,
    /// 大类与占比(万分比,**只到大类**)
    pub classes: Vec<L2Class>,
    /// 配置附注(可为空)
    pub note: Option<String>,
    /// 匹配理由(展示在方案页「投资账户内部配置」段)
    pub reason: String,
}

impl L2Allocation {
    /// 由配置与理由构造。两处匹配分支共用,避免各写一遍字段搬运。
    fn from_config(cfg: &L2Config, reason: String) -> Self {
        Self {
            id: cfg.id.clone(),
            name: cfg.name.clone(),
            classes: cfg.classes.clone(),
            note: cfg.note.clone(),
            reason,
        }
    }

    /// 是否为「久期不足一年」的现金类覆盖档。引擎据此发提示,不重复硬编码 id 字符串。
    pub fn is_cash_override(&self) -> bool {
        self.id == CASH_OVER_L2_ID
    }
}

/// L2 匹配失败:只可能因为模式库缺配置,而模式库在启动期已校验,故属程序缺陷。
#[derive(Debug, thiserror::Error)]
pub enum L2Error {
    /// 模式库中缺少所需配置
    #[error("模式库缺少 L2 配置: {0}")]
    MissingConfig(String),
}

/// 按回撤反应与久期匹配 L2 大类配置。
pub fn match_l2(profile: &Profile, lib: &ModeLibrary) -> Result<L2Allocation, L2Error> {
    // 久期不足一年:直接覆盖,不看回撤反应 —— 短期要用的钱不进权益
    if profile.horizon == Horizon::Within1y {
        let cfg = lib
            .l2(CASH_OVER_L2_ID)
            .ok_or_else(|| L2Error::MissingConfig(CASH_OVER_L2_ID.to_string()))?;
        let reason = format!(
            "资金久期在 {} 内,短期要用的钱不进入权益类与债券类",
            profile.horizon.label()
        );
        return Ok(L2Allocation::from_config(cfg, reason));
    }

    let base: usize = match profile.drawdown_response {
        DrawdownResponse::Liquidate | DrawdownResponse::Reduce => 0,
        DrawdownResponse::Hold => 1,
        DrawdownResponse::Add => 2,
    };
    // 久期 1-3 年降一档;下限为阶梯最低档
    let adjusted = if profile.horizon == Horizon::Y1to3 {
        base.saturating_sub(1)
    } else {
        base
    };

    let id = TIER_LADDER[adjusted];
    let cfg = lib
        .l2(id)
        .ok_or_else(|| L2Error::MissingConfig(id.to_string()))?;

    let mut reason = format!(
        "按你的回撤反应({})与资金久期({})匹配 {}",
        profile.drawdown_response.label(),
        profile.horizon.label(),
        cfg.name
    );
    if adjusted != base {
        reason.push_str(";资金久期 1-3 年,已在基础档上降一档");
    }

    Ok(L2Allocation::from_config(cfg, reason))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::{Goal, IncomeStability};

    fn profile_with(horizon: Horizon, drawdown: DrawdownResponse) -> Profile {
        Profile {
            horizon,
            drawdown_response: drawdown,
            income_stability: IncomeStability::Volatile,
            dependents: 1,
            inflow_cents: 1_200_000,
            expense_fixed_monthly_cents: 450_000,
            savings_cents: 2_400_000,
            goal: Goal::Wealth,
        }
    }

    fn lib() -> ModeLibrary {
        ModeLibrary::load_embedded().unwrap()
    }

    #[test]
    fn 回撤反应决定基础档() {
        let l = lib();
        for (resp, expected) in [
            (DrawdownResponse::Liquidate, "permanent_portfolio"),
            (DrawdownResponse::Reduce, "permanent_portfolio"),
            (DrawdownResponse::Hold, "sixty_forty"),
            (DrawdownResponse::Add, "core_satellite"),
        ] {
            let p = profile_with(Horizon::Y5to10, resp);
            let got = match_l2(&p, &l).unwrap();
            assert_eq!(got.id, expected, "回撤反应 {resp:?} 的档位不对");
        }
    }

    #[test]
    fn 久期一到三年降一档且下限为永久组合() {
        let l = lib();
        // 加仓(最高档)→ 降为 60/40
        let p = profile_with(Horizon::Y1to3, DrawdownResponse::Add);
        assert_eq!(match_l2(&p, &l).unwrap().id, "sixty_forty");
        // 不动 → 降为永久组合
        let p = profile_with(Horizon::Y1to3, DrawdownResponse::Hold);
        assert_eq!(match_l2(&p, &l).unwrap().id, "permanent_portfolio");
        // 最低档再降仍停在最低档
        let p = profile_with(Horizon::Y1to3, DrawdownResponse::Liquidate);
        assert_eq!(match_l2(&p, &l).unwrap().id, "permanent_portfolio");
    }

    #[test]
    fn 久期不足一年覆盖为现金类() {
        let l = lib();
        let p = profile_with(Horizon::Within1y, DrawdownResponse::Add);
        let got = match_l2(&p, &l).unwrap();
        assert!(got.is_cash_override());
        assert_eq!(got.classes.len(), 1);
        assert_eq!(got.classes[0].name, "现金类");
        assert_eq!(got.classes[0].basis_points, 10_000);
        assert!(got.reason.contains("不进入权益类"), "理由应说明原因");
    }

    #[test]
    fn 理由包含实际选择而非模板() {
        let l = lib();
        let p = profile_with(Horizon::Y5to10, DrawdownResponse::Hold);
        let got = match_l2(&p, &l).unwrap();
        assert!(got.reason.contains("不动"), "理由应含回撤反应: {}", got.reason);
        assert!(got.reason.contains("5-10 年"), "理由应含久期: {}", got.reason);
    }

    #[test]
    fn 降档时理由标注降档() {
        let l = lib();
        let p = profile_with(Horizon::Y1to3, DrawdownResponse::Add);
        let got = match_l2(&p, &l).unwrap();
        assert!(got.reason.contains("降一档"), "实际: {}", got.reason);
    }

    #[test]
    fn 缺配置时报错而非静默降级() {
        let empty = ModeLibrary::from_sources(&[], &[]).unwrap();
        let p = profile_with(Horizon::Y5to10, DrawdownResponse::Hold);
        let err = match_l2(&p, &empty).unwrap_err();
        assert!(err.to_string().contains("缺少 L2 配置"), "实际: {err}");
    }

    #[test]
    fn 引用的_l2_清单覆盖三档加现金覆盖() {
        let ids: Vec<&str> = required_l2_ids().collect();
        assert_eq!(
            ids,
            vec![
                "permanent_portfolio",
                "sixty_forty",
                "core_satellite",
                "cash_only"
            ]
        );
        // 清单里的每一档都必须在内嵌库中存在(load_embedded 已交叉校验,此处双保险)
        let l = lib();
        for id in ids {
            assert!(l.l2(id).is_some(), "内嵌库缺 {id}");
        }
    }
}
