use super::*;

#[tokio::test]
async fn documented_nested_routes_match_their_final_public_paths() {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
    let (routes, contracts) = documented_protected_routes();
    assert!(
        contracts
            .iter()
            .any(|contract| contract.path == "/api/auth/me")
    );
    assert!(
        contracts
            .iter()
            .any(|contract| contract.path == "/api/system/users")
    );
    let codec = JwtCodec::new("contract-test", 60);
    let app = Router::new()
        .merge(routes)
        .route_layer(middleware::from_fn_with_state(
            (codec.clone(), TestLoader),
            auth_middleware,
        ))
        .with_state(pool);

    let auth_response = app
        .clone()
        .oneshot(Request::get("/api/auth/me").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_json_error(auth_response, StatusCode::UNAUTHORIZED, 401).await;

    let viewer = codec.encode(2, "viewer").expect("token");
    let user_response = app
        .clone()
        .oneshot(
            Request::post("/api/system/users")
                .header("authorization", format!("Bearer {viewer}"))
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_json_error(user_response, StatusCode::FORBIDDEN, 403).await;

    let owner = codec.encode(1, "owner").expect("token");
    let missing_content_type = app
        .clone()
        .oneshot(
            Request::post("/api/system/users")
                .header("authorization", format!("Bearer {owner}"))
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        missing_content_type.status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
    assert_eq!(
        missing_content_type.headers().get("content-type").unwrap(),
        "text/plain; charset=utf-8"
    );
    assert!(
        !to_bytes(missing_content_type.into_body(), usize::MAX)
            .await
            .unwrap()
            .is_empty()
    );

    let malformed_json = app
        .clone()
        .oneshot(
            Request::post("/api/system/users")
                .header("authorization", format!("Bearer {owner}"))
                .header("content-type", "application/json")
                .body(Body::from("{"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(malformed_json.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        malformed_json.headers().get("content-type").unwrap(),
        "text/plain; charset=utf-8"
    );
    assert!(
        !to_bytes(malformed_json.into_body(), usize::MAX)
            .await
            .unwrap()
            .is_empty()
    );

    let invalid_dto = app
        .oneshot(
            Request::post("/api/system/users")
                .header("authorization", format!("Bearer {owner}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"username": 1}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_dto.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        invalid_dto.headers().get("content-type").unwrap(),
        "text/plain; charset=utf-8"
    );
    assert!(
        !to_bytes(invalid_dto.into_body(), usize::MAX)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn login_statuses_return_the_documented_json_errors() {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
    run_migrations(&pool).await.expect("migrations");
    let (routes, _) = public_auth_routes().into_parts();
    let app = routes
        .layer(Extension(ConnectInfo(
            "127.0.0.1:3000"
                .parse::<std::net::SocketAddr>()
                .expect("address"),
        )))
        .with_state(pool.clone());

    for (status, expected_status, code) in [
        (2_i16, StatusCode::FORBIDDEN, 10004),
        (3_i16, StatusCode::BAD_REQUEST, 10005),
        (4_i16, StatusCode::BAD_REQUEST, 10006),
    ] {
        sqlx::query("UPDATE users SET status = ? WHERE username = 'owner'")
            .bind(status)
            .execute(&pool)
            .await
            .expect("update seeded owner status");
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"username":"owner","password":"anything"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_json_error(response, expected_status, code).await;
    }
}

#[tokio::test]
async fn business_and_internal_errors_match_the_json_error_envelope() {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
    run_migrations(&pool).await.expect("migrations");
    let (routes, _) = documented_protected_routes();
    PermissionService::sync_permissions(&pool)
        .await
        .expect("permission cache");
    let codec = JwtCodec::new("contract-test", 60);
    let app = Router::new()
        .merge(routes)
        .route_layer(middleware::from_fn_with_state(
            (codec.clone(), TestLoader),
            auth_middleware,
        ))
        .with_state(pool);
    let owner = codec.encode(1, "owner").expect("token");
    let not_found = app
        .clone()
        .oneshot(
            Request::delete("/api/system/users/999")
                .header("authorization", format!("Bearer {owner}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_json_error(not_found, StatusCode::NOT_FOUND, 10001).await;

    let owner_status = codec.encode(1, "owner").expect("status token");
    let invalid_status = app
        .clone()
        .oneshot(
            Request::put("/api/system/users/1/status")
                .header("authorization", format!("Bearer {owner_status}"))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"status":99}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_json_error(invalid_status, StatusCode::BAD_REQUEST, 10007).await;

    assert_json_error(
        crate::common::error::AppError::from(crate::common::error::ServiceError::UsernameConflict)
            .into_response(),
        StatusCode::CONFLICT,
        10201,
    )
    .await;
    assert_json_error(
        crate::common::error::AppError::from(
            crate::common::error::ServiceError::DatabaseQueryFailed,
        )
        .into_response(),
        StatusCode::INTERNAL_SERVER_ERROR,
        20001,
    )
    .await;
}
