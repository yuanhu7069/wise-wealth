//! 档案(Profile):问卷答案 + 财务数字,引擎的输入。
//!
//! 只包含**引擎实际消费**的字段。问卷 Q4 采集的社保/商业保险/房贷余额本期仅入库备查
//! (产品 PRD §13.3 将房贷对比排在后续期),故不出现在本结构里 —— 与其在引擎里放三个
//! 用不到的字段,不如让「引擎吃什么」一眼可见。

use serde::{Deserialize, Serialize};

/// 资金久期(问卷第 1 题):唯一直接决定这笔钱能不能进权益类的变量(产品 PRD §4.2.2 Q1)。
///
/// 变体名带数字,serde 的自动 snake_case 转换在数字边界上不可预期,故**逐个显式 rename**:
/// 这些字符串会写进数据库文本列与 API 载荷,改动即数据迁移。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Horizon {
    /// 1 年内
    #[serde(rename = "within_1y")]
    Within1y,
    /// 1-3 年
    #[serde(rename = "y1_3")]
    Y1to3,
    /// 3-5 年
    #[serde(rename = "y3_5")]
    Y3to5,
    /// 5-10 年
    #[serde(rename = "y5_10")]
    Y5to10,
    /// 10 年以上
    #[serde(rename = "over_10")]
    Over10,
}

/// 回撤反应(问卷第 2 题):行为指标,比风险偏好自评有效一个数量级(产品 PRD §4.2.1)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawdownResponse {
    /// 清仓,先落袋
    Liquidate,
    /// 减仓观察
    Reduce,
    /// 不动,按计划执行
    Hold,
    /// 加仓,跌了更便宜
    Add,
}

/// 收入稳定性(问卷第 3 题):决定应急金要留几个月。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncomeStability {
    /// 很稳
    Stable,
    /// 一般
    Normal,
    /// 波动大
    Volatile,
    /// 自由职业
    Freelance,
}

/// 理财目标(问卷第 5 题):本期**采集、入库、在方案页展示,但不参与任何计算**
/// (为追踪期的「距离感进度条」与将来的达成概率预留)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Goal {
    /// 购房
    House,
    /// 子女教育
    Education,
    /// 退休
    Retirement,
    /// 财富增值
    Wealth,
}

impl Horizon {
    /// 中文名:供匹配理由拼装与页面展示。
    pub fn label(self) -> &'static str {
        match self {
            Horizon::Within1y => "1 年内",
            Horizon::Y1to3 => "1-3 年",
            Horizon::Y3to5 => "3-5 年",
            Horizon::Y5to10 => "5-10 年",
            Horizon::Over10 => "10 年以上",
        }
    }
}

impl DrawdownResponse {
    /// 中文名:供匹配理由拼装与页面展示。
    pub fn label(self) -> &'static str {
        match self {
            DrawdownResponse::Liquidate => "清仓",
            DrawdownResponse::Reduce => "减仓观察",
            DrawdownResponse::Hold => "不动",
            DrawdownResponse::Add => "加仓",
        }
    }
}

impl IncomeStability {
    /// 中文名:供推理链条目与页面展示。
    pub fn label(self) -> &'static str {
        match self {
            IncomeStability::Stable => "很稳",
            IncomeStability::Normal => "一般",
            IncomeStability::Volatile => "波动大",
            IncomeStability::Freelance => "自由职业",
        }
    }
}

impl Goal {
    /// 中文名:供方案页头部展示。
    pub fn label(self) -> &'static str {
        match self {
            Goal::House => "购房",
            Goal::Education => "子女教育",
            Goal::Retirement => "退休",
            Goal::Wealth => "财富增值",
        }
    }
}

/// 一份档案:引擎的完整输入。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// 资金久期 —— 决定权益上限
    pub horizon: Horizon,
    /// 回撤反应 —— 决定 L2 基础档
    pub drawdown_response: DrawdownResponse,
    /// 收入稳定性 —— 决定应急月数
    pub income_stability: IncomeStability,
    /// 需赡养人数 —— > 0 时应急月数上浮一档
    pub dependents: u32,
    /// 税后月收入(分) —— 比例桶的基数
    pub inflow_cents: i64,
    /// 月固定支出(分) —— 四账户「工资桶」的金额
    pub expense_fixed_monthly_cents: i64,
    /// 现有存款(分) —— 整体视为备用账户余额
    pub savings_cents: i64,
    /// 理财目标 —— 本期不参与计算,仅随方案展示
    pub goal: Goal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 枚举字符串表示锁定() {
        // 这些字符串进数据库文本列与 API 载荷,改动即数据迁移,故逐一锁定。
        assert_eq!(serde_json::to_string(&Horizon::Within1y).unwrap(), "\"within_1y\"");
        assert_eq!(serde_json::to_string(&Horizon::Y1to3).unwrap(), "\"y1_3\"");
        assert_eq!(serde_json::to_string(&Horizon::Y3to5).unwrap(), "\"y3_5\"");
        assert_eq!(serde_json::to_string(&Horizon::Y5to10).unwrap(), "\"y5_10\"");
        assert_eq!(serde_json::to_string(&Horizon::Over10).unwrap(), "\"over_10\"");
        assert_eq!(
            serde_json::to_string(&DrawdownResponse::Liquidate).unwrap(),
            "\"liquidate\""
        );
        assert_eq!(
            serde_json::to_string(&IncomeStability::Freelance).unwrap(),
            "\"freelance\""
        );
        assert_eq!(serde_json::to_string(&Goal::Wealth).unwrap(), "\"wealth\"");
    }

    #[test]
    fn 枚举可往返() {
        for h in [
            Horizon::Within1y,
            Horizon::Y1to3,
            Horizon::Y3to5,
            Horizon::Y5to10,
            Horizon::Over10,
        ] {
            let s = serde_json::to_string(&h).unwrap();
            assert_eq!(serde_json::from_str::<Horizon>(&s).unwrap(), h);
        }
        let d: DrawdownResponse = serde_json::from_str("\"liquidate\"").unwrap();
        assert_eq!(d, DrawdownResponse::Liquidate);
    }

    #[test]
    fn 中文名齐全() {
        assert_eq!(Horizon::Within1y.label(), "1 年内");
        assert_eq!(Horizon::Over10.label(), "10 年以上");
        assert_eq!(DrawdownResponse::Add.label(), "加仓");
        assert_eq!(IncomeStability::Volatile.label(), "波动大");
        assert_eq!(Goal::House.label(), "购房");
    }
}
