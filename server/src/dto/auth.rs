//! 鉴权相关 DTO。

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// 登录请求。长度上限同时是防滥用手段(基线 §4.5-3:字符串必须有长度上限)。
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// 用户名
    #[validate(length(min = 1, max = 64, message = "请输入用户名"))]
    pub username: String,
    /// 口令(不落日志、不回显)
    #[validate(length(min = 1, max = 128, message = "请输入密码"))]
    pub password: String,
}

/// 会话信息。**不含任何凭证** —— 只回一个用户名给前端渲染。
#[derive(Debug, Serialize, ToSchema)]
pub struct SessionInfo {
    /// 当前登录的用户名
    pub username: String,
}

/// 无数据成功的响应体(退出登录等)。
#[derive(Debug, Serialize, ToSchema)]
pub struct OkBody {
    /// 恒为 true;有信封在,这里只是给序列化一个具体类型
    pub ok: bool,
}
