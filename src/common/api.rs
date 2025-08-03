use crate::common::error::AppError;
// use axum::response::IntoResponse;

use axum::Json;
use serde::{Deserialize, Serialize};

// --- API Response Structures ---
/// A unified structure for successful API responses.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// Business status code. 0 for success.
    pub code: i32,
    /// Response message.
    pub message: String,
    /// Response data.
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Total number of items.
    pub total: Option<i64>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Creates a success response.
    pub fn success(data: T) -> Json<Self> {
        Json(Self { code: 0, message: "Success".to_string(), data, total: None })
    }
}

// 为 Vec 类型提供特殊实现
impl<T: Serialize> ApiResponse<Vec<T>> {
    pub fn page(data: Vec<T>, total: i64) -> Json<Self> {
        Json(Self { code: 0, message: "Success".to_string(), data, total: Some(total) })
    }
}

// --- API Result Type ---
/// A type alias for application-level results in API handlers.
pub type AppResult<T> = Result<Json<ApiResponse<T>>, AppError>;

// impl<T: Serialize> IntoResponse for AppResult<T> {
//     fn into_response(self) -> axum::response::Response {
//         match self {
//             Ok(json_response) => json_response.into_response(),
//             Err(error) => error.into_response(),
//         }
//     }
// }

/// A generic structure for dropdown options.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OptionItem<T> {
    pub label: String,
    pub value: T,
}

/// Query parameters for options endpoints
#[derive(Debug, Deserialize)]
pub struct OptionsQuery {
    pub q: Option<String>,
    pub limit: Option<i64>,
}

/// Query parameters for dict options endpoints
#[derive(Debug, Deserialize)]
pub struct DictOptionsQuery {
    pub dict_type: Option<String>,
    pub q: Option<String>,
    pub limit: Option<i64>,
}
