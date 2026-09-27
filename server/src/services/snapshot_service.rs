//! 快照 service(RULE-021 ~ RULE-029):校验 → 落库 → 追踪结论编排。
//!
//! 业务判断都在这一层:桶集合合法性(ADR-E-001 的 service 兜底)、未来月份拒绝、
//! 「仅最新月可删」。偏离与进度的**口径**在 [`crate::domain::tracking`] 纯函数里,
//! 这里只负责把数据摆到它面前 —— 前端零计算(ADR-E-002)。

use std::collections::BTreeMap;

use serde_json::Value as Json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::engine::EmergencyStatus;
use crate::domain::tracking::{
    self, Deviation, EmergencyGap, SnapshotPoint,
};
use crate::domain::ModeLibrary;
use crate::repos;
use crate::repos::snapshots::SnapshotRow;

/// 快照业务失败。区分「用户能自己修」(校验类)与「系统问题」,前端给不同文案。
#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    /// 还没有生成过方案(EMPTY-E-01:先去问卷)
    #[error("还没有方案,先去生成一份")]
    NoActivePlan,
    /// 提交的桶集合与当前方案不一致(多桶 / 少桶 / 未知桶)
    #[error("提交的账户与当前方案不一致: {0}")]
    BucketSetMismatch(String),
    /// 只允许写当月:未来月不存在,历史月不可变(RULE-021/028)
    #[error("快照只能记录当月({current}),{month} 无法提交")]
    MonthNotAllowed {
        /// 被拒绝的月份
        month: String,
        /// 服务器当前月
        current: String,
    },
    /// 删除目标不是最新月(或当月无快照)
    #[error("只能删除最新一个月的快照")]
    NotLatestMonth,
    /// 方案快照 JSON 损坏(应急金状态反序列化失败;属程序缺陷而非用户可修)
    #[error("方案快照数据损坏: {0}")]
    CorruptedSnapshot(String),
    /// 季度参数非法(H 期:格式应为 YYYY-QN,年份 2026-2100)
    #[error("季度格式应为 YYYY-QN:{0}")]
    InvalidQuarter(String),
    /// 落库失败
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// 服务器认定的「当前自然月」(YYYY-MM)。
///
/// 单独成函数:时间不是域层的东西(纯函数不认识时钟),但**决策**要用它 ——
/// 隔出来才好在代码里一眼看到「这里有一条与真实时间有关的边界」。
/// 当前月是**后端权威值**(API 随响应下发,前端不自己算 —— 评审发现 #4)。
pub fn current_month() -> String {
    chrono::Local::now().format("%Y-%m").to_string()
}

/// 桶集合校验(纯函数):提交的键集必须与当前方案桶集**恰好相等**。
///
/// 这是 ADR-E-001 的兜底:JSONB 没有关系级约束,「 balances 里混进一个方案里
/// 不存在的桶」只能在这里拦 —— 拦不住的话,偏离比对与缺口计算会拿到脏数据。
pub fn validate_bucket_set(
    expected: &[String],
    balances: &BTreeMap<String, i64>,
) -> Result<(), SnapshotError> {
    let unexpected: Vec<&str> = balances
        .keys()
        .filter(|k| !expected.iter().any(|e| e == *k))
        .map(String::as_str)
        .collect();
    let missing: Vec<&str> = expected
        .iter()
        .filter(|e| !balances.contains_key(e.as_str()))
        .map(String::as_str)
        .collect();
    if unexpected.is_empty() && missing.is_empty() {
        return Ok(());
    }
    let mut detail = Vec::new();
    if !missing.is_empty() {
        detail.push(format!("缺少 {}", missing.join(", ")));
    }
    if !unexpected.is_empty() {
        detail.push(format!("多出 {}", unexpected.join(", ")));
    }
    Err(SnapshotError::BucketSetMismatch(detail.join("；")))
}

