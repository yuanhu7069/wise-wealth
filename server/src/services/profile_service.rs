//! 问卷档案 service(RULE-003 / RULE-004 / RULE-005)。
//!
//! 分步保存与步进校验都是**纯函数** `apply_step`:输入既有档案 + 本步答案,输出合并后的
//! 档案与下一步号。这样「哪一步必填什么」这条规则可以脱离数据库逐条测试,
//! 而 handler 只剩「读 → apply_step → 写」三行编排。

use crate::domain::profile::wire;
use crate::dto::profile::StepRequest;
use crate::repos::profiles::ProfileRow;

/// 步进校验失败。文案面向用户,不含技术细节(RULE-008 三要素里的「为什么」)。
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StepError {
    /// 步号越界
    #[error("步号必须在 1-5 之间")]
    InvalidStep,
    /// 必填项缺失
    #[error("{0}")]
    Missing(String),
    /// 取值越界
    #[error("{0}")]
    OutOfRange(String),
}

/// 需赡养人数的上限(与 prd-v1 RULE-004 一致)
const MAX_DEPENDENTS: i16 = 20;
/// 问卷最后一题的步号(步 6 是推荐,不落库)
const LAST_ANSWER_STEP: i16 = 5;
/// 步进上限:答完步 5 之后停在步 6(推荐)
const MAX_DRAFT_STEP: i16 = 6;

