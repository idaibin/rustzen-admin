use super::support::{TestDatabase, insert_notification};
use crate::features::notifications::notification_routes;
use axum::{Extension, body::to_bytes, http::Request};
use rustzen_auth::auth::CurrentUser;
use tower::ServiceExt;

#[test]
fn selected_api_owner_registers_exactly_six_authenticated_routes() {
    let (_, contracts) = notification_routes().into_parts();
    assert_eq!(contracts.len(), 6);
    assert!(contracts.iter().all(|contract| matches!(
        contract.access,
        crate::infra::contract::RegisteredAccess::Authenticated
    )));
    let inventory = contracts
        .iter()
        .map(|contract| (contract.method.as_str(), contract.path.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        inventory,
        [
            ("GET", "/"),
            ("GET", "/unread-count"),
            ("GET", "/{id}"),
            ("PUT", "/{id}/read"),
            ("POST", "/read-all"),
            ("GET", "/stream"),
        ]
    );
    let mut source_routes = contracts
        .iter()
        .map(|contract| {
            let path = if contract.path == "/" {
                "/api/notifications".to_owned()
            } else {
                format!("/api/notifications{}", contract.path)
            };
            serde_json::json!({
                "access": {"kind": "authenticated"},
                "method": contract.method.as_str(),
                "operation": contract.operation.operation_id(),
                "path": path,
            })
        })
        .collect::<Vec<_>>();
    source_routes.sort_by(|left, right| {
        left["method"]
            .as_str()
            .cmp(&right["method"].as_str())
            .then(left["path"].as_str().cmp(&right["path"].as_str()))
    });
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../distribution/fixtures/monitor-notify-api-owner.json"
    ))
    .unwrap();
    assert_eq!(fixture["version"], 1);
    assert_eq!(fixture["routes"], serde_json::Value::Array(source_routes));
}
#[tokio::test]
async fn authenticated_http_detail_hides_another_users_recipient() {
    let database = TestDatabase::new().await;
    insert_notification(&database.primary, "private", 1).await;
    let (router, _) = notification_routes().into_parts();
    let app = router
        .layer(Extension(CurrentUser::new(2, "admin", ["*".to_owned()], true)))
        .with_state(database.primary.clone());
    let response = app
        .oneshot(Request::get("/private").body(axum::body::Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    let body = to_bytes(response.into_body(), 4096).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
        serde_json::json!({"code":10001,"message":"Notification not found.","data":null})
    );
    database.close().await;
}

#[tokio::test]
async fn http_list_rejects_invalid_query_state_with_json_and_hides_internal_sequence() {
    let database = TestDatabase::new().await;
    insert_notification(&database.primary, "wire-item", 1).await;
    let (router, _) = notification_routes().into_parts();
    let app = router
        .layer(Extension(CurrentUser::new(1, "owner", ["*".to_owned()], true)))
        .with_state(database.primary.clone());

    for (uri, message) in [
        ("/?limit=wrong", "Invalid inbox query parameters"),
        ("/?limit=0", "Inbox limit must be between 1 and 100"),
        ("/?limit=101", "Inbox limit must be between 1 and 100"),
        ("/?cursor=visible-json", "Invalid inbox cursor or snapshot"),
    ] {
        let response = app
            .clone()
            .oneshot(Request::get(uri).body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(
            response.headers()[axum::http::header::CONTENT_TYPE],
            "application/json",
            "{uri}"
        );
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({"code":10002,"message":message,"data":null}),
            "{uri}"
        );
    }

    let response =
        app.oneshot(Request::get("/").body(axum::body::Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = to_bytes(response.into_body(), 16 * 1024).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["data"]["items"][0]["id"], "wire-item");
    assert!(value["data"]["items"][0].get("inboxSeq").is_none());
    database.close().await;
}
