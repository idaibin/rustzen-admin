use crate::common::response::ApiResponse;
use crate::features::menu::model::Menu;
use axum::{Json, Router, routing::get};

pub fn router() -> Router {
    Router::new().route("/", get(get_menu_list))
}

async fn get_menu_list() -> Json<ApiResponse<Vec<Menu>>> {
    let mock_menus = vec![
        Menu {
            id: 1,
            parent_id: 0,
            name: "System Management".to_string(),
            path: Some("/system".to_string()),
            component: Some("BasicLayout".to_string()),
            icon: Some("setting".to_string()),
            r#type: 0,
        },
        Menu {
            id: 2,
            parent_id: 1,
            name: "User Management".to_string(),
            path: Some("user".to_string()),
            component: Some("system/user/index".to_string()),
            icon: Some("user".to_string()),
            r#type: 1,
        },
        Menu {
            id: 3,
            parent_id: 1,
            name: "Role Management".to_string(),
            path: Some("role".to_string()),
            component: Some("system/role/index".to_string()),
            icon: Some("team".to_string()),
            r#type: 1,
        },
        Menu {
            id: 4,
            parent_id: 2,
            name: "Add User".to_string(),
            path: None,
            component: None,
            icon: None,
            r#type: 2,
        },
    ];
    ApiResponse::success(mock_menus)
}
