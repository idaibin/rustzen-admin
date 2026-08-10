use axum::http::Uri;
use chrono::Utc;
use rustzen_storage::SqlitePool;
use sha2::{Digest, Sha256};

use crate::common::error::AppError;
use crate::features::tracking::IngestionState;

use super::{
    repo,
    types::{CollectionPolicy, CollectionPolicyUpdate, Settings},
};

/// Hash shipped by the original 0001 seed. It is not an operator-configured
/// routing identifier and must never make a fresh installation appear ready.
pub(crate) const LEGACY_SEEDED_PROJECT_KEY_HASH: &str =
    "6ab538c2b9772ed3ea67476cf10035de9a31718833b1ab27c2d28c269f9a5b95";

pub async fn get(pool: &SqlitePool) -> Result<Settings, AppError> {
    repo::get(pool).await.map_err(AppError::internal)
}

pub async fn get_collection_policy(pool: &SqlitePool) -> Result<CollectionPolicy, AppError> {
    let row = repo::get_collection_policy(pool).await.map_err(AppError::internal)?;
    let allowed_origins = parse_allowed_origins(&row.allowed_origins)?;
    Ok(CollectionPolicy {
        collection_enabled: row.collection_enabled != 0,
        project_configured: project_configured(&row.project_key_hash),
        allowed_origins,
    })
}

pub async fn update_collection_policy(
    pool: &SqlitePool,
    ingestion: &IngestionState,
    update: CollectionPolicyUpdate,
) -> Result<CollectionPolicy, AppError> {
    let _policy_guard = ingestion.policy_guard().await;
    let current = repo::get_collection_policy(pool).await.map_err(AppError::internal)?;
    let project_key_hash = update.project_key.as_deref().map(hash_project_key).transpose()?;
    if update.collection_enabled
        && project_key_hash.is_none()
        && !project_configured(&current.project_key_hash)
    {
        return Err(AppError::bad_request(
            "a new project key is required before collection can be enabled",
        ));
    }
    let allowed_origins = update
        .allowed_origins
        .into_iter()
        .map(|origin| {
            normalize_origin(&origin)
                .map_err(|_| AppError::bad_request("allowed origin is invalid"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let allowed_origins = serde_json::to_string(&allowed_origins).map_err(AppError::internal)?;
    let updated_at = Utc::now().to_rfc3339();
    repo::update_collection_policy(
        pool,
        update.collection_enabled,
        project_key_hash.as_deref(),
        &allowed_origins,
        &updated_at,
    )
    .await
    .map_err(|error| {
        if let sqlx::Error::Database(database) = &error
            && database.message().contains("UNIQUE")
        {
            return AppError::bad_request("project key is already registered");
        }
        AppError::internal(error)
    })?;
    get_collection_policy(pool).await
}

fn project_configured(project_key_hash: &str) -> bool {
    let project_key_hash = project_key_hash.trim();
    !project_key_hash.is_empty() && project_key_hash != LEGACY_SEEDED_PROJECT_KEY_HASH
}

pub(crate) fn hash_project_key(project_key: &str) -> Result<String, AppError> {
    let project_key = project_key.trim();
    if project_key.is_empty() || project_key.len() > 256 {
        return Err(AppError::bad_request("project key must contain 1 to 256 characters"));
    }
    Ok(Sha256::digest(project_key.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect())
}

pub(crate) fn normalize_origin(raw: &str) -> Result<String, ()> {
    let uri: Uri = raw.trim().parse().map_err(|_| ())?;
    let scheme = uri.scheme_str().ok_or(())?.to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https")
        || uri.path_and_query().is_some_and(|path| !matches!(path.as_str(), "" | "/"))
        || raw.contains(['@', '?', '#', '\n', '\r'])
    {
        return Err(());
    }
    let authority = uri.authority().ok_or(())?;
    let host = authority.host().trim().to_ascii_lowercase();
    if host.is_empty() {
        return Err(());
    }
    let port = match (scheme.as_str(), authority.port_u16()) {
        ("http", Some(80)) | ("https", Some(443)) | (_, None) => String::new(),
        (_, Some(port)) => format!(":{port}"),
    };
    Ok(format!("{scheme}://{host}{port}"))
}

fn parse_allowed_origins(raw: &str) -> Result<Vec<String>, AppError> {
    let origins: Vec<String> = serde_json::from_str(raw).map_err(AppError::internal)?;
    origins
        .into_iter()
        .map(|origin| {
            normalize_origin(&origin)
                .map_err(|_| AppError::internal("Insights origin policy is invalid"))
        })
        .collect()
}
