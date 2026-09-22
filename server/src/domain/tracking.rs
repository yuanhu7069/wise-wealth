//! 追踪域逻辑(RULE-024 ~ RULE-028)—— 纯函数,零 IO(ADR-E-002)。
//!
//! 与 [`crate::domain::engine`] 同一立场:输入快照序列与方案口径,输出可判定的结论,
//! 不认识数据库、HTTP 或时钟。正因如此,RULE-025 的每一条边界(基准回溯、同名桶、
//! 前值 ≤ 0)都能用金例钉死在 `cargo test` 里,前端与 P01/P05 只渲染、零计算。
//!
//! 本模块只回答「这组快照说明了什么」;「快照该不该存在、桶集合对不对」是
//! service 层的事(ADR-E-001 的校验兜底落在那里)。

use std::collections::BTreeMap;

/// 偏离判定(RULE-025):前值必须为正才参与百分比比对 —— 从 0 起步与透支回正
/// 不按百分比误报(「上个月 0 元这个月 2 千」报 +∞% 是语义事故)。
const BP_DENOM: i64 = 10_000;

/// 一条快照在域层的最小形态(由 service 从 repo 行投影而来)。
///
/// `balances` 用 `BTreeMap`:键即桶 id,遍历顺序稳定(按 id 字典序),
/// 展示顺序由视图层按方案桶序重排 —— 域层不管展示。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotPoint {
    /// 自然月(YYYY-MM,当月 1 日落库前的字符串形态)
    pub month: String,
    /// 各桶余额(分);键集合必须等于提交时 active 方案的桶集(service 层校验)
    pub balances: BTreeMap<String, i64>,
    /// 「本月特殊」标记(RULE-024)
    pub special_month: bool,
}

/// 偏离方向。涨跌的语义色由前端决定(红跌绿涨),域层只给事实。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// 现值 ≥ 前值
    Up,
    /// 现值 < 前值
    Down,
}

/// 一个桶的偏离结论(RULE-025)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deviation {
    /// 桶 id
    pub bucket_id: String,
    /// 基准值(分)
    pub prev_cents: i64,
    /// 本月值(分)
    pub current_cents: i64,
    /// |现值 − 前值| ÷ 前值,万分比(向下取整;精度损失 ≤ 0.01%,阈值判定不受影响)
    pub deviation_bp: i64,
    /// 方向
    pub direction: Direction,
}

/// 相邻快照偏离判定(RULE-024 / RULE-025 / RULE-028)。
///
/// - `current` 是最新一条快照;`history` 是**它之前**的快照、按时间升序、不含它自己;
/// - 返回 `None`:本月特殊(RULE-024:不判偏离),或历史中没有合格的
///   非特殊基准(首条录入、历史全部特殊 —— RULE-024/025);
/// - 返回 `Some(vec![])`:有基准但没有任何桶超阈 —— 「看过,没问题」与
///   「没得比」是两回事,前端据此决定偏离条是隐藏还是显示「一切如常」;
/// - 只比对两侧**同名桶**(RULE-028:桶结构变化后自动失去可比性);
/// - 基准只取 `history` 中**最近一条**非特殊快照,更早的中间特殊月被跳过。
pub fn deviations(
    current: &SnapshotPoint,
    history: &[SnapshotPoint],
    threshold_bp: i64,
) -> Option<Vec<Deviation>> {
    if current.special_month {
        return None;
    }
    let baseline = history.iter().rev().find(|s| !s.special_month)?;
    let mut out = Vec::new();
    for (bucket_id, current_cents) in &current.balances {
        let Some(prev_cents) = baseline.balances.get(bucket_id) else {
            continue;
        };
        if *prev_cents <= 0 {
            continue;
        }
        // 值域经 RULE-022 校验(|v| < 10^12 分),差值与 ×10^4 都在 i64 内,无溢出
        let deviation_bp = (current_cents - *prev_cents).abs() * BP_DENOM / *prev_cents;
        if deviation_bp >= threshold_bp {
            out.push(Deviation {
                bucket_id: bucket_id.clone(),
                prev_cents: *prev_cents,
                current_cents: *current_cents,
                deviation_bp,
                direction: if *current_cents >= *prev_cents {
                    Direction::Up
                } else {
                    Direction::Down
                },
            });
        }
    }
    Some(out)
}

