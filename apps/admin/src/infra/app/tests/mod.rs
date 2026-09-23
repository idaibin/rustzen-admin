use std::{collections::BTreeMap, sync::Arc};

use super::*;
use async_trait::async_trait;
use axum::{
    Extension, Router,
    body::{Body, to_bytes},
    extract::ConnectInfo,
    http::{Method, Request, StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::any,
};
use rustzen_auth::{
    auth::{AuthClaims, AuthContextLoader, CurrentUser, JwtCodec, auth_middleware},
    error::CoreError,
};
use rustzen_ipc::{AccessMode, DelegationSigner, ModuleManifest, RouteManifest};
use tower::ServiceExt;

use crate::features::manage::deploy::service::DeployService;
use crate::features::modules::{
    registry::{ModuleRegistry, RegistrySnapshot},
    service::ModuleControlState,
    types::{ModuleCondition, ModuleRuntime, ModuleSpec},
};
use crate::{
    features::{auth::public_auth_routes, modules::gateway},
    infra::{db::run_migrations, permission::PermissionService},
};
use sqlx::SqlitePool;

use super::routes::{admin_cors, health};

#[derive(Clone)]
struct TestLoader;

#[async_trait]
impl AuthContextLoader for TestLoader {
    async fn load_current_user(&self, claims: &AuthClaims) -> Result<CurrentUser, CoreError> {
        let permissions =
            if claims.username == "owner" { vec!["*".to_owned()] } else { Vec::new() };
        Ok(CurrentUser::new(claims.user_id, claims.username.clone(), permissions, false))
    }
}

async fn session_token(
    pool: &sqlx::SqlitePool,
    codec: &JwtCodec,
    user_id: i64,
    username: &str,
) -> String {
    let epoch: i64 = sqlx::query_scalar("SELECT auth_epoch FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("auth epoch");
    let sid = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let claims = codec.claims_at(user_id, username, &sid, epoch, now);
    crate::features::auth::session::SessionRepository::create(
        pool,
        user_id,
        &sid,
        epoch,
        now,
        claims.exp as i64,
    )
    .await
    .expect("access session");
    codec.encode_claims(&claims).expect("token")
}

async fn activate_seed_owner(pool: &sqlx::SqlitePool) {
    let changed = sqlx::query(
        "UPDATE users SET status = 1 WHERE username = 'owner' AND status = 2 AND deleted_at IS NULL",
    )
    .execute(pool)
    .await
    .expect("activate seeded owner")
    .rows_affected();
    assert_eq!(changed, 1);
}

async fn assert_json_error(response: axum::response::Response, status: StatusCode, code: i32) {
    assert_eq!(response.status(), status);
    assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
            .expect("JSON error response");
    assert_eq!(body["code"], code);
    assert!(body["message"].is_string());
    assert!(body["data"].is_null());
}

mod cors;
mod multipart;
mod roles;
mod routes;