/// repo 行 → 域层快照点。JSON 损坏在提交路径上不可能(刚写的就是刚校验的),
/// 读路径上等于程序缺陷,如实报错。
fn point_from_row(row: &SnapshotRow) -> Result<SnapshotPoint, SnapshotError> {
    let balances: BTreeMap<String, i64> = serde_json::from_value(row.balances.clone())
        .map_err(|e| SnapshotError::CorruptedSnapshot(format!("余额对象损坏: {e}")))?;
    Ok(SnapshotPoint {
        month: row.month.clone(),
        balances,
        special_month: row.special_month,
    })
}

/// 一条快照的偏离结论:基准存在与否是**业务事实**(`None` = 没得比),
/// 与「比了但没超阈」(`Some(空)`)是两回事,前端文案不同。
#[derive(Debug, Clone)]
pub struct LatestDeviations {
    pub month: String,
    pub deviations: Option<Vec<Deviation>>,
}

/// 给定最新快照,算它的偏离结论(`baseline` 已按 RULE-025 由 repo 选出)。
fn deviations_for(
    latest: &SnapshotRow,
    baseline: Option<SnapshotRow>,
    threshold_bp: i64,
) -> Result<Option<Vec<Deviation>>, SnapshotError> {
    let current = point_from_row(latest)?;
    let history: Vec<SnapshotPoint> = match baseline {
        Some(b) => vec![point_from_row(&b)?],
        None => vec![],
    };
    Ok(tracking::deviations(&current, &history, threshold_bp))
}

/// 提交(或覆盖)当月快照,返回落库结果与新快照的偏离结论。
///
/// **只允许当月**(RULE-021/028):未来月不存在;历史月不可变 —— 覆盖过去会
/// 连带改写 plan_id 冻结与偏离基准,是历史被篡改的入口(评审发现 #1,原实现漏检)。
pub async fn upsert(
    pool: &PgPool,
    threshold_bp: i64,
    user_id: Uuid,
    month: &str,
    balances: BTreeMap<String, i64>,
    special_month: bool,
) -> Result<(SnapshotRow, bool, Option<Vec<Deviation>>), SnapshotError> {
    // 1. 月份门槛(格式由 DTO 层校验,基线 §4.5)
    let current = current_month();
    if month != current.as_str() {
        return Err(SnapshotError::MonthNotAllowed {
            month: month.to_string(),
            current,
        });
    }

    // 2. 桶集合必须与当前 active 方案一致(RULE-021 全桶必填)
    let plan = repos::plans::active_for_user(pool, user_id)
        .await?
        .ok_or(SnapshotError::NoActivePlan)?;
    let buckets = repos::plans::buckets_of(pool, plan.id).await?;
    let expected: Vec<String> = buckets.iter().map(|b| b.bucket_id.clone()).collect();
    validate_bucket_set(&expected, &balances)?;

    // 3. 落库(同月覆盖;方案版本号随行冻结)
    let json: Json = serde_json::to_value(&balances).unwrap_or(Json::Null);
    let (row, inserted) = repos::snapshots::upsert(
        pool,
        user_id,
        plan.id,
        plan.version,
        month,
        &json,
        special_month,
    )
    .await?;

    // 4. 本次快照的偏离结论(基准 = 之前最近一条非特殊快照,RULE-024/025)
    let baseline = repos::snapshots::prev_non_special_before(pool, user_id, month).await?;
    let deviations = deviations_for(&row, baseline, threshold_bp)?;

    Ok((row, inserted, deviations))
}

/// 距离感进度里的应急金结论(展示面)。
#[derive(Debug, Clone, Copy)]
pub struct EmergencyGapView {
    /// 应急目标(分,方案快照冻结值)
    pub target_cents: i64,
    /// 最新快照的应急桶余额(分)
    pub balance_cents: i64,
    /// 缺口结论(RULE-026)
    pub gap: EmergencyGap,
}

