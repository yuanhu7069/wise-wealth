//! L1 分账求解器(RULE-007 ~ RULE-014)—— 纯函数,零 IO(ADR-B-004)。
//!
//! 求解顺序是固定的:固定支出桶 → 比例桶(基数 = 税后月收入)→ 规则桶 → 余量桶。
//! 资源不足时按 safety_first 让位:投资桶归零 → 缩减规则桶 → 仍不足则给出缺口提示,
//! 但**任何桶不得出现负数**。
//!
//! 本模块不认识数据库、HTTP 或时间:输入一份档案与一个模式,输出一张可照做的表。
//! 正因如此,金例 A/B/C 可以直接作为 `cargo test` 的断言,而不需要起服务或连库。
//!
//! 各条业务规则的落点(基线 §4.4 要求规则有唯一实现位置并标注编号):
//! RULE-007 求解顺序 · RULE-008 单一余量桶 · RULE-009 应急月数 · RULE-010 必要月支出
//! RULE-011 应急目标与缺口 · RULE-012 规则桶月转入 · RULE-013 存款语义 · RULE-014 safety_first

use crate::domain::l2::{self, L2Allocation, L2Error};
use crate::domain::mode::{EmergencyFundRule, ModeConfig, ModeLibrary, ShareType};
use crate::domain::profile::{IncomeStability, Profile};
use serde::{Deserialize, Serialize};

/// 百元 = 10_000 分,规则桶月转入的取整单位
const HUNDRED_YUAN_CENTS: i64 = 10_000;
/// 万分比分母
const BP_DENOM: i64 = 10_000;

/// 一个桶的求解结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BucketAmount {
    /// 桶 id(与模式配置一致)
    pub id: String,
    /// 展示名
    pub name: String,
    /// 用途一句话
    pub purpose: String,
    /// 每月转入(分)
    pub amount_monthly_cents: i64,
    /// 目标金额(分),仅规则桶且未达标时有值
    pub target_cents: Option<i64>,
}

/// 应急金状态(RULE-009 ~ RULE-013)。两种模式都有 —— 无规则桶的模式同样需要达标标注。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EmergencyStatus {
    /// 应急目标月数
    pub months: i32,
    /// 必要月支出(分)
    pub necessary_monthly_cents: i64,
    /// 应急目标(分)
    pub target_cents: i64,
    /// 缺口(分);为 0 即达标
    pub gap_cents: i64,
    /// 每月用于补应急的金额(分)
    pub monthly_toward_emergency_cents: i64,
    /// 按当前节奏还差几个月达标;已达标为 None
    pub months_to_fill: Option<i32>,
    /// 当前覆盖月数 × 10(如 30 表示「约 3.0 个月」),整数运算避免浮点
    pub coverage_tenths: i32,
    /// 超出应急目标的部分(分),仅作展示,不参与分配
    pub surplus_cents: i64,
    /// 是否已达标。**达标不是 Notice** —— 它是状态而非告警,与 `surplus_cents` 一起
    /// 构成方案页的达标文案;若再往 notices 里塞一条,同一事实就有了两个来源。
    pub is_met: bool,
}

/// 方案提示。UI 按此渲染警示条,引擎只给事实不给文案。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Notice {
    /// 收入不足以覆盖固定支出与比例桶(safety_first 链走到尽头)
    InsufficientIncome,
    /// 固定支出已超过必要桶额度(prd-v1 §7.3 业务边界:无固定支出桶的模式才可能发生)
    FixedExceedsNecessary,
    /// 久期不足一年,本方案不配置权益
    ShortHorizonCashOnly,
}

/// 推理链条目的输出单位。用枚举而非字符串:单位随行变化的裸数字是缺陷温床(基线 §4.9)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceUnit {
    /// 金额,单位分
    Cents,
    /// 月数
    Months,
}

/// 推理链条目(产品 PRD §5.1.2 的 DecisionTrace 形状,裁剪到 B 期所需)。
///
/// B 期不在界面上展开推理链(那是 Plus 的能力),这里产出它是为了让方案快照能冻结
/// 「这个数字怎么来的」—— 将来开放推理链时不必回填历史。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trace {
    /// 规则标识
    pub rule_id: String,
    /// 输出值
    pub output: i64,
    /// 输出单位
    pub unit: TraceUnit,
    /// 一句话说明这条规则的输入与依据
    pub rationale: String,
}