/// 把某一步的答案合并进既有档案。
///
/// 返回合并后的行与新的草稿步号。不修改入参(纯函数)。
pub fn apply_step(current: &ProfileRow, req: &StepRequest) -> Result<ProfileRow, StepError> {
    if req.step < 1 || req.step > LAST_ANSWER_STEP {
        return Err(StepError::InvalidStep);
    }

    let mut next = current.clone();

    match req.step {
        1 => {
            let horizon = req
                .horizon
                .ok_or_else(|| StepError::Missing("请选择这笔钱几年后要用".into()))?;
            next.horizon = Some(wire(&horizon));
        }
        2 => {
            let drawdown = req
                .drawdown_response
                .ok_or_else(|| StepError::Missing("请选择一个回答".into()))?;
            next.drawdown_response = Some(wire(&drawdown));
        }
        3 => {
            let stability = req
                .income_stability
                .ok_or_else(|| StepError::Missing("请选择收入是否稳定".into()))?;
            next.income_stability = Some(wire(&stability));
        }
        4 => {
            let dependents = req
                .dependents
                .ok_or_else(|| StepError::Missing("请填写需赡养人数(没有请填 0)".into()))?;
            if !(0..=MAX_DEPENDENTS).contains(&dependents) {
                return Err(StepError::OutOfRange(format!(
                    "需赡养人数需在 0-{MAX_DEPENDENTS} 之间"
                )));
            }
            if let Some(mortgage) = req.mortgage_balance_cents
                && mortgage < 0
            {
                return Err(StepError::OutOfRange("房贷余额不能为负数".into()));
            }
            next.dependents = Some(dependents);
            next.has_social_security = req.has_social_security.unwrap_or(false);
            next.has_commercial_insurance = req.has_commercial_insurance.unwrap_or(false);
            next.mortgage_balance_cents = req.mortgage_balance_cents;
        }
        5 => {
            let inflow = req
                .inflow_cents
                .ok_or_else(|| StepError::Missing("请填写月收入".into()))?;
            if inflow <= 0 {
                return Err(StepError::OutOfRange("金额需大于 0".into()));
            }
            let fixed = req
                .expense_fixed_monthly_cents
                .ok_or_else(|| StepError::Missing("请填写月固定支出(没有请填 0)".into()))?;
            if fixed < 0 {
                return Err(StepError::OutOfRange("金额不能为负数".into()));
            }
            let savings = req.savings_cents.unwrap_or(0);
            if savings < 0 {
                return Err(StepError::OutOfRange("金额不能为负数".into()));
            }
            let goal = req
                .goal
                .ok_or_else(|| StepError::Missing("请选择一个理财目标".into()))?;

            next.inflow_cents = Some(inflow);
            next.expense_fixed_monthly_cents = Some(fixed);
            next.savings_cents = Some(savings);
            next.goal = Some(wire(&goal));
            next.questionnaire_completed = next.is_complete();
        }
        _ => return Err(StepError::InvalidStep),
    }

    // 步进永不回退:重新答旧步(重新生成时预填后改数字)不该把进度打回去
    let advanced = (req.step + 1).min(MAX_DRAFT_STEP);
    next.draft_step = current.draft_step.max(advanced);

    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::{DrawdownResponse, Goal, Horizon, IncomeStability};

    fn empty() -> ProfileRow {
        ProfileRow::default()
    }

    fn step1() -> StepRequest {
        StepRequest {
            step: 1,
            horizon: Some(Horizon::Y5to10),
            drawdown_response: None,
            income_stability: None,
            has_social_security: None,
            has_commercial_insurance: None,
            mortgage_balance_cents: None,
            dependents: None,
            inflow_cents: None,
            expense_fixed_monthly_cents: None,
            savings_cents: None,
            goal: None,
        }
    }

    fn step5() -> StepRequest {
        StepRequest {
            step: 5,
            horizon: None,
            drawdown_response: None,
            income_stability: None,
            has_social_security: None,
            has_commercial_insurance: None,
            mortgage_balance_cents: None,
            dependents: None,
            inflow_cents: Some(1_200_000),
            expense_fixed_monthly_cents: Some(450_000),
            savings_cents: Some(2_400_000),
            goal: Some(Goal::Wealth),
        }
    }

    #[test]
    fn 步一保存久期并推进到步二() {
        let row = apply_step(&empty(), &step1()).unwrap();
        assert_eq!(row.horizon.as_deref(), Some("y5_10"));
        assert_eq!(row.draft_step, 2);
        assert!(!row.questionnaire_completed);
    }

    #[test]
    fn 必填项缺失被拒且说明原因() {
        let mut req = step1();
        req.horizon = None;
        let err = apply_step(&empty(), &req).unwrap_err();
        assert!(matches!(err, StepError::Missing(_)));
        assert!(err.to_string().contains("几年后要用"), "实际: {err}");
    }

    #[test]
    fn 步号越界被拒() {
        for bad in [0, 6, 99] {
            let mut req = step1();
            req.step = bad;
            assert_eq!(apply_step(&empty(), &req).unwrap_err(), StepError::InvalidStep);
        }
    }

    #[test]
    fn 步四_赡养人数必填且限界() {
        let mut req = step1();
        req.step = 4;
        // 缺失
        assert!(apply_step(&empty(), &req).is_err());
        // 越界
        req.dependents = Some(21);
        assert!(matches!(
            apply_step(&empty(), &req).unwrap_err(),
            StepError::OutOfRange(_)
        ));
        // 合法 + 其余可空
        req.dependents = Some(1);
        let row = apply_step(&empty(), &req).unwrap();
        assert_eq!(row.dependents, Some(1));
        assert!(!row.has_social_security, "未勾选视为无");
        assert_eq!(row.mortgage_balance_cents, None, "选填留空即为空");
        assert_eq!(row.draft_step, 5);
    }

    #[test]
    fn 步五_收入必须为正() {
        let mut req = step5();
        req.inflow_cents = Some(0);
        assert!(matches!(
            apply_step(&empty(), &req).unwrap_err(),
            StepError::OutOfRange(_)
        ));
        req.inflow_cents = Some(-1);
        assert!(apply_step(&empty(), &req).is_err());
    }

    #[test]
    fn 步五_存款缺省视为零() {
        let mut req = step5();
        req.savings_cents = None;
        let row = apply_step(&empty(), &req).unwrap();
        assert_eq!(row.savings_cents, Some(0), "选填缺省应为 0 而非 NULL");
    }

    #[test]
    fn 步五答全才标记完成() {
        // 只有步 5:前面的步没答,不算完成(否则推荐端点会拿到半份档案)
        let only5 = apply_step(&empty(), &step5()).unwrap();
        assert!(!only5.questionnaire_completed, "跳步作答不该标记完成");
        assert_eq!(only5.draft_step, 6, "进度仍推进到推荐步");

        // 逐步答全:标记完成
        let mut row = apply_step(&empty(), &step1()).unwrap();
        let mut r2 = step1();
        r2.step = 2;
        r2.drawdown_response = Some(DrawdownResponse::Hold);
        row = apply_step(&row, &r2).unwrap();
        let mut r3 = step1();
        r3.step = 3;
        r3.income_stability = Some(IncomeStability::Volatile);
        row = apply_step(&row, &r3).unwrap();
        let mut r4 = step1();
        r4.step = 4;
        r4.dependents = Some(1);
        row = apply_step(&row, &r4).unwrap();
        let row = apply_step(&row, &step5()).unwrap();
        assert!(row.questionnaire_completed, "答全五步才标记完成");
    }

    #[test]
    fn 完整判定不看标志位只看字段() {
        // 人为构造「标志为真但字段缺失」的行,is_complete 必须说假
        let row = ProfileRow {
            questionnaire_completed: true,
            ..ProfileRow::default()
        };
        assert!(!row.is_complete(), "标志位不可信,要看字段");
    }

    #[test]
    fn 重答旧步不回退进度() {
        // 先答到步 5
        let mut row = apply_step(&empty(), &step1()).unwrap();
        let mut req2 = step1();
        req2.step = 2;
        req2.drawdown_response = Some(DrawdownResponse::Hold);
        row = apply_step(&row, &req2).unwrap();
        let mut req3 = step1();
        req3.step = 3;
        req3.income_stability = Some(IncomeStability::Volatile);
        row = apply_step(&row, &req3).unwrap();
        assert_eq!(row.draft_step, 4);

        // 回头改步 1(重新生成时预填后改答案):进度必须留在步 4
        let mut redo1 = step1();
        redo1.horizon = Some(Horizon::Y3to5);
        let row = apply_step(&row, &redo1).unwrap();
        assert_eq!(row.horizon.as_deref(), Some("y3_5"), "答案应被更新");
        assert_eq!(row.draft_step, 4, "进度不得回退");
    }

    #[test]
    fn 合并只动本步字段() {
        // 步 5 提交不得清掉步 1-3 的答案
        let mut row = apply_step(&empty(), &step1()).unwrap();
        let mut req2 = step1();
        req2.step = 2;
        req2.drawdown_response = Some(DrawdownResponse::Hold);
        row = apply_step(&row, &req2).unwrap();

        let row = apply_step(&row, &step5()).unwrap();
        assert_eq!(row.horizon.as_deref(), Some("y5_10"), "步 1 答案应保留");
        assert_eq!(row.drawdown_response.as_deref(), Some("hold"));
        assert_eq!(row.goal.as_deref(), Some("wealth"));
    }
}