/// 追踪摘要(GET /snapshots 的 summary 段;P01 追踪卡与 P05 同源,ADR-E-002)。
#[derive(Debug, Clone)]
pub struct TrackingSummary {
    /// 已坚持月数(RULE-027)
    pub persisted_months: i64,
    /// 最新快照的偏离结论;从未录入过快照则为 None
    pub latest: Option<LatestDeviations>,
    /// 应急金结论;没有快照或方案缺应急桶时为 None(空态文案由前端给)
    pub emergency: Option<EmergencyGapView>,
}

/// 应急金的「余额观察桶」:有规则桶用规则桶(四账户 = 备用账户),
/// 无规则桶用投资桶(50/30/20 = 储蓄账户 —— 应急金在未达标期由它补位,
/// 见 fifty_30_20.toml 的桶注释)。桶 id 是稳定标识(mode.rs),跨版本可用。
fn emergency_bucket_id(library: &ModeLibrary, l1_mode: &str) -> Option<String> {
    let mode = library.mode(l1_mode)?;
    mode.rule_bucket()
        .or_else(|| mode.investable_bucket())
        .map(|b| b.id.clone())
}

/// 组装追踪摘要。
pub async fn summary(
    pool: &PgPool,
    library: &ModeLibrary,
    threshold_bp: i64,
    user_id: Uuid,
) -> Result<TrackingSummary, SnapshotError> {
    let persisted_months = repos::snapshots::count(pool, user_id).await?;

    let latest_row = repos::snapshots::latest(pool, user_id).await?;
    let Some(latest_row) = latest_row else {
        return Ok(TrackingSummary {
            persisted_months,
            latest: None,
            emergency: None,
        });
    };

    // 偏离结论
    let baseline =
        repos::snapshots::prev_non_special_before(pool, user_id, &latest_row.month).await?;
    let latest = LatestDeviations {
        month: latest_row.month.clone(),
        deviations: deviations_for(&latest_row, baseline, threshold_bp)?,
    };

    // 应急金结论:目标与必要月支出来自方案快照(RULE-026 零新口径)。
    // 三道门槛各自成立才给结论:方案快照可读、能定位余额观察桶、快照里确有该桶 ——
    // 缺任何一环都不编一个假结论,空态文案交给前端。
    let mut emergency = None;
    if let Ok((status, l1_mode)) = emergency_status_of(pool, latest_row.plan_id).await
        && let Some(bucket_id) = emergency_bucket_id(library, &l1_mode)
        && let Some(balance_cents) = point_from_row(&latest_row)?.balances.get(&bucket_id).copied()
    {
        emergency = Some(EmergencyGapView {
            target_cents: status.target_cents,
            balance_cents,
            gap: tracking::emergency_gap(
                status.target_cents,
                balance_cents,
                status.necessary_monthly_cents,
            ),
        });
    }

    Ok(TrackingSummary {
        persisted_months,
        latest: Some(latest),
        emergency,
    })
}

/// 读快照所属方案版本的应急金状态与 L1 模式 id(都是生成时冻结的)。
async fn emergency_status_of(
    pool: &PgPool,
    plan_id: Uuid,
) -> Result<(EmergencyStatus, String), SnapshotError> {
    let plan = repos::plans::by_id(pool, plan_id)
        .await?
        .ok_or_else(|| SnapshotError::CorruptedSnapshot("快照指向的方案版本不存在".into()))?;
    let status: EmergencyStatus = serde_json::from_value(plan.emergency)
        .map_err(|e| SnapshotError::CorruptedSnapshot(format!("应急金快照损坏: {e}")))?;
    Ok((status, plan.l1_mode))
}

/// 删除快照(RULE-029:仅最新月;录错恢复口)。
pub async fn delete_month(
    pool: &PgPool,
    user_id: Uuid,
    month: &str,
) -> Result<(), SnapshotError> {
    let latest = repos::snapshots::latest(pool, user_id).await?;
    match latest {
        Some(row) if row.month == month => {
            repos::snapshots::delete_month(pool, user_id, month).await?;
            Ok(())
        }
        _ => Err(SnapshotError::NotLatestMonth),
    }
}

