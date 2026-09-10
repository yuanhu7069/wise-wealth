//! 方案生成 service:读档案 → 调引擎 → 版本化落库。
//!
//! 编排之外只剩一件事是「业务」:**生成即快照**。档案、L2 配置、应急金状态、提示与
//! 推理链全部冻结进方案行 —— 模式库配置日后修改,不应改写用户手里已有的那张表。

use serde_json::Value as Json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::engine::{self, PlanResult};
use crate::domain::profile::Profile;
use crate::domain::ModeLibrary;
use crate::repos::plans::{self, BucketRow, NewPlan, PlanRecord};

/// 生成失败的原因。区分「用户能自己修」与「系统问题」,前端才好给不同文案。
#[derive(Debug, thiserror::Error)]
pub enum PlanError {
    /// 问卷未答全
    #[error("请先完成问卷")]
    ProfileIncomplete,
    /// 请求了不存在的模式
    #[error("模式不存在: {0}")]
    UnknownMode(String),
    /// 引擎拒绝求解
    #[error(transparent)]
    Engine(#[from] engine::EngineError),
    /// 落库失败
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// 生成结果:方案行 + 桶明细。
#[derive(Debug, Clone)]
pub struct GeneratedPlan {
    /// 方案行(含快照与版本号)
    pub plan: PlanRecord,
    /// 桶明细
    pub buckets: Vec<BucketRow>,
}

/// 生成一份方案并落库。已存在 active 方案时,旧版本会被置为非 active 并保留。
pub async fn generate(
    pool: &PgPool,
    library: &ModeLibrary,
    user_id: Uuid,
    l1_mode_id: &str,
    profile: &Profile,
) -> Result<GeneratedPlan, PlanError> {
    let mode = library
        .mode(l1_mode_id)
        .ok_or_else(|| PlanError::UnknownMode(l1_mode_id.to_string()))?;

    let result = engine::solve(profile, mode, library)?;
    let buckets = to_bucket_rows(&result);

    // 快照:引擎输出整体冻结。读回时前端直接渲染,不重算 ——
    // 重算会把「当时算出来的表」变成「按今天的配置重算的表」,那就不是历史了。
    let profile_snapshot = serde_json::to_value(profile).unwrap_or(Json::Null);
    let l2_allocation = serde_json::to_value(&result.l2).unwrap_or(Json::Null);
    let emergency = serde_json::to_value(&result.emergency).unwrap_or(Json::Null);
    let notices = serde_json::to_value(&result.notices).unwrap_or(Json::Null);
    let traces = serde_json::to_value(&result.traces).unwrap_or(Json::Null);

    let plan_id = plans::insert_plan(
        pool,
        &NewPlan {
            user_id,
            l1_mode: &result.mode_id,
            l2_mode: &result.l2.id,
            profile_snapshot: &profile_snapshot,
            l2_allocation: &l2_allocation,
            emergency: &emergency,
            notices: &notices,
            traces: &traces,
            buckets: &buckets,
        },
    )
    .await?;

    let plan = plans::active_for_user(pool, user_id)
        .await?
        .ok_or_else(|| PlanError::Db(sqlx::Error::RowNotFound))?;
    debug_assert_eq!(plan.id, plan_id);

    Ok(GeneratedPlan { plan, buckets })
}

/// 引擎输出 → 落库行。纯函数,顺序即展示顺序。
fn to_bucket_rows(result: &PlanResult) -> Vec<BucketRow> {
    result
        .buckets
        .iter()
        .map(|b| BucketRow {
            bucket_id: b.id.clone(),
            name: b.name.clone(),
            purpose: b.purpose.clone(),
            amount_monthly_cents: b.amount_monthly_cents,
            target_cents: b.target_cents,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::profile::{DrawdownResponse, Goal, Horizon, IncomeStability};

    fn lib() -> ModeLibrary {
        ModeLibrary::load_embedded().unwrap()
    }

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

    #[test]
    fn 桶行与引擎输出逐项一致且保序() {
        let m = lib();
        let result = engine::solve(&profile_a(), m.mode("four_accounts").unwrap(), &m).unwrap();
        let rows = to_bucket_rows(&result);

        assert_eq!(rows.len(), 4);
        // 顺序即展示顺序:工资 → 消费 → 备用 → 投资
        let ids: Vec<&str> = rows.iter().map(|r| r.bucket_id.as_str()).collect();
        assert_eq!(ids, vec!["salary", "spend", "reserve", "invest"]);
        // 金例 A 的四个金额(AC:与引擎输出逐项一致)
        let amounts: Vec<i64> = rows.iter().map(|r| r.amount_monthly_cents).collect();
        assert_eq!(amounts, vec![450_000, 360_000, 310_000, 80_000]);
        // 只有规则桶带目标金额
        assert_eq!(rows[2].target_cents, Some(9_720_000));
        assert_eq!(rows[0].target_cents, None);
    }

    #[test]
    fn 收入不足时落库的桶不含负数() {
        let m = lib();
        let mut p = profile_a();
        p.inflow_cents = 800_000;
        p.expense_fixed_monthly_cents = 700_000;
        let result = engine::solve(&p, m.mode("four_accounts").unwrap(), &m).unwrap();
        let rows = to_bucket_rows(&result);
        assert!(rows.iter().all(|r| r.amount_monthly_cents >= 0));
    }

    // `connect_lazy` 需要 Tokio 运行期上下文(池的维护任务在此登记),故用 `#[tokio::test]`
    // 而非 `#[test]` + 手搓 Runtime —— 后者会在建池时因「this functionality requires a
    // Tokio context」直接 panic,把一条断言变成一条噪声。
    #[tokio::test]
    async fn 不存在的模式报错而非默认一个() {
        let m = lib();
        // 用一个必然连不上的池:未知模式的判断在取连接之前发生
        let pool = sqlx::PgPool::connect_lazy("postgres://u:p@127.0.0.1:1/none").unwrap();
        let err = generate(&pool, &m, Uuid::now_v7(), "no_such_mode", &profile_a())
            .await
            .unwrap_err();
        assert!(matches!(err, PlanError::UnknownMode(_)), "实际: {err}");
    }
}
