//! DTO 层:serde 请求/响应模型 + validator 字段校验(基线 §4.5)。
//!
//! 字段校验(长度/范围/格式)在本层;业务校验(口令对不对、缺口够不够)在 service 层。

pub mod analytics;
pub mod auth;
pub mod mode;
pub mod plan;
pub mod profile;
pub mod snapshot;