/// 求解结果:方案页与落库所需的全部内容。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanResult {
    /// 使用的 L1 模式
    pub mode_id: String,
    /// 各桶金额
    pub buckets: Vec<BucketAmount>,
    /// 投资桶的大类配置
    pub l2: L2Allocation,
    /// L2 作用在哪个桶上(可能没有 —— 模式未标注投资桶时)
    pub l2_bucket_id: Option<String>,
    /// 应急金状态
    pub emergency: EmergencyStatus,
    /// 提示
    pub notices: Vec<Notice>,
    /// 推理链
    pub traces: Vec<Trace>,
}

/// 求解失败。
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// 档案本身不合法(API 层已校验,此处是最后一道防线)
    #[error("档案非法: {0}")]
    InvalidProfile(String),
    /// L2 匹配失败
    #[error(transparent)]
    L2(#[from] L2Error),
}

/// 求解一份方案。
pub fn solve(
    profile: &Profile,
    mode: &ModeConfig,
    lib: &ModeLibrary,
) -> Result<PlanResult, EngineError> {
    if profile.inflow_cents <= 0 {
        return Err(EngineError::InvalidProfile("月收入必须为正".into()));
    }
    if profile.expense_fixed_monthly_cents < 0 || profile.savings_cents < 0 {
        return Err(EngineError::InvalidProfile(
            "固定支出与现有存款不可为负".into(),
        ));
    }

    // ── 1) RULE-007 求解顺序:固定支出桶 → 比例桶;顺带记下规则桶/余量桶/投资桶的位置
    let mut amounts = vec![0i64; mode.buckets.len()];
    let mut mandatory_cents = 0i64;
    let mut has_fixed_bucket = false;
    let mut rule_idx: Option<usize> = None;
    let mut remainder_idx: Option<usize> = None;
    let mut investable_idx: Option<usize> = None;
    for (i, b) in mode.buckets.iter().enumerate() {
        match b.share {
            ShareType::FixedExpenses => {
                amounts[i] = profile.expense_fixed_monthly_cents;
                mandatory_cents += amounts[i];
                has_fixed_bucket = true;
            }
            ShareType::Pct { basis_points } => {
                amounts[i] = mul_bp(profile.inflow_cents, basis_points);
                mandatory_cents += amounts[i];
            }
            // RULE-008:一个模式至多一个余量桶,已在装载期校验
            ShareType::Rule { .. } => rule_idx = Some(i),
            ShareType::Remainder => remainder_idx = Some(i),
        }
        if b.is_investable {
            investable_idx = Some(i);
        }
    }

    // ── 2) RULE-009/010/011:应急月数 → 必要月支出 → 应急目标 → 缺口
    let necessary_cents: i64 = mode
        .buckets
        .iter()
        .enumerate()
        .filter(|(_, b)| b.is_necessary)
        .map(|(i, _)| amounts[i])
        .sum();
    let base = base_months(profile.income_stability, &mode.emergency_fund);
    let months = emergency_months(profile, &mode.emergency_fund);
    let target_cents = necessary_cents * i64::from(months);
    let gap_cents = (target_cents - profile.savings_cents).max(0);
    let is_met = gap_cents == 0;
    // RULE-013:存款超出目标的部分仅作展示,不参与分配
    let surplus_cents = (profile.savings_cents - target_cents).max(0);

    // RULE-012:缺口按规划月数摊平,取整到百元
    let nominal_transfer = if gap_cents > 0 {
        pacing_transfer(gap_cents, mode.emergency_fund.pacing_months)
    } else {
        0
    };

    // ── 3) RULE-014 safety_first:投资桶先归零 → 缩减规则桶 → 仍不足则记缺口
    let mut transfer = if rule_idx.is_some() { nominal_transfer } else { 0 };
    let mut insufficient = false;
    if let Some(ri) = remainder_idx {
        let raw = profile.inflow_cents - mandatory_cents - transfer;
        if raw >= 0 {
            amounts[ri] = raw;
        } else {
            transfer = transfer.min((profile.inflow_cents - mandatory_cents).max(0));
            let after = profile.inflow_cents - mandatory_cents - transfer;
            if after < 0 {
                insufficient = true;
            }
            amounts[ri] = after.max(0);
        }
    }
    if let Some(xi) = rule_idx {
        amounts[xi] = transfer;
    }

    // ── 4) 提示
    let mut notices = Vec::new();
    if insufficient {
        notices.push(Notice::InsufficientIncome);
    }
    if !has_fixed_bucket && profile.expense_fixed_monthly_cents > necessary_cents {
        notices.push(Notice::FixedExceedsNecessary);
    }

    // ── 5) L2 匹配
    let l2_alloc = l2::match_l2(profile, lib)?;
    if l2_alloc.is_cash_override() {
        notices.push(Notice::ShortHorizonCashOnly);
    }
    let l2_bucket_id = investable_idx.map(|i| mode.buckets[i].id.clone());
    let investable_cents = investable_idx.map(|i| amounts[i]).unwrap_or(0);

    // ── 6) 应急金状态:每月补应急的钱,规则桶优先;无规则桶时储蓄桶在未达标期补位
    let monthly_toward = if is_met {
        0
    } else if transfer > 0 {
        transfer
    } else {
        investable_cents
    };
    let months_to_fill = if !is_met && monthly_toward > 0 {
        Some(div_round_half_away(gap_cents, monthly_toward) as i32)
    } else {
        None
    };
    let coverage_tenths = if necessary_cents > 0 {
        div_round_half_away(profile.savings_cents * 10, necessary_cents) as i32
    } else {
        0
    };

    // ── 7) 组装
    let buckets = mode
        .buckets
        .iter()
        .enumerate()
        .map(|(i, b)| BucketAmount {
            id: b.id.clone(),
            name: b.name.clone(),
            purpose: b.purpose.clone(),
            amount_monthly_cents: amounts[i],
            target_cents: if matches!(b.share, ShareType::Rule { .. }) && !is_met {
                Some(target_cents)
            } else {
                None
            },
        })
        .collect();

    let bump_note = if months > base {
        format!(
            ",需赡养人数 {} > 0 上浮一档至 {} 个月",
            profile.dependents, months
        )
    } else {
        String::new()
    };
    let traces = vec![
        Trace {
            rule_id: "emergency_fund_months".into(),
            output: i64::from(months),
            unit: TraceUnit::Months,
            rationale: format!(
                "收入稳定性={} → {} 个月{}",
                profile.income_stability.label(),
                base,
                bump_note
            ),
        },
        Trace {
            rule_id: "emergency_fund_target".into(),
            output: target_cents,
            unit: TraceUnit::Cents,
            rationale: format!("必要月支出 {necessary_cents} 分 × {months} 个月"),
        },
        Trace {
            rule_id: "emergency_fund_pacing".into(),
            output: transfer,
            unit: TraceUnit::Cents,
            rationale: format!(
                "缺口 {gap_cents} 分 ÷ {} 个月,取整到百元",
                mode.emergency_fund.pacing_months
            ),
        },
    ];

    Ok(PlanResult {
        mode_id: mode.id.clone(),
        buckets,
        l2: l2_alloc,
        l2_bucket_id,
        emergency: EmergencyStatus {
            months,
            necessary_monthly_cents: necessary_cents,
            target_cents,
            gap_cents,
            monthly_toward_emergency_cents: monthly_toward,
            months_to_fill,
            coverage_tenths,
            surplus_cents,
            is_met,
        },
        notices,
        traces,
    })
}

