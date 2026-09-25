//! L1 模式推荐 service(RULE-006)。
//!
//! 纯函数:输入一份档案与模式库,输出「主推哪个模式 + 一句人话理由」。
//! 与引擎一样零 IO —— 推荐规则是产品判断,不该藏在 handler 里。

use crate::domain::profile::Profile;
use crate::domain::{ModeLibrary, ModeConfig};

/// 主推规则的分母:固定支出的倍数
const MONTHS_OF_FIXED_EXPENSE: i64 = 3;

/// 推荐结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recommendation {
    /// 主推模式的 id
    pub recommended_id: String,
    /// 一句人话理由(直接展示在步 6 的主推卡上)
    pub reason: String,
}

/// RULE-006:现有存款不足 3 个月固定支出 → 主推四账户;否则主推 50/30/20。
///
/// 判断依据是「有没有余力」而不是风险偏好:储蓄薄的人先要纪律与应急金,
/// 储蓄厚的人更需要一个不啰嗦的比例框架。两张卡始终同时展示,用户自己选。
pub fn recommend(profile: &Profile, library: &ModeLibrary) -> Option<Recommendation> {
    let thin = profile.savings_cents < profile.expense_fixed_monthly_cents * MONTHS_OF_FIXED_EXPENSE;

    // 主推哪个模式由规则定,但模式本身必须真的在库里 —— 否则宁可不出推荐,
    // 也不要让界面指向一个算不出方案的模式。
    let (id, reason) = if thin {
        (
            "four_accounts",
            "你的存款不足 3 个月固定支出,先建立分账纪律与应急金更合适",
        )
    } else {
        ("fifty_30_20", "结构最简洁,适合已有储蓄习惯的你")
    };

    library.mode(id)?;
    Some(Recommendation {
        recommended_id: id.to_string(),
        reason: reason.to_string(),
    })
}

/// 供界面渲染的模式卡:只暴露配置里真实存在的字段,页面不硬编码任何模式信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeCard {
    /// 模式 id
    pub id: String,
    /// 展示名
    pub name: String,
    /// 一句话理念
    pub tagline: String,
    /// 可信度(已考证 / 存疑 / 谨慎)
    pub credibility: crate::domain::Credibility,
    /// 适合人群标签
    pub fit_for: Vec<String>,
}

/// 列出全部 L1 模式(顺序与配置目录扫描顺序一致)。
pub fn list_modes(library: &ModeLibrary) -> Vec<ModeCard> {
    library.modes().iter().map(card_of).collect()
}

fn card_of(mode: &ModeConfig) -> ModeCard {
    ModeCard {
        id: mode.id.clone(),
        name: mode.name.clone(),
        tagline: mode.tagline.clone(),
        credibility: mode.credibility,
        fit_for: mode.fit_for.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::{DrawdownResponse, Goal, Horizon, IncomeStability};

    fn lib() -> ModeLibrary {
        ModeLibrary::load_embedded().unwrap()
    }

    fn profile(income: i64, fixed: i64, savings: i64) -> Profile {
        Profile {
            horizon: Horizon::Y5to10,
            drawdown_response: DrawdownResponse::Hold,
            income_stability: IncomeStability::Volatile,
            dependents: 1,
            inflow_cents: income,
            expense_fixed_monthly_cents: fixed,
            savings_cents: savings,
            goal: Goal::Wealth,
        }
    }

    #[test]
    fn 存款薄时主推四账户() {
        // 金例 A:存款 24,000 < 3 × 4,500 = 13,500? 不成立 → 走另一支
        // 用真正的薄档案:存款 10,000 < 13,500
        let r = recommend(&profile(1_200_000, 450_000, 1_000_000), &lib()).unwrap();
        assert_eq!(r.recommended_id, "four_accounts");
        assert!(r.reason.contains("应急金"), "理由应说明原因: {}", r.reason);
    }

    #[test]
    fn 存款厚时主推五十三十二十() {
        // 金例 A 的存款 24,000 > 13,500
        let r = recommend(&profile(1_200_000, 450_000, 2_400_000), &lib()).unwrap();
        assert_eq!(r.recommended_id, "fifty_30_20");
        assert!(r.reason.contains("简洁"), "实际: {}", r.reason);
    }

    #[test]
    fn 恰好三倍固定支出时不算薄() {
        // 边界:< 3 倍才算薄,等于 3 倍不算
        let r = recommend(&profile(1_200_000, 450_000, 1_350_000), &lib()).unwrap();
        assert_eq!(r.recommended_id, "fifty_30_20");
    }

    #[test]
    fn 零存款主推四账户() {
        let r = recommend(&profile(1_200_000, 450_000, 0), &lib()).unwrap();
        assert_eq!(r.recommended_id, "four_accounts");
    }

    #[test]
    fn 模式卡来自配置而非硬编码() {
        let cards = list_modes(&lib());
        assert!(cards.len() >= 2, "至少两个 L1 模式");
        let four = cards.iter().find(|c| c.id == "four_accounts").unwrap();
        assert_eq!(four.name, "四账户理财法");
        assert_eq!(four.tagline, "工资 · 消费 · 备用 · 投资");
        assert_eq!(four.credibility, crate::domain::Credibility::Verified);
        assert!(!four.fit_for.is_empty(), "适用标签应来自配置");
    }

    #[test]
    fn 库里没有主推模式时宁可不推荐() {
        // 注入一个空库:主推规则指向的模式不存在 → 返回 None 而不是给一个算不出的 id
        let empty = ModeLibrary::from_sources(&[], &[]).unwrap();
        assert!(recommend(&profile(1_200_000, 450_000, 0), &empty).is_none());
    }

    #[test]
    fn 主推永不指向存疑模式_rule033() {
        // RULE-033:disputed 模式(标准普尔象限)不进自动匹配 —— 薄厚两支档案都验证
        let lib = lib();
        for (desc, p) in [
            ("存款薄", profile(1_200_000, 450_000, 0)),
            ("存款厚", profile(1_200_000, 450_000, 99_000_000)),
        ] {
            let r = recommend(&p, &lib).unwrap();
            assert_ne!(
                r.recommended_id, "snp_quadrant",
                "{desc}:主推不得指向存疑模式"
            );
        }
        // 模式卡列表照常含存疑卡(浏览可达,仅不主推)
        let cards = list_modes(&lib);
        let snp = cards.iter().find(|c| c.id == "snp_quadrant").unwrap();
        assert_eq!(snp.credibility, crate::domain::Credibility::Disputed);
    }
}
