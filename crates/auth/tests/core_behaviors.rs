use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    middleware,
    response::IntoResponse,
    routing::get,
};
use rustzen_auth::{
    auth::{AuthClaims, AuthContextLoader, CurrentUser, JwtCodec, auth_middleware},
    error::CoreError,
    permission::{
        PermissionsCheck, RouterExt, register_permission_codes, take_registered_permission_codes,
    },
};
use tower::util::ServiceExt;

#[tokio::test]
async fn jwt_codec_round_trip() {
    let codec = JwtCodec::new("secret", 3600);

    let token = codec.encode(7, "alice").expect("token should encode");
    let claims = codec.decode(&token).expect("token should decode");

    assert_eq!(claims.iss, "rustzen-admin");
    assert_eq!(claims.aud, "rustzen-entry");
    assert!(uuid::Uuid::parse_str(&claims.sid).is_ok());
    assert_eq!(claims.user_id, 7);
    assert_eq!(claims.username, "alice");
    assert_eq!(claims.user_auth_epoch, 1);
    assert!(claims.exp > claims.iat);
}

#[test]
fn jwt_codec_rejects_expired_tokens() {
    let codec = JwtCodec::new("secret", -1);
    let token = codec.encode(7, "alice").expect("token should encode");

    assert!(codec.decode(&token).is_err());
}

#[test]
fn jwt_codec_rejects_wrong_issuer_and_audience() {
    let expected = JwtCodec::with_issuer_audience("secret", 3600, "installation-a", "entry-web");
    let wrong_issuer =
        JwtCodec::with_issuer_audience("secret", 3600, "installation-b", "entry-web");
    let wrong_audience =
        JwtCodec::with_issuer_audience("secret", 3600, "installation-a", "other-client");
    let token = wrong_issuer.encode(7, "alice").expect("issuer token");
    assert!(expected.decode(&token).is_err());
    let token = wrong_audience.encode(7, "alice").expect("audience token");
    assert!(expected.decode(&token).is_err());
}

#[test]
fn jwt_codec_rejects_claims_without_session_authority() {
    #[derive(serde::Serialize)]
    struct ClaimsWithoutSession {
        user_id: i64,
        username: &'static str,
        exp: usize,
        iat: usize,
    }
    let now = chrono::Utc::now().timestamp() as usize;
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &ClaimsWithoutSession { user_id: 7, username: "alice", exp: now + 60, iat: now },
        &jsonwebtoken::EncodingKey::from_secret(b"secret"),
    )
    .expect("token without session authority");
    assert!(JwtCodec::new("secret", 3600).decode(&token).is_err());
}

#[tokio::test]
async fn permission_check_respects_super_flag() {
    let user = CurrentUser {
        user_id: 1,
        username: "root".to_string(),
        permissions: Arc::new(HashSet::new()),
        is_super: true,
    };

    assert!(PermissionsCheck::Require("system:user:list").check(&user));
}

#[tokio::test]
async fn permission_check_accepts_exact_global_and_prefix_wildcards() {
    let wildcard_user = CurrentUser::new(
        2,
        "manager",
        ["system:user:list".to_string(), "manage:task:*".to_string(), "dashboard:*".to_string()],
        false,
    );
    let root_user = CurrentUser::new(3, "root", ["*".to_string()], false);

    assert!(PermissionsCheck::Require("system:user:list").check(&wildcard_user));
    assert!(PermissionsCheck::Require("manage:task:update").check(&wildcard_user));
    assert!(PermissionsCheck::Require("manage:task:run:status").check(&wildcard_user));
    assert!(PermissionsCheck::Require("dashboard:view").check(&wildcard_user));
    assert!(PermissionsCheck::Require("anything:anywhere").check(&root_user));
}

#[test]
fn runtime_capability_check_reuses_exact_global_and_prefix_rules() {
    let user = CurrentUser::new(
        5,
        "runtime",
        ["reports:view".to_string(), "insights:*".to_string()],
        false,
    );

    assert!(user.has_capability("reports:view"));
    assert!(user.has_capability("insights:manage"));
    assert!(!user.has_capability("monitor:view"));
}

#[tokio::test]
async fn permission_check_rejects_unrelated_wildcards() {
    let user = CurrentUser::new(4, "viewer", ["manage:task:*".to_string()], false);

    assert!(!PermissionsCheck::Require("manage:deploy:list").check(&user));
    assert!(!PermissionsCheck::Require("system:user:list").check(&user));
}

#[test]
fn registry_collects_and_clears_codes() {
    let _ = take_registered_permission_codes();
    register_permission_codes(["system:user:list", "system:user:create"]);

    assert_eq!(
        take_registered_permission_codes(),
        vec!["system:user:list".to_string(), "system:user:create".to_string()]
    );
    assert!(take_registered_permission_codes().is_empty());
}

#[tokio::test]
async fn route_permission_denies_missing_permission() {
    let app = Router::new()
        .route_with_permission(
            "/users",
            get(|| async { "ok".into_response() }),
            PermissionsCheck::Require("system:user:list"),
        )
        .layer(middleware::from_fn(inject_user_without_permission));

    let response = app
        .oneshot(Request::builder().uri("/users").body(Body::empty()).expect("request"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn auth_middleware_inserts_loaded_user() {
    let codec = JwtCodec::new("secret", 3600);
    let token = codec.encode(9, "alice").expect("token should encode");

    let app = Router::new()
        .route("/me", get(|user: CurrentUser| async move { user.username }))
        .route_layer(middleware::from_fn_with_state((codec.clone(), FixedLoader), auth_middleware));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/me")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_rejects_missing_credentials() {
    let codec = JwtCodec::new("secret", 3600);
    let app = Router::new()
        .route("/me", get(|user: CurrentUser| async move { user.username }))
        .route_layer(middleware::from_fn_with_state((codec, FixedLoader), auth_middleware));

    let response = app
        .oneshot(Request::builder().uri("/me").body(Body::empty()).expect("request"))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

async fn inject_user_without_permission(
    mut request: Request<Body>,
    next: middleware::Next,
) -> impl IntoResponse {
    request.extensions_mut().insert(CurrentUser::new(1, "alice", Vec::<String>::new(), false));
    next.run(request).await
}

#[derive(Clone)]
struct FixedLoader;

#[async_trait]
impl AuthContextLoader for FixedLoader {
    async fn load_current_user(&self, claims: &AuthClaims) -> Result<CurrentUser, CoreError> {
        Ok(CurrentUser::new(
            claims.user_id,
            claims.username.clone(),
            ["system:user:list".to_string()],
            false,
        ))
    }
}
