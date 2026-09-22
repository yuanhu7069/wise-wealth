//! 快照 DTO(RULE-021/022 + §7.3 边界)。
//!
//! 录入金额以**元字符串**过线(用户输入形态),在这里解析成分 —— 解析是纯手写
//! 十进制运算,金额红线(基线 ADR-004:禁 float)从字符串的第一口气就成立。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::tracking::Deviation;

/// 录入/覆盖快照的请求。余额是**元字符串**(如 "3150.50"),键 = 桶 id;
/// 单独一个桶缺了或多了解释权在 service(对照当前方案桶集,422)。
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertSnapshotRequest {
    /// 各桶余额(元字符串,允许负数表示透支,最多两位小数)
    pub balances: BTreeMap<String, String>,
    /// 本月特殊(RULE-024:豁免偏离且不作基准)
    #[serde(default)]
    pub special_month: bool,
}

/// 元字符串 → 分(RULE-022:最多两位小数;|值| < ¥100 亿;允许负数)。
///
/// 手写十进制解析而非 `f64` 中转:字符串 → float → int 的路线会把
/// "0.29" 这类值变成 28.999…,而金额红线不允许「几乎正确」。
pub fn parse_yuan_to_cents(raw: &str) -> Result<i64, String> {
    let s = raw.trim();
    let (negative, digits) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    if digits.is_empty() {
        return Err("金额不能为空".into());
    }
    let (int_part, frac_part) = match digits.split_once('.') {
        // 小数点必须两侧都有数("12." / ".5" 是残缺输入,不是「省略的零」)
        Some((i, f)) if i.is_empty() || f.is_empty() => return Err("金额不合法".into()),
        Some((i, f)) => (i, f),
        None => (digits, ""),
    };
    if int_part.is_empty() {
        return Err("金额不能为空".into());
    }
    if int_part.len() > 10 {
        return Err("金额太大了".into());
    }
    if frac_part.len() > 2 {
        return Err("金额最多两位小数".into());
    }
    if !int_part.chars().all(|c| c.is_ascii_digit())
        || !frac_part.chars().all(|c| c.is_ascii_digit())
    {
        return Err("金额只能是数字".into());
    }
    let int_val: i64 = int_part.parse().map_err(|_| "金额太大了".to_string())?;
    let frac_val: i64 = match frac_part.len() {
        0 => 0,
        1 => frac_part.parse::<i64>().map_err(|_| "金额不合法".to_string())? * 10,
        _ => frac_part.parse().map_err(|_| "金额不合法".to_string())?,
    };
    let cents = int_val * 100 + frac_val;
    if cents >= 1_000_000_000_000 {
        // |值| < ¥100 亿(RULE-022);此处 int_val ≤ 10 位数已兜住大半,这是最后一道
        return Err("金额太大了".into());
    }
    Ok(if negative { -cents } else { cents })
}

/// 月份合法性(格式 YYYY-MM + 真实日历 + 合理年代)。格式错的月份进不了数据库,
/// 年代护栏挡的是 `2000-01` / `9999-12` 这类手滑 —— 快照是按月打卡,不是考古。
pub fn validate_month(month: &str) -> Result<(), String> {
    let bytes = month.as_bytes();
    let ok_shape = bytes.len() == 7
        && bytes[4] == b'-'
        && month[..4].chars().all(|c| c.is_ascii_digit())
        && month[5..].chars().all(|c| c.is_ascii_digit());
    if !ok_shape {
        return Err("月份格式应为 YYYY-MM".into());
    }
    let year: i32 = month[..4].parse().map_err(|_| "月份格式应为 YYYY-MM".to_string())?;
    let m: u32 = month[5..].parse().map_err(|_| "月份格式应为 YYYY-MM".to_string())?;
    if !(2000..=2100).contains(&year) {
        return Err("月份超出合理范围".into());
    }
    if !(1..=12).contains(&m) {
        return Err("月份应在 01 到 12 之间".into());
    }
    Ok(())
}

/// 一条快照(读路径)。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SnapshotView {
    /// 快照 id
    pub id: Uuid,
    /// 自然月(YYYY-MM)
    pub month: String,
    /// 提交时的方案版本号(展示「这个月的数是哪版方案下录的」)
    pub plan_version: i32,
    /// 各桶余额(分;展示层转元加千分位)
    pub balances: BTreeMap<String, i64>,
    /// 本月特殊
    pub special_month: bool,
}

