use super::model::DictItem;
use crate::common::api::ApiResponse;
use axum::{Json, Router, routing::get};
use sqlx::PgPool;

pub fn dict_routes() -> Router<PgPool> {
    Router::new().route("/", get(get_dict_list))
}

async fn get_dict_list() -> Json<ApiResponse<Vec<DictItem>>> {
    let mock_dicts = vec![
        DictItem {
            id: 1,
            dict_type: "user_status".to_string(),
            label: "Active".to_string(),
            value: "1".to_string(),
            is_default: true,
        },
        DictItem {
            id: 2,
            dict_type: "user_status".to_string(),
            label: "Inactive".to_string(),
            value: "0".to_string(),
            is_default: false,
        },
        DictItem {
            id: 3,
            dict_type: "gender".to_string(),
            label: "Male".to_string(),
            value: "M".to_string(),
            is_default: false,
        },
        DictItem {
            id: 4,
            dict_type: "gender".to_string(),
            label: "Female".to_string(),
            value: "F".to_string(),
            is_default: false,
        },
    ];
    ApiResponse::success(mock_dicts)
}
