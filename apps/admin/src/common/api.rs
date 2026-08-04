use crate::common::error::AppError;

use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Json<Self> {
        Json(Self::new(data, None))
    }

    pub fn new(data: T, total: Option<i64>) -> Self {
        Self { code: 0, message: "Success".to_string(), data, total }
    }
}

impl<T: Serialize> ApiResponse<Vec<T>> {
    pub fn page(data: Vec<T>, total: i64) -> Json<Self> {
        Json(Self::new(data, Some(total)))
    }
}

pub type AppResult<T> = Result<Json<ApiResponse<T>>, AppError>;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct OptionItem<T> {
    pub label: String,
    pub value: T,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub struct OptionsQuery {
    pub q: Option<String>,
    pub limit: Option<i64>,
}
