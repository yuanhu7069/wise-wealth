//! 埋点 service(RULE-019 / ADR-B-005):5 类事件入库,写入失败不阻断主流程。
//!
//! **金额永不入埋点**这条红线不靠人记得,靠类型:事件只能是 [`Event`] 的变体,
//! 每个变体自带的字段只有页面 id、步号、模式 id、版本号 —— 没有能塞进金额的位置。
//! 调用点因此不可能「顺手多带一个 amount」,review 也不必逐条核对 payload。
//!
//! 触发位置(基线 §11.3:后端 / Server Action,禁止散落客户端各处):
//! - 后端自己知道的三件事 —— 每步保存成功、问卷答全、方案生成成功 —— 由 handler 直接记;
//! - 只有页面知道的两件事 —— 页面触达、问卷开始 —— 由前端 Server 端经
//!   `POST /api/v1/analytics/events` 交给本模块,**白名单只放行这两个**
//!   (见 [`Event::from_client`]),否则同一个事件会有两个来源,迟早重复计数。

use serde_json::{json, Value as Json};
use sqlx::PgPool;

use crate::repos;

/// 事件名(wire 字符串,与 prd-v1 §9.5 表一致)。
pub const PAGE_VIEW: &str = "page_view";
pub const QUESTIONNAIRE_START: &str = "questionnaire_start";
pub const QUESTIONNAIRE_STEP_COMPLETED: &str = "questionnaire_step_completed";
pub const QUESTIONNAIRE_COMPLETED: &str = "questionnaire_completed";
pub const PLAN_GENERATED: &str = "plan_generated";

/// 页面 id(埋点只认这三张页面,白名单而非自由字符串)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    /// P01 产品首页
    P01,
    /// P03 引导问卷
    P03,
    /// P04 方案页
    P04,
}

impl PageId {
    pub const fn as_str(self) -> &'static str {
        match self {
            PageId::P01 => "p01",
            PageId::P03 => "p03",
            PageId::P04 => "p04",
        }
    }

    /// wire 字符串 → 页面 id。**不做大小写与空白容错**:写入端是自家前端,
    /// 宽松解析只会把「前端传错了」变成「埋点里多了一条来路不明的 p01 」。
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "p01" => Some(PageId::P01),
            "p03" => Some(PageId::P03),
            "p04" => Some(PageId::P04),
            _ => None,
        }
    }
}

/// 一条待记录的事件。变体即白名单,字段即 payload 的全部可能内容。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// P01/P03/P04 服务端渲染时触达
    PageView { page_id: PageId },
    /// P03 首次进入(尚无草稿)
    QuestionnaireStart,
    /// 问卷某一步保存成功(步 1-5;步 6 是推荐位,不落库故无此事件)
    QuestionnaireStepCompleted { step: i16 },
    /// 问卷答全(步 5 保存后由「未完成 → 完成」的那一刻触发)
    QuestionnaireCompleted,
    /// 方案生成成功
    PlanGenerated { l1_mode: String, plan_version: i32 },
}

impl Event {
    /// 事件名(落库的 `event_type`)。
    pub const fn name(&self) -> &'static str {
        match self {
            Event::PageView { .. } => PAGE_VIEW,
            Event::QuestionnaireStart => QUESTIONNAIRE_START,
            Event::QuestionnaireStepCompleted { .. } => QUESTIONNAIRE_STEP_COMPLETED,
            Event::QuestionnaireCompleted => QUESTIONNAIRE_COMPLETED,
            Event::PlanGenerated { .. } => PLAN_GENERATED,
        }
    }

    /// 事件载荷:只有枚举与步号(金额永不入埋点)。
    pub fn payload(&self) -> Json {
        match self {
            Event::PageView { page_id } => json!({ "page_id": page_id.as_str() }),
            Event::QuestionnaireStart => json!({}),
            Event::QuestionnaireStepCompleted { step } => json!({ "step": step }),
            Event::QuestionnaireCompleted => json!({}),
            Event::PlanGenerated {
                l1_mode,
                plan_version,
            } => json!({ "l1_mode": l1_mode, "plan_version": plan_version }),
        }
    }
}

/// 客户端上报被拒的原因。文案说清楚哪一项不对。
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EventError {
    /// 事件名不在客户端可上报的白名单里
    #[error("不支持的埋点事件:{0}")]
    UnknownEvent(String),
    /// page_view 缺 page_id
    #[error("page_view 需要携带 page_id")]
    MissingPageId,
    /// 页面 id 不在白名单里
    #[error("未知页面 id:{0}")]
    UnknownPageId(String),
    /// 不是 page_view 却带了 page_id
    #[error("{0} 不携带 page_id")]
    PageIdNotAllowed(String),
}

impl Event {
    /// 客户端上报的两种事件 → [`Event`]。
    ///
    /// 白名单只有 `page_view` 与 `questionnaire_start`:其余三类事件后端自己就知道,
    /// 放行它们等于给同一个事实开两个来源(前端多报一次,完成率就超过 100%)。
    pub fn from_client(name: &str, page_id: Option<&str>) -> Result<Event, EventError> {
        match name {
            PAGE_VIEW => {
                let raw = page_id.ok_or(EventError::MissingPageId)?;
                let page_id =
                    PageId::parse(raw).ok_or_else(|| EventError::UnknownPageId(raw.to_string()))?;
                Ok(Event::PageView { page_id })
            }
            QUESTIONNAIRE_START => match page_id {
                Some(_) => Err(EventError::PageIdNotAllowed(name.to_string())),
                None => Ok(Event::QuestionnaireStart),
            },
            other => Err(EventError::UnknownEvent(other.to_string())),
        }
    }
}

