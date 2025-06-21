use crate::common::response::ApiResponse;
use crate::features::role::model::Role;
use axum::{Json, Router, routing::get};

pub fn router() -> Router {
    Router::new().route("/", get(get_role_list))
}

async fn get_role_list() -> Json<ApiResponse<Vec<Role>>> {
    let mock_roles = vec![
        Role {
            id: 1,
            role_name: "Administrator".to_string(),
            role_code: "admin".to_string(),
            remark: Some("System administrator with all permissions".to_string()),
        },
        Role {
            id: 2,
            role_name: "Editor".to_string(),
            role_code: "editor".to_string(),
            remark: Some("Content editor".to_string()),
        },
        Role {
            id: 3,
            role_name: "Guest".to_string(),
            role_code: "guest".to_string(),
            remark: None,
        },
    ];
    ApiResponse::success(mock_roles)
}
