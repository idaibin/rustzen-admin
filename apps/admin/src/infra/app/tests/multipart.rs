use super::*;

fn multipart_body(boundary: &str, fields: &[(&str, Option<&str>, &str, &str)]) -> Body {
    let mut body = String::new();
    for (name, filename, content_type, value) in fields {
        body.push_str(&format!("--{boundary}\r\n"));
        body.push_str(&format!("Content-Disposition: form-data; name=\"{name}\""));
        if let Some(filename) = filename {
            body.push_str(&format!("; filename=\"{filename}\""));
        }
        body.push_str("\r\n");
        if !content_type.is_empty() {
            body.push_str(&format!("Content-Type: {content_type}\r\n"));
        }
        body.push_str(&format!("\r\n{value}\r\n"));
    }
    body.push_str(&format!("--{boundary}--\r\n"));
    Body::from(body)
}

#[tokio::test]
async fn multipart_contract_routes_accept_generated_file_inputs() {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
    run_migrations(&pool).await.expect("migrations");
    activate_seed_owner(&pool).await;
    let (routes, _) = documented_protected_routes();
    let codec = JwtCodec::new("contract-test", 60);
    let app = Router::new()
        .merge(routes)
        .layer(Extension(std::sync::Arc::new(DeployService::new(pool.clone()))))
        .route_layer(middleware::from_fn_with_state((codec.clone(), TestLoader), auth_middleware))
        .with_state(pool.clone());
    let owner = session_token(&pool, &codec, 1, "owner").await;

    let boundary = "avatar-boundary";
    let mut png = std::io::Cursor::new(Vec::new());
    image::RgbImage::new(2, 2).write_to(&mut png, image::ImageFormat::Png).unwrap();
    let mut avatar_body = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"avatar.png\"\r\nContent-Type: image/png\r\n\r\n").into_bytes();
    avatar_body.extend(png.into_inner());
    avatar_body.extend(format!("\r\n--{boundary}--\r\n").into_bytes());
    let avatar_response = app
        .clone()
        .oneshot(
            Request::post("/api/account/avatar")
                .header("authorization", format!("Bearer {owner}"))
                .header("content-type", format!("multipart/form-data; boundary={boundary}"))
                .body(Body::from(avatar_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(avatar_response.status(), StatusCode::OK);
    let avatar_payload: serde_json::Value =
        serde_json::from_slice(&to_bytes(avatar_response.into_body(), usize::MAX).await.unwrap())
            .expect("avatar response");
    let avatar_url = avatar_payload["data"].as_str().expect("avatar URL");
    assert!(avatar_url.ends_with(".png"));
    crate::common::files::remove_avatar_by_url(avatar_url).await.expect("avatar cleanup");

    let boundary = "avatar-extension-spoof";
    let mut png = std::io::Cursor::new(Vec::new());
    image::RgbImage::new(2, 2).write_to(&mut png, image::ImageFormat::Png).unwrap();
    let mut avatar_body = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"avatar.svg\"\r\nContent-Type: image/svg+xml\r\n\r\n").into_bytes();
    avatar_body.extend(png.into_inner());
    avatar_body.extend(format!("\r\n--{boundary}--\r\n").into_bytes());
    let spoofed_extension_response = app
        .clone()
        .oneshot(
            Request::post("/api/account/avatar")
                .header("authorization", format!("Bearer {owner}"))
                .header("content-type", format!("multipart/form-data; boundary={boundary}"))
                .body(Body::from(avatar_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(spoofed_extension_response.status(), StatusCode::OK);
    let avatar_payload: serde_json::Value = serde_json::from_slice(
        &to_bytes(spoofed_extension_response.into_body(), usize::MAX).await.unwrap(),
    )
    .expect("avatar response");
    let avatar_url = avatar_payload["data"].as_str().expect("avatar URL");
    assert!(avatar_url.ends_with(".png"));
    crate::common::files::remove_avatar_by_url(avatar_url).await.expect("avatar cleanup");

    let svg_disguised_as_png = app
        .clone()
        .oneshot(
            Request::post("/api/account/avatar")
                .header("authorization", format!("Bearer {owner}"))
                .header("content-type", "multipart/form-data; boundary=avatar-svg")
                .body(multipart_body(
                    "avatar-svg",
                    &[(
                        "file",
                        Some("avatar.png"),
                        "image/png",
                        "<svg xmlns='http://www.w3.org/2000/svg'/>",
                    )],
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_json_error(svg_disguised_as_png, StatusCode::BAD_REQUEST, 10002).await;

    let application_oversized_avatar = "x".repeat(1024 * 1024 + 1);
    let application_oversized_response = app
        .clone()
        .oneshot(
            Request::post("/api/account/avatar")
                .header("authorization", format!("Bearer {owner}"))
                .header(
                    "content-type",
                    "multipart/form-data; boundary=application-oversized-avatar",
                )
                .body(multipart_body(
                    "application-oversized-avatar",
                    &[("file", Some("avatar.png"), "image/png", &application_oversized_avatar)],
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_json_error(application_oversized_response, StatusCode::PAYLOAD_TOO_LARGE, 10013).await;

    let oversized_avatar = "x".repeat(3 * 1024 * 1024 + 1024);
    let oversized_response = app
        .clone()
        .oneshot(
            Request::post("/api/account/avatar")
                .header("authorization", format!("Bearer {owner}"))
                .header("content-type", "multipart/form-data; boundary=oversized-avatar")
                .body(multipart_body(
                    "oversized-avatar",
                    &[("file", Some("avatar.png"), "image/png", &oversized_avatar)],
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_json_error(oversized_response, StatusCode::PAYLOAD_TOO_LARGE, 10013).await;

    let invalid_deployment_id = app
        .clone()
        .oneshot(
            Request::get("/api/manage/deploy/not-a-number")
                .header("authorization", format!("Bearer {owner}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_deployment_id.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        invalid_deployment_id.headers().get("content-type").unwrap(),
        "text/plain; charset=utf-8"
    );
    assert!(!to_bytes(invalid_deployment_id.into_body(), usize::MAX).await.unwrap().is_empty());

    let deployment_boundary = "deployment-boundary";
    let deployment_response = app
        .oneshot(
            Request::post("/api/manage/deploy/upload")
                .header("authorization", format!("Bearer {owner}"))
                .header(
                    "content-type",
                    format!("multipart/form-data; boundary={deployment_boundary}"),
                )
                .body(multipart_body(
                    deployment_boundary,
                    &[
                        ("component", None, "text/plain", "release"),
                        ("version", None, "text/plain", "0.5.0"),
                        ("file", Some("release.tar"), "application/octet-stream", "not-a-bundle"),
                    ],
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(deployment_response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn export_logs_route_returns_csv_content_type_and_body() {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
    run_migrations(&pool).await.expect("migrations");
    let (routes, _) = documented_protected_routes();
    let codec = JwtCodec::new("contract-test", 60);
    let app = Router::new()
        .merge(routes)
        .route_layer(middleware::from_fn_with_state((codec.clone(), TestLoader), auth_middleware))
        .with_state(pool);
    let owner = codec.encode(1, "owner").expect("token");
    let invalid_query = app
        .clone()
        .oneshot(
            Request::get("/api/manage/logs/export?current=invalid")
                .header("authorization", format!("Bearer {owner}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(invalid_query.status(), StatusCode::BAD_REQUEST);
    assert_eq!(invalid_query.headers().get("content-type").unwrap(), "text/plain; charset=utf-8");
    assert!(!to_bytes(invalid_query.into_body(), usize::MAX).await.unwrap().is_empty());
    let response = app
        .oneshot(
            Request::get("/api/manage/logs/export")
                .header("authorization", format!("Bearer {owner}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/csv; charset=utf-8");
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(
        body,
        "ID,user_id,username,action,description,status,duration_ms,ip_address,user_agent,created_at\n"
    );
}

#[test]
fn admin_native_contract_inventory_has_no_duplicate_operations_or_routes() {
    let contracts = documented_all_contracts();
    assert_eq!(contracts.len(), 54);

    let mut operations = std::collections::BTreeSet::new();
    let mut routes = std::collections::BTreeSet::new();
    for contract in contracts {
        assert!(operations.insert(contract.operation.operation_id()));
        assert!(routes.insert((contract.method, contract.path)));
    }
    assert_eq!(operations.len(), 54);
    assert_eq!(routes.len(), 54);
}
