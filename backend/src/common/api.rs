use axum::Json;
use serde::{Deserialize, Serialize};

/// 通用 API 响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
}

/// 成功响应的快速实现
impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T) -> Json<Self> {
        Json(Self { code: 200, message: "success".to_string(), data: Some(data) })
    }

    /// 创建错误响应
    pub fn fail(code: u16, message: String) -> Json<Self> {
        Json(Self { code, message, data: None })
    }
}