/// 一个桶的偏离结论。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct DeviationView {
    pub bucket_id: String,
    /// 基准值(分)
    pub prev_cents: i64,
    /// 本月值(分)
    pub current_cents: i64,
    /// 偏离幅度(万分比)
    pub deviation_bp: i64,
    /// 方向:up / down
    pub direction: String,
}

impl From<&Deviation> for DeviationView {
    fn from(d: &Deviation) -> Self {
        Self {
            bucket_id: d.bucket_id.clone(),
            prev_cents: d.prev_cents,
            current_cents: d.current_cents,
            deviation_bp: d.deviation_bp,
            direction: match d.direction {
                crate::domain::tracking::Direction::Up => "up".into(),
                crate::domain::tracking::Direction::Down => "down".into(),
            },
        }
    }
}

/// 最新快照的偏离结论。`deviations` 为 null = 没得比(首条/历史全特殊/本月特殊),
/// 空数组 = 有基准且一切如常 —— 两种文案不同(prd-e §8.3)。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LatestDeviationsView {
    pub month: String,
    pub deviations: Option<Vec<DeviationView>>,
}

/// 距离感进度里的应急金结论(RULE-026;`gap_months_tenths` = 缺口月数 ×10,如 32 = 3.2)。
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct EmergencyGapView {
    /// 应急目标(分,方案快照冻结值)
    pub target_cents: i64,
    /// 最新快照应急桶余额(分)
    pub balance_cents: i64,
    /// 缺口(分);达标为 0
    pub gap_cents: i64,
    /// 缺口月数 ×10;达标为 0
    pub gap_months_tenths: i64,
    /// 是否达标
    pub met: bool,
}

/// 追踪摘要(summary 段;P01 追踪卡与 P05 同源)。
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TrackingSummaryView {
    /// 已坚持月数
    pub persisted_months: i64,
    /// 最新快照偏离结论;从未录入为 null
    pub latest: Option<LatestDeviationsView>,
    /// 应急金结论;无快照或方案缺应急桶时为 null(前端给空态文案)
    pub emergency: Option<EmergencyGapView>,
}

/// GET /snapshots 响应。
#[derive(Debug, Serialize, ToSchema)]
pub struct SnapshotsResponse {
    /// 历史快照,按月倒序
    pub items: Vec<SnapshotView>,
    pub summary: TrackingSummaryView,
}

/// PUT /snapshots/{month} 响应:落库后的快照 + 它的偏离结论。
#[derive(Debug, Serialize, ToSchema)]
pub struct SnapshotMutationResponse {
    pub snapshot: SnapshotView,
    /// null = 没得比;空数组 = 一切如常
    pub deviations: Option<Vec<DeviationView>>,
}

/// DELETE /snapshots/{month} 回执(成功也要有 data,基线 §6.1)。
#[derive(Debug, Serialize, ToSchema)]
pub struct SnapshotDeleted {
    pub deleted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 元字符串转分的金例() {
        assert_eq!(parse_yuan_to_cents("3150").unwrap(), 315_000);
        assert_eq!(parse_yuan_to_cents("3150.5").unwrap(), 315_050);
        assert_eq!(parse_yuan_to_cents("3150.50").unwrap(), 315_050);
        assert_eq!(parse_yuan_to_cents("0.29").unwrap(), 29, "float 路线会得到 28.999…");
        assert_eq!(parse_yuan_to_cents("-1200.00").unwrap(), -120_000, "透支是真实状态");
        assert_eq!(parse_yuan_to_cents("0").unwrap(), 0);
        assert_eq!(parse_yuan_to_cents(" 42 ").unwrap(), 4_200, "容忍首尾空白");
        assert_eq!(parse_yuan_to_cents("9999999999.99").unwrap(), 999_999_999_999, "¥100 亿 - 0.01");
    }

    #[test]
    fn 非法金额逐类被拒() {
        for bad in ["", "-", ".", "12.345", "12,000", "1e9", "abc", "12.", ".5", "-"] {
            assert!(parse_yuan_to_cents(bad).is_err(), "{bad:?} 应被拒");
        }
        // 超上限:¥100 亿整
        assert!(parse_yuan_to_cents("10000000000").is_err());
    }

    #[test]
    fn 月份格式与日历校验() {
        assert!(validate_month("2026-09").is_ok());
        assert!(validate_month("2026-01").is_ok());
        for bad in [
            "2026-9",
            "2026-13",
            "2026-00",
            "26-09",
            "2026/09",
            "209901",
            "9999-12",
            "1999-12",
            "2026-1x",
        ] {
            assert!(validate_month(bad).is_err(), "{bad:?} 应被拒");
        }
    }
}