/// CSV 导出长表( RULE-030):月 × 桶一行。此处返回**原始字段**,
/// 转义统一由 `render_csv`(emit_cell)执行 —— 两层各转一次会把引号翻倍(评审发现 #6)。
pub async fn export_csv_rows(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<(Vec<&'static str>, Vec<Vec<String>>), SnapshotError> {
    let rows = repos::snapshots::export_rows(pool, user_id).await?;
    let lines = rows
        .into_iter()
        .map(|r| {
            vec![
                r.month,
                r.version.to_string(),
                r.bucket_id.clone(),
                r.bucket_name.unwrap_or(r.bucket_id),
                crate::domain::csv::yuan_string(r.cents),
                if r.special_month { "是" } else { "" }.to_string(),
            ]
        })
        .collect();
    Ok((
        vec!["月份", "方案版本", "桶ID", "桶名", "余额(元)", "本月特殊"],
        lines,
    ))
}

// ── 季度复盘(H 期 RULE-049/050,ADR-H-003)──

/// 季度参数缺省时的「当前季度」(服务器时钟权威,同 `current_month` 立场)。
pub fn current_quarter() -> String {
    use chrono::Datelike;
    let now = chrono::Local::now();
    let q = (now.month() - 1) / 3 + 1;
    format!("{}-Q{}", now.year(), q)
}

/// 季度参数解析:缺省 → 当前季;格式 `YYYY-QN`,年份 2026..=2100(首份可看报告 =
/// 2026 Q4,更早的季度直接给空集而非报错;手滑的远古/未来年份按非法拒绝)。
pub fn resolve_quarter(param: Option<&str>) -> Result<String, SnapshotError> {
    let Some(raw) = param else {
        return Ok(current_quarter());
    };
    let bytes = raw.as_bytes();
    let ok_shape = bytes.len() == 7
        && bytes[4] == b'-'
        && bytes[5] == b'Q'
        && raw[..4].chars().all(|c| c.is_ascii_digit())
        && raw[6..].chars().all(|c| c.is_ascii_digit());
    if !ok_shape {
        return Err(SnapshotError::InvalidQuarter(raw.to_string()));
    }
    let year: i32 = raw[..4].parse().map_err(|_| SnapshotError::InvalidQuarter(raw.to_string()))?;
    let q: u32 = raw[6..].parse().map_err(|_| SnapshotError::InvalidQuarter(raw.to_string()))?;
    if !(2026..=2100).contains(&year) || !(1..=4).contains(&q) {
        return Err(SnapshotError::InvalidQuarter(raw.to_string()));
    }
    Ok(raw.to_string())
}

/// 季度 → 自然月区间 `[start_ym, end_ym_exclusive)`(repo 查询的半开区间形态)。
fn quarter_month_range(quarter: &str) -> (String, String) {
    let year: i32 = quarter[..4].parse().expect("resolve_quarter 已校验过形状");
    let q: u32 = quarter[6..].parse().expect("resolve_quarter 已校验过形状");
    let start_month = 3 * (q - 1) + 1;
    let (end_year, end_month) = if q == 4 {
        (year + 1, 1)
    } else {
        (year, 3 * q + 1)
    };
    (
        format!("{year}-{start_month:02}"),
        format!("{end_year}-{end_month:02}"),
    )
}

/// 复盘聚合的完整产物:域结论 + 桶名映射(名称随季末快照所属方案版本冻结)。
#[derive(Debug, Clone)]
pub struct QuarterReviewData {
    /// 域层聚合结论
    pub review: tracking::QuarterReview,
    /// 桶 id → 展示名(取季末非特殊快照所属方案版本的 `plan_buckets`;缺名回落桶 id)
    pub names: BTreeMap<String, String>,
}

/// 季度复盘聚合(RULE-049/050):读本季快照与方案口径,把数据摆到域函数面前 ——
/// 口径在 [`crate::domain::tracking::quarter_review`],与 E 期追踪摘要同源
/// (应急目标/必要月支出取**季末非特殊快照所属方案版本**的冻结值)。
pub async fn quarter_review(
    pool: &PgPool,
    library: &ModeLibrary,
    user_id: Uuid,
    quarter_param: Option<&str>,
) -> Result<QuarterReviewData, SnapshotError> {
    let quarter = resolve_quarter(quarter_param)?;
    let (start_ym, end_ym) = quarter_month_range(&quarter);

    let persisted = repos::snapshots::count(pool, user_id).await?;
    let rows = repos::snapshots::list_range_asc(pool, user_id, &start_ym, &end_ym).await?;
    let points: Vec<SnapshotPoint> = rows
        .iter()
        .map(point_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    // 应急输入:季末非特殊快照所属方案版本的冻结口径(与 E 期 summary 同源)
    let last_normal = rows.iter().rev().find(|r| !r.special_month);
    let mut emergency_ref = None;
    if let Some(row) = last_normal
        && let Ok((status, l1_mode)) = emergency_status_of(pool, row.plan_id).await
        && let Some(bucket_id) = emergency_bucket_id(library, &l1_mode)
    {
        emergency_ref = Some(crate::domain::tracking::EmergencyRef {
            target_cents: status.target_cents,
            necessary_monthly_cents: status.necessary_monthly_cents,
            bucket_id,
        });
    }

    let review = crate::domain::tracking::quarter_review(
        &quarter,
        persisted as usize,
        &points,
        emergency_ref,
    );

    // 桶名:随季末非特殊快照所属方案版本(同 E 期 CSV 的按版本取名立场)
    let mut names = BTreeMap::new();
    if let Some(row) = last_normal {
        for b in repos::plans::buckets_of(pool, row.plan_id).await? {
            names.insert(b.bucket_id, b.name);
        }
    }

    Ok(QuarterReviewData { review, names })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 桶集合校验(ADR-E-001 兜底;纯函数,金例即四账户桶集)──

    fn four_buckets() -> Vec<String> {
        ["salary", "spend", "reserve", "invest"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn 桶集恰好相等时通过() {
        let balances = BTreeMap::from([
            ("salary".into(), 1),
            ("spend".into(), 2),
            ("reserve".into(), 3),
            ("invest".into(), 4),
        ]);
        assert!(validate_bucket_set(&four_buckets(), &balances).is_ok());
    }

    #[test]
    fn 缺桶与多桶分别指出() {
        // 缺 reserve/invest,多出 savings:错误信息要说清两边,不能只报一个数
        let balances = BTreeMap::from([("salary".into(), 1), ("savings".into(), 2)]);
        let err = validate_bucket_set(&four_buckets(), &balances).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("缺少"), "实际: {msg}");
        assert!(msg.contains("reserve"), "实际: {msg}");
        assert!(msg.contains("多出"), "实际: {msg}");
        assert!(msg.contains("savings"), "实际: {msg}");
    }

    #[test]
    fn 全缺时按缺桶报错() {
        let err = validate_bucket_set(&four_buckets(), &BTreeMap::new()).unwrap_err();
        assert!(err.to_string().contains("缺少"));
    }

    // ── 应急桶选择(有规则桶用规则桶,无则投资桶)──

    #[test]
    fn 四账户的应急桶是备用账户() {
        let lib = ModeLibrary::load_embedded().unwrap();
        assert_eq!(
            emergency_bucket_id(&lib, "four_accounts").as_deref(),
            Some("reserve")
        );
    }

    #[test]
    fn 五十三十二十的应急桶回落到储蓄账户() {
        let lib = ModeLibrary::load_embedded().unwrap();
        // 50/30/20 无规则桶:应急金未达标期由储蓄桶补位,观察它才对得上「攒到哪了」
        assert_eq!(
            emergency_bucket_id(&lib, "fifty_30_20").as_deref(),
            Some("savings")
        );
    }
}
