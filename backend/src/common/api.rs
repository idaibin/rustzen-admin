use crate::common::error::AppError;
use axum::Json;
use serde::{Deserialize, Serialize};

// --- API Response Structures ---

/// A unified structure for successful API responses.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    /// Business status code. 0 for success.
    pub code: i32,
    /// Response message.
    pub message: String,
    /// Response data.
    pub data: T,
}

impl<T: Serialize> ApiResponse<T> {
    /// Creates a success response.
    pub fn success(data: T) -> Json<Self> {
        Json(Self { code: 0, message: "Success".to_string(), data })
    }
}

/// A generic structure for dropdown options.
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionItem<T> {
    pub label: String,
    pub value: T,
}

/// Query parameters for options endpoints
#[derive(Debug, Deserialize)]
pub struct OptionsQuery {
    pub status: Option<String>,
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

// --- API Result Type ---

/// A type alias for application-level results in API handlers.
pub type AppResult<T> = Result<T, AppError>;