/// 记录一条事件。**失败只告警,不返回错误**(RULE-019):埋点是旁路观测,
/// 它坏了不该让用户存不下问卷、生成不了方案。返回 `()` 是刻意的 ——
/// 调用方拿不到错误,也就不可能「顺手」把它变成主流程的失败。
pub async fn record(pool: &PgPool, event: &Event) {
    // 「问卷开始」是一次性的(prd-v1 §9.5 说的是「**首次**进入」):前端每次
    // 「无草稿进入」都会上报,而刷新一次页面就是又一次「无草稿进入」—— 照单全收的话,
    // 开始率会被刷新刷到 100% 以上。已经记过就不再记。
    // 为什么按「表里有没有」判断:草稿一旦存在(draft_step ≥ 2)就不会再消失,
    // 所以「首次」在当前数据模型下等价于「表里还没有第一条」。
    if matches!(event, Event::QuestionnaireStart)
        && matches!(
            repos::analytics::count_of_type(pool, QUESTIONNAIRE_START).await,
            Ok(n) if n > 0
        )
    {
        return;
    }

    if let Err(e) = repos::analytics::insert(pool, event.name(), &event.payload()).await {
        tracing::warn!(
            event = event.name(),
            "埋点写入失败(不影响主流程): {e}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn all_events() -> Vec<Event> {
        vec![
            Event::PageView {
                page_id: PageId::P01,
            },
            Event::PageView {
                page_id: PageId::P03,
            },
            Event::PageView {
                page_id: PageId::P04,
            },
            Event::QuestionnaireStart,
            Event::QuestionnaireStepCompleted { step: 1 },
            Event::QuestionnaireCompleted,
            Event::PlanGenerated {
                l1_mode: "four_accounts".into(),
                plan_version: 2,
            },
        ]
    }

    #[test]
    fn 事件名与载荷逐条锁定() {
        // 载荷形态是埋点的对外契约(查验 SQL 直接读 payload->>'step' 这类键),
        // 全量断言而不是抽查:改了形状就得改这条测试,不会悄悄漂移。
        let cases = [
            (
                Event::PageView {
                    page_id: PageId::P03,
                },
                "page_view",
                json!({"page_id": "p03"}),
            ),
            (
                Event::QuestionnaireStart,
                "questionnaire_start",
                json!({}),
            ),
            (
                Event::QuestionnaireStepCompleted { step: 4 },
                "questionnaire_step_completed",
                json!({"step": 4}),
            ),
            (
                Event::QuestionnaireCompleted,
                "questionnaire_completed",
                json!({}),
            ),
            (
                Event::PlanGenerated {
                    l1_mode: "fifty_30_20".into(),
                    plan_version: 3,
                },
                "plan_generated",
                json!({"l1_mode": "fifty_30_20", "plan_version": 3}),
            ),
        ];
        for (event, name, payload) in cases {
            assert_eq!(event.name(), name);
            assert_eq!(event.payload(), payload, "载荷不符: {name}");
        }
    }

    #[test]
    fn 载荷键只有白名单里的那几个() {
        // 金额永不入埋点(arch §8):把「能出现的键」钉死在这几个上 ——
        // 将来有人给某个变体加字段,这条测试会逼他先回答「这是不是敏感数值」。
        const ALLOWED: [&str; 3] = ["page_id", "step", "l1_mode"];
        for event in all_events() {
            let Some(obj) = event.payload().as_object().cloned() else {
                panic!("载荷必须是对象");
            };
            for (key, value) in obj {
                // plan_version 是版本号(小整数),不在键白名单里但同样不是金额
                if key != "plan_version" {
                    assert!(ALLOWED.contains(&key.as_str()), "载荷出现计划外的键: {key}");
                }
                if let Some(n) = value.as_i64() {
                    assert!(n < 1000, "载荷里的数值只该是步号/版本号,收到 {n}");
                }
            }
        }
    }

    #[test]
    fn 客户端只能上报页面触达与问卷开始() {
        assert_eq!(
            Event::from_client("page_view", Some("p01")).unwrap(),
            Event::PageView {
                page_id: PageId::P01
            }
        );
        assert_eq!(
            Event::from_client("questionnaire_start", None).unwrap(),
            Event::QuestionnaireStart
        );
    }

    #[test]
    fn 客户端上报被拒的四种情形() {
        // 后端自己会记的三类事件不给客户端上报(否则重复计数)
        for name in [
            "questionnaire_step_completed",
            "questionnaire_completed",
            "plan_generated",
        ] {
            assert_eq!(
                Event::from_client(name, None).unwrap_err(),
                EventError::UnknownEvent(name.to_string())
            );
        }
        assert_eq!(
            Event::from_client("page_view", None).unwrap_err(),
            EventError::MissingPageId
        );
        assert_eq!(
            Event::from_client("page_view", Some("p99")).unwrap_err(),
            EventError::UnknownPageId("p99".into())
        );
        assert_eq!(
            Event::from_client("questionnaire_start", Some("p03")).unwrap_err(),
            EventError::PageIdNotAllowed("questionnaire_start".into())
        );
    }

    // 与 plan_service 的同类测试一样用 `#[tokio::test]`:建池需要 Tokio 上下文。
    #[tokio::test]
    async fn 写入失败不阻断主流程() {
        // 指向一个必然连不上的地址:record 必须**正常返回**,而不是把错误抛给调用方。
        let pool = sqlx::postgres::PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(200))
            .connect_lazy("postgres://u:p@127.0.0.1:1/none")
            .unwrap();
        record(&pool, &Event::QuestionnaireCompleted).await;
        record(&pool, &Event::QuestionnaireStart).await;
    }
}
