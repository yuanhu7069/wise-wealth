//! 统一响应信封 + AppError + ErrorCode(RULE-003,arch 基线 §6.1/§6.2)。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

/// 统一响应信封。
///
/// 成功:`{ "success": true, "data": {...}, "errorCode": null, "message": null }`
/// 失败:`{ "success": false, "data": null, "errorCode": "...", "message": "..." }`
#[derive(Debug, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[allow(non_snake_case)]
pub struct Envelope<T: Serialize + utoipa::ToSchema> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errorCode: Option<ErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize + utoipa::ToSchema> Envelope<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            errorCode: None,
            message: None,
        }
    }
}

impl Envelope<()> {
    pub fn err(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            errorCode: Some(code),
            message: Some(message.into()),
        }
    }
}

/// 集中错误码表(arch 基线 §6.2)。A 期实际可能出现的仅 INTERNAL_ERROR;
/// 其余码随各期业务引入,RULE-003 要求 errorCode 只能取自本枚举。
/// wire 格式为 SCREAMING_SNAKE(前端按字符串映射文案),故保留全大写命名。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
pub enum ErrorCode {
    VALIDATION_ERROR,
    UNAUTHORIZED,
    FORBIDDEN,
    NOT_FOUND,
    CONFLICT,
    RATE_LIMITED,
    INTERNAL_ERROR,
}

impl ErrorCode {
    /// HTTP 状态码映射(基线 §6.2)。
    pub fn status(self) -> StatusCode {
        match self {
            ErrorCode::VALIDATION_ERROR => StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::UNAUTHORIZED => StatusCode::UNAUTHORIZED,
            ErrorCode::FORBIDDEN => StatusCode::FORBIDDEN,
            ErrorCode::NOT_FOUND => StatusCode::NOT_FOUND,
            ErrorCode::CONFLICT => StatusCode::CONFLICT,
            ErrorCode::RATE_LIMITED => StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::INTERNAL_ERROR => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// 应用错误:thiserror 领域错误,统一转信封响应。
/// A 期 health 链路直接在 handler 内吸收 db 故障(不上抛),AppError 供 B 期起
/// 业务端点使用;为避免骨架期 dead_code 告警,当前仅错误转换单测消费。
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    /// 数据库访问失败(pg pool 查询/初始化故障)。内部信息不外泄,响应仅带兜底文案。
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    /// 兜底内部错误
    #[error("内部错误: {0}")]
    Internal(#[from] anyhow::Error),
}

impl From<ErrorCode> for AppError {
    fn from(code: ErrorCode) -> Self {
        AppError::Internal(anyhow::anyhow!("{code:?}"))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 兜底文案:生产环境禁止技术细节(基线 §6.1 规则 3)。
        // A 期出现的实际错误仅 INTERNAL_ERROR 一类(arch-a.md §4)。
        let code = match &self {
            AppError::Database(_) | AppError::Internal(_) => ErrorCode::INTERNAL_ERROR,
        };
        let body = Envelope::err(code, "服务器开小差了,请稍后重试");
        let status = code.status();
        (status, Json(body)).into_response()
    }
}

/// 成功信封的便捷 IntoResponse
pub struct ApiOk<T: Serialize + utoipa::ToSchema>(pub T);

impl<T: Serialize + utoipa::ToSchema> IntoResponse for ApiOk<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(Envelope::ok(self.0))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// RULE-003 / 基线 §6.1:成功信封结构与基线一致(字段名 camelCase,errorCode/message 为 null)
    #[test]
    fn success_envelope_shape() {
        let e = Envelope::ok(json!({"status": "ok"}));
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["success"], json!(true));
        assert_eq!(v["data"], json!({"status": "ok"}));
        assert_eq!(v["errorCode"], serde_json::Value::Null);
        assert_eq!(v["message"], serde_json::Value::Null);
    }

    /// RULE-003:失败信封结构
    #[test]
    fn error_envelope_shape() {
        let e = Envelope::err(ErrorCode::INTERNAL_ERROR, "服务器开小差了,请稍后重试");
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["success"], json!(false));
        assert_eq!(v["data"], serde_json::Value::Null);
        assert_eq!(v["errorCode"], json!("INTERNAL_ERROR"));
        assert_eq!(v["message"], json!("服务器开小差了,请稍后重试"));
    }

    /// 信封反序列化:errorCode 只能是集中定义的枚举值
    #[test]
    fn error_code_enum_roundtrip() {
        let v = serde_json::to_value(ErrorCode::VALIDATION_ERROR).unwrap();
        assert_eq!(v, json!("VALIDATION_ERROR"));
        let parsed: ErrorCode = serde_json::from_value(v).unwrap();
        assert_eq!(parsed, ErrorCode::VALIDATION_ERROR);
    }
}
