//! 埋点上报 DTO。
//!
//! 结构刻意扁平且字段全可空,由 `analytics_service::Event::from_client` 做白名单判定 ——
//! 「哪种事件必须带哪个字段」这条规则集中在 service 一处,而不是拆到多个枚举变体上,
//! 这样被拒时能给出「哪一项不对」的原话,而不是 serde 的一句反序列化失败。

use serde::Deserialize;
use utoipa::ToSchema;

/// 客户端上报一条埋点事件(仅页面触达与问卷开始;其余三类后端自己记)。
#[derive(Debug, Deserialize, ToSchema)]
pub struct ClientEventRequest {
    /// 事件名:`page_view` / `questionnaire_start`
    pub event: String,
    /// 页面 id:`p01` / `p03` / `p04`(page_view 必填,其余不得携带)
    pub page_id: Option<String>,
}

/// 上报回执。与 A 期 `logout` 的 `OkBody` 同一形状(基线 §6.1 统一信封:
/// 成功也要有 data),字段名如实描述**请求被受理**,不声称「已入库」——
/// 写失败在服务端就被吞掉(RULE-019),这里说 recorded 会是假话。
#[derive(Debug, serde::Serialize, ToSchema)]
pub struct EventAck {
    /// 固定 true:请求合法且已交给埋点服务
    pub accepted: bool,
}