/// 距离感进度里的应急金结论(RULE-026)。口径完全复用引擎:`target_cents` 与
/// `necessary_monthly_cents` 取自方案快照(`emergency_json`),本期零新口径(ADR-E-002)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmergencyGap {
    /// 缺口月数 ×10(如 32 = 「还差 3.2 个月」);达标为 0
    pub gap_months_tenths: i64,
    /// 余额 ≥ 应急目标(CONTEXT「达标」:规则桶月转入为 0 的既定状态)
    pub met: bool,
    /// 距目标的缺口(分);已达标为 0
    pub gap_cents: i64,
}

/// 应急金缺口月数(RULE-026):(应急目标 − 最新余额)÷ 必要月支出,保留 1 位小数。
///
/// 舍入与引擎同族(半量上取整,即「(x + 半分母) ÷ 分母」的整数写法,见 ADR-B-004):
/// 展示口径不引入第二种舍入习惯。
/// 防御分支:目标 ≤ 0 或必要月支出 ≤ 0 时按达标处理 —— 正常方案不可能走到这里
/// (目标 = 必要月支出 × 月数,月数在装载期被校验为正),这只兜住手工构造的坏输入。
pub fn emergency_gap(
    target_cents: i64,
    balance_cents: i64,
    necessary_monthly_cents: i64,
) -> EmergencyGap {
    let gap_cents = (target_cents - balance_cents).max(0);
    if gap_cents == 0 || necessary_monthly_cents <= 0 {
        return EmergencyGap {
            gap_months_tenths: 0,
            met: gap_cents == 0,
            gap_cents,
        };
    }
    let tenths = (gap_cents * 10 + necessary_monthly_cents / 2) / necessary_monthly_cents;
    EmergencyGap {
        gap_months_tenths: tenths,
        met: false,
        gap_cents,
    }
}

