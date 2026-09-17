use super::*;

#[tokio::test]
async fn module_gateway_owns_cors_while_admin_routes_keep_wildcard_cors() {
    let upstream = Router::new().route("/api/insights/track", any(fake_insights_cors));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind fake module");
    let address = listener.local_addr().expect("fake module address");
    let server = tokio::spawn(async move {
        axum::serve(listener, upstream).await.expect("serve fake module");
    });

    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
    let registry = ModuleRegistry::new(
        vec![ModuleSpec {
            id: "insights",
            name: "Insights",
            base_url: format!("http://{address}"),
        }],
        &BTreeMap::new(),
    );
    let manifest = ModuleManifest {
        module: "insights".into(),
        name: "Insights".into(),
        api_prefix: "/api/insights".into(),
        contract_version: 1,
        release_version: env!("CARGO_PKG_VERSION").into(),
        menus: Vec::new(),
        routes: vec![
            RouteManifest {
                method: "OPTIONS".into(),
                path: "/track".into(),
                access: AccessMode::Public,
                permission: None,
            },
            RouteManifest {
                method: "POST".into(),
                path: "/track".into(),
                access: AccessMode::Public,
                permission: None,
            },
        ],
    };
    registry.replace(RegistrySnapshot::from_modules(BTreeMap::from([(
        "insights".into(),
        ModuleRuntime {
            spec: ModuleSpec {
                id: "insights",
                name: "Insights",
                base_url: format!("http://{address}"),
            },
            enabled: true,
            condition: ModuleCondition::Healthy,
            manifest: Some(Arc::new(manifest)),
            manifest_hash: Some([1; 32]),
            storage: None,
            last_seen_at: Some(chrono::Utc::now()),
            error: None,
        },
    )])));
    let module_state = ModuleControlState {
        pool,
        registry,
        client: reqwest::Client::new(),
        signer: DelegationSigner::new("cors-test-secret").expect("signer"),
        enabled_update: Arc::default(),
    };
    let admin_routes = Router::new()
        .route("/health", axum::routing::get(health))
        .fallback(crate::infra::web::serve)
        .layer(admin_cors());
    let app = Router::new().merge(gateway::routes().with_state(module_state)).merge(admin_routes);

    let allowed_options = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/api/insights/track")
                .header(header::ORIGIN, "https://app.example")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                .header(
                    header::ACCESS_CONTROL_REQUEST_HEADERS,
                    "content-type, x-rustzen-project-key",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("allowed module preflight");
    assert_eq!(allowed_options.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        allowed_options.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "https://app.example"
    );
    assert_eq!(allowed_options.headers()[header::ACCESS_CONTROL_ALLOW_METHODS], "POST");
    assert_eq!(
        allowed_options.headers()[header::ACCESS_CONTROL_ALLOW_HEADERS],
        "content-type, x-rustzen-project-key"
    );
    assert_ne!(allowed_options.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");

    let denied_options = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/api/insights/track")
                .header(header::ORIGIN, "https://not-allowed.example")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                .header(
                    header::ACCESS_CONTROL_REQUEST_HEADERS,
                    "content-type, x-rustzen-project-key",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("denied module preflight");
    assert_eq!(denied_options.status(), StatusCode::FORBIDDEN);
    assert_no_cors_allow_headers(&denied_options);

    let allowed_post = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/insights/track")
                .header(header::ORIGIN, "https://app.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("allowed module POST");
    assert_eq!(allowed_post.status(), StatusCode::OK);
    assert_eq!(allowed_post.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "https://app.example");
    assert_ne!(allowed_post.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");

    let denied_post = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/insights/track")
                .header(header::ORIGIN, "https://not-allowed.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("denied module POST");
    assert_eq!(denied_post.status(), StatusCode::FORBIDDEN);
    assert_no_cors_allow_headers(&denied_post);

    let admin_health = app
        .clone()
        .oneshot(
            Request::get("/health")
                .header(header::ORIGIN, "https://any-admin-client.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Admin CORS");
    assert_eq!(admin_health.status(), StatusCode::OK);
    assert_eq!(admin_health.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
    let admin_preflight = app
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/health")
                .header(header::ORIGIN, "https://any-admin-client.example")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Admin preflight CORS");
    assert_eq!(admin_preflight.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
    assert_eq!(
        admin_preflight.headers()[header::ACCESS_CONTROL_ALLOW_METHODS],
        "GET,POST,PUT,PATCH,DELETE"
    );
    server.abort();
}

fn assert_no_cors_allow_headers(response: &axum::response::Response) {
    for name in [
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::ACCESS_CONTROL_ALLOW_METHODS,
        header::ACCESS_CONTROL_ALLOW_HEADERS,
    ] {
        assert!(response.headers().get(&name).is_none(), "unexpected {name}");
    }
}

async fn fake_insights_cors(request: axum::extract::Request) -> Response {
    let allowed = request.headers().get(header::ORIGIN).and_then(|value| value.to_str().ok())
        == Some("https://app.example");
    if !allowed {
        return StatusCode::FORBIDDEN.into_response();
    }
    if request.method() == Method::OPTIONS {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "https://app.example")
            .header(header::ACCESS_CONTROL_ALLOW_METHODS, "POST")
            .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "content-type, x-rustzen-project-key")
            .header(header::VARY, "Origin")
            .body(Body::empty())
            .unwrap();
    }
    Response::builder()
        .status(StatusCode::OK)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "https://app.example")
        .header(header::VARY, "Origin")
        .body(Body::empty())
        .unwrap()
}