/// 按收入稳定性取基础应急月数(未含赡养上浮)。
fn base_months(stability: IncomeStability, rule: &EmergencyFundRule) -> i32 {
    match stability {
        IncomeStability::Stable => rule.stable,
        IncomeStability::Normal => rule.normal,
        IncomeStability::Volatile => rule.volatile,
        IncomeStability::Freelance => rule.freelance,
    }
}

/// RULE-009 应急月数:基础档 + 赡养上浮。上浮是**跨档**(3→6→9→12)而非加一个月。
fn emergency_months(profile: &Profile, rule: &EmergencyFundRule) -> i32 {
    let base = base_months(profile.income_stability, rule).min(rule.max_months);
    if profile.dependents == 0 || rule.dependents_bump <= 0 {
        return base;
    }
    let mut tiers = vec![
        rule.stable,
        rule.normal,
        rule.volatile,
        rule.freelance,
        rule.max_months,
    ];
    tiers.sort_unstable();
    tiers.dedup();
    let idx = tiers
        .iter()
        .position(|&t| t >= base)
        .unwrap_or(tiers.len() - 1);
    let target = (idx + rule.dependents_bump as usize).min(tiers.len() - 1);
    tiers[target].min(rule.max_months)
}

/// RULE-012 规则桶月转入:缺口按规划月数摊平后**一次**取整到百元。
///
/// 「一次」是重点:先把商四舍五入、再取整到百元,会让中间结果被放大一个档 ——
/// 缺口 ¥1,199.88 会被算成 ¥100/月(正确答案是 ¥0/月:不足半个百元档)。
fn pacing_transfer(gap_cents: i64, pacing_months: i64) -> i64 {
    let unit = HUNDRED_YUAN_CENTS * pacing_months;
    div_round_half_away(gap_cents, unit) * HUNDRED_YUAN_CENTS
}