/// 已坚持月数(RULE-027):有快照记录的自然月**累计数**。
///
/// 跳过月不惩罚 —— 数的是「坚持过的月份总量」,不是连续性;入参即去重后的月份集合大小。
pub fn persisted_months(snapshotted_months: &[&str]) -> usize {
    snapshotted_months.iter().collect::<std::collections::BTreeSet<_>>().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 便捷构造:按 (桶 id, 分) 列表造一条快照。
    fn snap(month: &str, special: bool, balances: &[(&str, i64)]) -> SnapshotPoint {
        SnapshotPoint {
            month: month.to_string(),
            balances: balances
                .iter()
                .map(|(id, cents)| (id.to_string(), *cents))
                .collect(),
            special_month: special,
        }
    }

    const THRESHOLD: i64 = 2_000; // 20%(缺省档,ADR-E-003)

    // ── 金例:超阈命中(prd-e §7.1:消费 3,150 → 4,200,+33.3%)──

    #[test]
    fn 偏离超阈的桶被逐个报出且方向正确() {
        let history = vec![snap(
            "2026-08",
            false,
            &[("salary", 1_240_000), ("spend", 315_000), ("reserve", 3_000_000)],
        )];
        let current = snap(
            "2026-09",
            false,
            &[("salary", 1_240_000), ("spend", 420_000), ("reserve", 3_200_000)],
        );

        let out = deviations(&current, &history, THRESHOLD).expect("有合格基准");
        // 只有消费桶超阈:|4,200−3,150|/3,150 = 33.33% → 3333 bp ≥ 2000
        assert_eq!(out.len(), 1, "其余桶在阈内: {out:?}");
        let d = &out[0];
        assert_eq!(d.bucket_id, "spend");
        assert_eq!(d.prev_cents, 315_000);
        assert_eq!(d.current_cents, 420_000);
        assert_eq!(d.deviation_bp, 3_333);
        assert_eq!(d.direction, Direction::Up);
    }

    #[test]
    fn 全部桶在阈内时返回空列表而非_none() {
        let history = vec![snap(
            "2026-08",
            false,
            &[("spend", 3_000_000), ("reserve", 30_000_000)],
        )];
        let current = snap(
            "2026-09",
            false,
            &[("spend", 3_200_000), ("reserve", 32_000_000)],
        );
        // +6.67% → 666 bp,未超 20%:结论是「有基准,一切如常」(Some(vec![])),
        // 不是「没得比」(None)—— 前端靠这个区别决定偏离条是「不占位」还是「显示如常」。
        assert_eq!(deviations(&current, &history, THRESHOLD), Some(vec![]));
    }

    // ── 金例:基准回溯(RULE-024,拷问 Q9:特殊月不作为基准)──

    #[test]
    fn 基准跳过特殊月落在更早的非特殊快照上() {
        let history = vec![
            snap("2026-07", false, &[("spend", 300_000)]),
            snap("2026-08", true, &[("spend", 900_000)]), // 特殊月:数字再离谱也不作基准
        ];
        let current = snap("2026-09", false, &[("spend", 360_000)]);

        // 基准必须是 7 月(300,000),而非紧邻的特殊月 8 月(900,000):
        // 若错拿了 8 月,prev 值会变成 900,000 —— 用 prev 断言直接钉死选基行为。
        let out = deviations(&current, &history, THRESHOLD).expect("7 月为合格基准");
        assert_eq!(out.len(), 1, "+20% 恰好达阈,见「恰好达阈即偏离」");
        assert_eq!(out[0].prev_cents, 300_000, "基准必须是 7 月,而非特殊月 8 月");
    }

    #[test]
    fn 恰好达阈即偏离() {
        // RULE-025 是「≥ 阈值即偏离」:+20% 恰好等于 2000 bp,必须命中。
        // (上一条用例与它共同锁定边界两侧;独立成条让失败信息直说语义。)
        let history = vec![snap("2026-07", false, &[("spend", 300_000)])];
        let current = snap("2026-09", false, &[("spend", 360_000)]);
        let out = deviations(&current, &history, THRESHOLD).expect("有基准");
        assert_eq!(out[0].deviation_bp, 2_000, "恰好 20% 计入偏离");
        assert_eq!(out[0].direction, Direction::Up);

        // 阈内一侧:19.99% 不得命中
        let current = snap("2026-09", false, &[("spend", 359_990)]);
        assert_eq!(deviations(&current, &history, THRESHOLD), Some(vec![]));
    }

    #[test]
    fn 本月特殊则不判偏离() {
        let history = vec![snap("2026-08", false, &[("spend", 300_000)])];
        let current = snap("2026-09", true, &[("spend", 900_000)]);
        assert_eq!(deviations(&current, &history, THRESHOLD), None);
    }

    // ── 金例:无合格基准(首条 / 历史全部特殊)──

    #[test]
    fn 无历史时不判偏离() {
        let current = snap("2026-09", false, &[("spend", 300_000)]);
        assert_eq!(deviations(&current, &[], THRESHOLD), None);
    }

    #[test]
    fn 历史全部为特殊时视同无基准() {
        let history = vec![snap("2026-08", true, &[("spend", 300_000)])];
        let current = snap("2026-09", false, &[("spend", 900_000)]);
        assert_eq!(deviations(&current, &history, THRESHOLD), None);
    }

    // ── 金例:前值 ≤ 0 不参与(RULE-025:从 0 起步与透支不报百分比)──

    #[test]
    fn 前值为零的桶不参与比对() {
        let history = vec![snap("2026-08", false, &[("invest", 0), ("spend", 300_000)])];
        let current = snap("2026-09", false, &[("invest", 200_000), ("spend", 300_000)]);
        // invest 从 0 → 2,000:若按百分比会得到 +∞;RULE-025 规定不计
        assert_eq!(deviations(&current, &history, THRESHOLD), Some(vec![]));
    }

    #[test]
    fn 前值为负的桶不参与比对() {
        // arch-e §10 金例:-3,150 → -1,200 是「透支收窄」,不是 +61.9% 的「增长」
        let history = vec![snap("2026-08", false, &[("spend", -315_000)])];
        let current = snap("2026-09", false, &[("spend", -120_000)]);
        assert_eq!(deviations(&current, &history, THRESHOLD), Some(vec![]));
    }

    #[test]
    fn 前值为正而本月透支是真实的下跌偏离() {
        // RULE-025 只拿「前值 > 0」当门槛:本月转负是真实发生的事,报下跌不报百分比爆炸
        let history = vec![snap("2026-08", false, &[("spend", 315_000)])];
        let current = snap("2026-09", false, &[("spend", -120_000)]);
        let out = deviations(&current, &history, THRESHOLD).expect("前值为正,参与比对");
        assert_eq!(out[0].direction, Direction::Down);
        assert_eq!(out[0].deviation_bp, 13_809, "435,000/315,000 = 138.09…%");
    }

    // ── 金例:同名桶才比对(RULE-028)──

    #[test]
    fn 桶结构变化后只比同名桶() {
        let history = vec![snap(
            "2026-08",
            false,
            &[("salary", 1_000_000), ("reserve", 300_000), ("invest", 500_000)],
        )];
        // 换了方案:invest 桶消失,多出 savings 桶 —— 两者都不可比,reserve 照常比对
        let current = snap(
            "2026-09",
            false,
            &[("reserve", 100_000), ("savings", 2_000_000)],
        );
        let out = deviations(&current, &history, THRESHOLD).expect("reserve 同名可比");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].bucket_id, "reserve");
        assert_eq!(out[0].direction, Direction::Down);
    }

    // ── 自定义阈值(ADR-E-003:阈值可调不改代码)──

    #[test]
    fn 阈值收紧后原先放行的波动被报出() {
        let history = vec![snap("2026-08", false, &[("spend", 1_000_000)])];
        let current = snap("2026-09", false, &[("spend", 1_100_000)]);
        assert_eq!(deviations(&current, &history, 2_000), Some(vec![]));
        assert_eq!(
            deviations(&current, &history, 1_000).map(|v| v.len()),
            Some(1),
            "10% 阈下 +10% 命中"
        );
    }

    // ── 金例:缺口月数(RULE-026;prd-e §7.1:目标 ¥77,000、余额 ¥32,000、必要 ¥14,000 → 3.2)──

    #[test]
    fn 缺口月数与产品示例同源() {
        let gap = emergency_gap(7_700_000, 3_200_000, 1_400_000);
        assert!(!gap.met);
        assert_eq!(gap.gap_cents, 4_500_000);
        assert_eq!(gap.gap_months_tenths, 32, "4.5M/1.4M = 3.214 → 半量上取整 3.2");
    }

    #[test]
    fn 缺口月数半量上取整的边界() {
        // 3.25 个十分 → 3.3(半量进位);3.249 → 3.2(不进)
        assert_eq!(
            emergency_gap(4_250_000, 1_000_000, 1_000_000).gap_months_tenths,
            33
        );
        assert_eq!(
            emergency_gap(4_249_000, 1_000_000, 1_000_000).gap_months_tenths,
            32
        );
    }

    #[test]
    fn 余额达标即达标态且缺口为零() {
        let gap = emergency_gap(7_700_000, 7_700_000, 1_400_000);
        assert!(gap.met);
        assert_eq!(gap.gap_cents, 0);
        assert_eq!(gap.gap_months_tenths, 0);

        // 超出目标同样达标(CONTEXT「达标」:余额 ≥ 应急目标)
        let gap = emergency_gap(7_700_000, 9_000_000, 1_400_000);
        assert!(gap.met);
        assert_eq!(gap.gap_cents, 0);
    }

    #[test]
    fn 坏输入下不产生无穷月数() {
        // 防御分支:必要月支出 ≤ 0 时无法定义「还差几个月」,按「未达标但无数可算」落地,
        // 而不是除零 panic 或给一个天文数字 —— 正常方案走不到这里(装载期已校验月数为正)。
        let gap = emergency_gap(7_700_000, 3_200_000, 0);
        assert!(!gap.met);
        assert_eq!(gap.gap_months_tenths, 0);
    }

    // ── 金例:已坚持月数(RULE-027;arch-e §10:1/2/4 月有快照、3 月跳过 → 3)──

    #[test]
    fn 断月不清零且只数有快照的月() {
        assert_eq!(
            persisted_months(&["2026-01", "2026-02", "2026-04"]),
            3,
            "3 月跳过不清零:累计坚持 3 个月"
        );
        assert_eq!(persisted_months(&[]), 0, "首月未录 = 0");
    }

    #[test]
    fn 重复月份按集合语义去重() {
        // 数据层 UNIQUE(user_id, month) 已保证不重;这里的去重是对域函数自身的语义兜底
        assert_eq!(persisted_months(&["2026-01", "2026-01", "2026-02"]), 2);
    }
}
