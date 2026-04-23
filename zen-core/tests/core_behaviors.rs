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
use rustzen_core::{
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

    assert_eq!(
        claims,
        AuthClaims { user_id: 7, username: "alice".to_string(), exp: claims.exp, iat: claims.iat }
    );
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