/// 整数除法,四舍五入(half away from zero)。全链路金额以此保证可复现。
fn div_round_half_away(num: i64, den: i64) -> i64 {
    debug_assert!(den > 0, "分母必须为正");
    if num >= 0 {
        (num + den / 2) / den
    } else {
        -((-num + den / 2) / den)
    }
}

/// 按万分比取额:金额 × basis_points / 10000,四舍五入。
fn mul_bp(amount_cents: i64, basis_points: i64) -> i64 {
    div_round_half_away(amount_cents * basis_points, BP_DENOM)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::mode::ModeLibrary;
    use crate::domain::profile::{DrawdownResponse, Goal, Horizon};

    fn lib() -> ModeLibrary {
        ModeLibrary::load_embedded().unwrap()
    }

    /// 金例 A 档案:月收入 ¥12,000 / 固定支出 ¥4,500 / 存款 ¥24,000 / 久期 5-10 年 /
    /// 回撤不动 / 收入波动大 / 赡养 1 人。金例 B、C 在此之上改个别字段。
    fn profile_a() -> Profile {
        Profile {
            horizon: Horizon::Y5to10,
            drawdown_response: DrawdownResponse::Hold,
            income_stability: IncomeStability::Volatile,
            dependents: 1,
            inflow_cents: 1_200_000,
            expense_fixed_monthly_cents: 450_000,
            savings_cents: 2_400_000,
            goal: Goal::Wealth,
        }
    }

    fn amount(r: &PlanResult, id: &str) -> i64 {
        r.buckets
            .iter()
            .find(|b| b.id == id)
            .unwrap_or_else(|| panic!("方案里没有桶 {id}"))
            .amount_monthly_cents
    }

    #[test]
    fn 金例a_四账户四桶金额逐项正确() {
        let p = profile_a();
        let m = lib();
        let r = solve(&p, m.mode("four_accounts").unwrap(), &m).unwrap();

        assert_eq!(amount(&r, "salary"), 450_000, "工资账户");
        assert_eq!(amount(&r, "spend"), 360_000, "消费账户");
        assert_eq!(amount(&r, "reserve"), 310_000, "备用账户");
        assert_eq!(amount(&r, "invest"), 80_000, "投资账户");

        // 应急金:波动大 9 + 赡养上浮一档 = 12 个月;必要 8,100;目标 97,200
        assert_eq!(r.emergency.months, 12);
        assert_eq!(r.emergency.necessary_monthly_cents, 810_000);
        assert_eq!(r.emergency.target_cents, 9_720_000);
        assert_eq!(r.emergency.gap_cents, 7_320_000);
        assert_eq!(r.emergency.monthly_toward_emergency_cents, 310_000);
        assert_eq!(r.emergency.coverage_tenths, 30, "当前约 3.0 个月");
        assert_eq!(r.emergency.months_to_fill, Some(24), "还差约 24 个月");
        assert!(!r.emergency.is_met);
        assert!(r.notices.is_empty(), "金例 A 不应有任何提示");
    }

    #[test]
    fn 金例a_的_l2_为_60_40() {
        let p = profile_a();
        let m = lib();
        let r = solve(&p, m.mode("four_accounts").unwrap(), &m).unwrap();
        assert_eq!(r.l2.id, "sixty_forty");
        assert_eq!(r.l2_bucket_id.as_deref(), Some("invest"));
        let equity = r.l2.classes.iter().find(|c| c.name == "权益类").unwrap();
        assert_eq!(equity.basis_points, 6000);
    }

    #[test]
    fn 金例b_存款达标时规则桶归零且份额进投资() {
        let mut p = profile_a();
        p.savings_cents = 15_000_000; // ¥150,000
        let m = lib();
        let r = solve(&p, m.mode("four_accounts").unwrap(), &m).unwrap();

        assert_eq!(amount(&r, "reserve"), 0, "达标后备用桶应停止转入");
        assert_eq!(amount(&r, "invest"), 390_000, "¥3,900");
        assert!(r.emergency.is_met);
        assert_eq!(r.emergency.gap_cents, 0);
        assert_eq!(r.emergency.surplus_cents, 5_280_000, "家底 ¥52,800");
        assert_eq!(r.emergency.coverage_tenths, 185, "约 18.5 个月");
        assert_eq!(r.emergency.months_to_fill, None);
        // 达标时备用桶不再有目标金额
        let reserve = r.buckets.iter().find(|b| b.id == "reserve").unwrap();
        assert_eq!(reserve.target_cents, None);
    }

    #[test]
    fn 金例c_收入不足时让位链走完且无负数() {
        let mut p = profile_a();
        p.inflow_cents = 800_000; // ¥8,000
        p.expense_fixed_monthly_cents = 700_000; // ¥7,000
        p.savings_cents = 500_000; // ¥5,000
        let m = lib();
        let r = solve(&p, m.mode("four_accounts").unwrap(), &m).unwrap();

        assert_eq!(amount(&r, "salary"), 700_000);
        assert_eq!(amount(&r, "spend"), 240_000, "30% × 8,000");
        assert_eq!(amount(&r, "reserve"), 0, "safety_first:规则桶让位归零");
        assert_eq!(amount(&r, "invest"), 0, "safety_first:投资桶先归零");
        assert!(
            r.notices.contains(&Notice::InsufficientIncome),
            "应给出收入不足提示"
        );
        assert!(
            r.buckets.iter().all(|b| b.amount_monthly_cents >= 0),
            "任何桶都不得出现负数"
        );
    }

    #[test]
    fn 五十三十二十_同档案三桶正确且储蓄桶兼作投资桶() {
        let p = profile_a();
        let m = lib();
        let r = solve(&p, m.mode("fifty_30_20").unwrap(), &m).unwrap();

        assert_eq!(amount(&r, "needs"), 600_000, "¥6,000");
        assert_eq!(amount(&r, "wants"), 360_000, "¥3,600");
        assert_eq!(amount(&r, "savings"), 240_000, "¥2,400");
        assert_eq!(r.l2_bucket_id.as_deref(), Some("savings"));

        // 必要月支出 = 必要桶;目标 = 12 × 6,000 = 72,000;当前 4.0 个月
        assert_eq!(r.emergency.necessary_monthly_cents, 600_000);
        assert_eq!(r.emergency.target_cents, 7_200_000);
        assert_eq!(r.emergency.gap_cents, 4_800_000);
        assert_eq!(r.emergency.coverage_tenths, 40);
        // 无规则桶:每月补应急的是储蓄桶,48,000 ÷ 2,400 = 20 个月
        assert_eq!(r.emergency.monthly_toward_emergency_cents, 240_000);
        assert_eq!(r.emergency.months_to_fill, Some(20));
    }

    #[test]
    fn 标准普尔象限_同档案四桶按4321静态切分() {
        let p = profile_a();
        let m = lib();
        let r = solve(&p, m.mode("snp_quadrant").unwrap(), &m).unwrap();

        // 纯 pct 恰好占满:10% / 20% / 30% / 40% × ¥12,000
        assert_eq!(amount(&r, "spending"), 120_000);
        assert_eq!(amount(&r, "protection"), 240_000);
        assert_eq!(amount(&r, "growth"), 360_000);
        assert_eq!(amount(&r, "preserve"), 480_000);
        assert_eq!(r.l2_bucket_id.as_deref(), Some("growth"), "投资桶 = 生钱的钱");
        assert_eq!(r.l2.id, "sixty_forty", "L2 全局匹配,与 L1 无关");

        // 必要 = 要花的钱 ¥1,200;目标 = 12 × 1,200 = 14,400 < 存款 24,000 → 达标
        assert_eq!(r.emergency.necessary_monthly_cents, 120_000);
        assert_eq!(r.emergency.target_cents, 1_440_000);
        assert!(r.emergency.is_met);
        assert_eq!(r.emergency.monthly_toward_emergency_cents, 0, "达标即停");
        assert_eq!(r.emergency.coverage_tenths, 200, "约 20.0 个月");

        // 无固定支出桶:固定支出 ¥4,500 > 必要桶 ¥1,200,与 50/30/20 同语义应提示
        assert!(r.notices.contains(&Notice::FixedExceedsNecessary));
    }

    #[test]
    fn 四笔钱_同档案四桶含应急规则与余量() {
        let p = profile_a();
        let m = lib();
        let r = solve(&p, m.mode("four_pots").unwrap(), &m).unwrap();

        // 活钱 = 固定支出;稳钱 = 20%;保障 = 应急节奏;长钱 = 余量
        assert_eq!(amount(&r, "liquid"), 450_000);
        assert_eq!(amount(&r, "stable"), 240_000, "20% × ¥12,000");
        assert_eq!(
            amount(&r, "safeguard"),
            130_000,
            "应急节奏:缺口 ¥30,000 ÷ 24 期 = ¥1,250/月,取整到百元 = ¥1,300"
        );
        assert_eq!(amount(&r, "long_term"), 380_000, "余量兜底");
        assert_eq!(r.l2_bucket_id.as_deref(), Some("long_term"));

        // 必要 = 活钱 ¥4,500;目标 = 12 × 4,500 = 54,000;缺口 30,000 ÷ 24 = 1,250
        assert_eq!(r.emergency.necessary_monthly_cents, 450_000);
        assert_eq!(r.emergency.target_cents, 5_400_000);
        assert_eq!(r.emergency.gap_cents, 3_000_000);
        assert!(!r.emergency.is_met);
        assert_eq!(r.emergency.monthly_toward_emergency_cents, 130_000);
        assert_eq!(r.emergency.months_to_fill, Some(23), "3,000,000 ÷ 130,000 = 23.08");
        assert!(r.notices.is_empty(), "有固定支出桶,不应有 FixedExceedsNecessary");
    }

    #[test]
    fn 固定支出超过必要桶额度时给出提示() {
        let mut p = profile_a();
        p.expense_fixed_monthly_cents = 700_000; // ¥7,000 > 必要桶 ¥6,000
        let m = lib();
        let r = solve(&p, m.mode("fifty_30_20").unwrap(), &m).unwrap();
        assert!(
            r.notices.contains(&Notice::FixedExceedsNecessary),
            "实际提示: {:?}",
            r.notices
        );
    }

    #[test]
    fn 四账户不会误报固定支出超额度() {
        // 四账户有专门的固定支出桶,该提示不该出现
        let p = profile_a();
        let m = lib();
        let r = solve(&p, m.mode("four_accounts").unwrap(), &m).unwrap();
        assert!(!r.notices.contains(&Notice::FixedExceedsNecessary));
    }

    #[test]
    fn 久期不足一年时不配权益并给出提示() {
        let mut p = profile_a();
        p.horizon = Horizon::Within1y;
        let m = lib();
        let r = solve(&p, m.mode("four_accounts").unwrap(), &m).unwrap();
        assert_eq!(r.l2.id, "cash_only");
        assert!(r.notices.contains(&Notice::ShortHorizonCashOnly));
        assert!(
            r.l2.classes.iter().all(|c| c.name != "权益类"),
            "不应出现权益类"
        );
    }

    #[test]
    fn 缺口取整到百元只取整一次() {
        // 缺口 ¥1,199.88:÷24 得 ¥49.995,不足半个百元档 → ¥0/月
        assert_eq!(pacing_transfer(119_988, 24), 0);
        // 恰好半个百元档(¥1,200 ÷ 24 = ¥50)→ 远离零取整到 ¥100
        assert_eq!(pacing_transfer(120_000, 24), 10_000);
        // 差一分钱到半数 → 仍取 0
        assert_eq!(pacing_transfer(119_999, 24), 0);
        // 金例 A:缺口 ¥73,200 ÷ 24 = ¥3,050(恰为半数)→ ¥3,100
        assert_eq!(pacing_transfer(7_320_000, 24), 310_000);
        // 无缺口不虚构金额
        assert_eq!(pacing_transfer(0, 24), 0);
    }

    #[test]
    fn 万分比取额无浮点误差() {
        assert_eq!(mul_bp(1_200_000, 3000), 360_000);
        assert_eq!(mul_bp(1_200_000, 5000), 600_000);
        assert_eq!(mul_bp(800_000, 3000), 240_000);
        // 无法整除时四舍五入,而非截断
        assert_eq!(mul_bp(100, 3_333), 33);
    }

    #[test]
    fn 覆盖月数按半远离零取整() {
        // 24,000 ÷ 8,100 = 2.96 → 3.0 个月
        assert_eq!(div_round_half_away(2_400_000 * 10, 810_000), 30);
        // 恰好 .5 时不截断
        assert_eq!(div_round_half_away(15, 10), 2);
        assert_eq!(div_round_half_away(14, 10), 1);
    }

    #[test]
    fn 赡养上浮按档位跨档而非加一个月() {
        let rule = EmergencyFundRule {
            stable: 3,
            normal: 6,
            volatile: 9,
            freelance: 9,
            dependents_bump: 1,
            max_months: 12,
            pacing_months: 24,
        };
        let mut p = profile_a();
        p.dependents = 0;
        p.income_stability = IncomeStability::Stable;
        assert_eq!(emergency_months(&p, &rule), 3);
        p.dependents = 1;
        assert_eq!(emergency_months(&p, &rule), 6, "很稳 + 赡养 → 6,而非 4");
        p.income_stability = IncomeStability::Normal;
        assert_eq!(emergency_months(&p, &rule), 9);
        p.income_stability = IncomeStability::Volatile;
        assert_eq!(emergency_months(&p, &rule), 12);
        // 上限封顶:再上浮也不会超过 12
        p.dependents = 5;
        assert_eq!(emergency_months(&p, &rule), 12);
    }

    #[test]
    fn 推理链记录了月数与缺口依据() {
        let p = profile_a();
        let m = lib();
        let r = solve(&p, m.mode("four_accounts").unwrap(), &m).unwrap();
        assert_eq!(r.traces.len(), 3);
        let months = r
            .traces
            .iter()
            .find(|t| t.rule_id == "emergency_fund_months")
            .unwrap();
        assert_eq!(months.output, 12);
        assert_eq!(months.unit, TraceUnit::Months);
        assert!(
            months.rationale.contains("波动大"),
            "实际: {}",
            months.rationale
        );
        assert!(months.rationale.contains("上浮"), "实际: {}", months.rationale);
        let pacing = r
            .traces
            .iter()
            .find(|t| t.rule_id == "emergency_fund_pacing")
            .unwrap();
        assert_eq!(pacing.output, 310_000);
        assert_eq!(pacing.unit, TraceUnit::Cents);
    }

    #[test]
    fn 收入为零时拒绝求解而非产出垃圾() {
        let mut p = profile_a();
        p.inflow_cents = 0;
        let m = lib();
        let err = solve(&p, m.mode("four_accounts").unwrap(), &m).unwrap_err();
        assert!(err.to_string().contains("月收入必须为正"), "实际: {err}");
    }

    #[test]
    fn 求解不依赖模式库以外的状态() {
        // 同一输入重复求解必须逐字节一致(纯函数的直接推论,也是方案可复现的前提)
        let p = profile_a();
        let m = lib();
        let mode = m.mode("four_accounts").unwrap();
        let a = solve(&p, mode, &m).unwrap();
        let b = solve(&p, mode, &m).unwrap();
        assert_eq!(a, b);
    }
}
