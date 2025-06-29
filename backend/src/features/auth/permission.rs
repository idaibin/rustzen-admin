use crate::{common::error::ServiceError, features::auth::repo::AuthRepository};
use chrono::{DateTime, Duration, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Permission cache expiration time (1 hour)
const CACHE_EXPIRE_HOURS: i64 = 1;
const ZEN_ADMIN_CODE: &str = "zen_admin";

/// Permission check types for flexible access control
#[derive(Debug, Clone)]
pub enum PermissionsCheck {
    /// User needs any one of the permissions (OR logic)
    Any(Vec<&'static str>),
    /// User needs all permissions (AND logic)
    All(Vec<&'static str>),
    /// User needs this specific permission
    Single(&'static str),
}

impl PermissionsCheck {
    /// Core permission validation logic
    pub fn check(&self, user_permissions: &HashSet<String>) -> bool {
        // If user is ZenAdmin, allow all permissions
        if user_permissions.contains(ZEN_ADMIN_CODE) {
            return true;
        }
        match self {
            PermissionsCheck::Single(code) => user_permissions.contains(*code),
            PermissionsCheck::Any(codes) => {
                codes.iter().any(|code| user_permissions.contains(*code))
            }
            PermissionsCheck::All(codes) => {
                codes.iter().all(|code| user_permissions.contains(*code))
            }
        }
    }

    /// Returns a description of the permission check for logging
    pub fn description(&self) -> String {
        match self {
            PermissionsCheck::Single(permission) => format!("single permission '{}'", permission),
            PermissionsCheck::Any(permissions) => format!("any of permissions {:?}", permissions),
            PermissionsCheck::All(permissions) => format!("all permissions {:?}", permissions),
        }
    }
}

/// Cached user permissions with expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionCache {
    /// User's permissions set
    pub permissions: HashSet<String>,
    /// Cache creation timestamp
    pub cached_at: DateTime<Utc>,
}

impl UserPermissionCache {
    /// Create new permission cache
    pub fn new(permissions: Vec<String>) -> Self {
        Self { permissions: permissions.into_iter().collect(), cached_at: Utc::now() }
    }

    /// Check if cache has expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let expire_time = self.cached_at + Duration::hours(CACHE_EXPIRE_HOURS);
        now > expire_time
    }

    /// Get remaining cache time in seconds
    pub fn remaining_seconds(&self) -> i64 {
        let now = Utc::now();
        let expire_time = self.cached_at + Duration::hours(CACHE_EXPIRE_HOURS);
        (expire_time - now).num_seconds().max(0)
    }
}

/// Thread-safe in-memory permission cache manager
pub struct PermissionCacheManager {
    cache: Arc<RwLock<HashMap<i64, UserPermissionCache>>>,
}

impl PermissionCacheManager {
    fn new() -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Get cached permissions for user
    pub fn get(&self, user_id: i64) -> Option<UserPermissionCache> {
        self.cache.read().ok()?.get(&user_id).cloned()
    }

    /// Store user permissions in cache
    pub fn set(&self, user_id: i64, permission_cache: UserPermissionCache) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(user_id, permission_cache);
            tracing::debug!(
                "Cached {} permissions for user {} (expires in {}h)",
                cache.get(&user_id).map(|c| c.permissions.len()).unwrap_or(0),
                user_id,
                CACHE_EXPIRE_HOURS
            );
        }
    }

    /// Remove user permissions from cache
    pub fn remove(&self, user_id: i64) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(&user_id);
            tracing::debug!("Removed permission cache for user {}", user_id);
        }
    }
}

/// Global permission cache instance
static PERMISSION_CACHE: Lazy<PermissionCacheManager> = Lazy::new(PermissionCacheManager::new);

/// Permission service with intelligent caching
pub struct PermissionService;

impl PermissionService {
    /// Check user permissions with caching
    ///
    /// Strategy:
    /// 1. Try cache first (auto-refresh if expired)
    /// 2. Return permission check result
    /// 3. Require re-auth if no cache exists
    pub async fn check_permissions(
        pool: &PgPool,
        user_id: i64,
        permissions_check: &PermissionsCheck,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Checking {} for user {}", permissions_check.description(), user_id);

        // Try to get cached permissions
        if let Some(cache) = Self::get_cached_permissions(pool, user_id).await? {
            let has_permission = permissions_check.check(&cache.permissions);

            tracing::debug!(
                "Permission check {} for user {} ({})",
                if has_permission { "GRANTED" } else { "DENIED" },
                user_id,
                permissions_check.description()
            );

            return Ok(has_permission);
        }

        // No cache - require re-authentication
        tracing::warn!("No permission cache for user {} - requiring re-auth", user_id);
        Err(ServiceError::InvalidToken)
    }

    /// Cache user permissions (called during login)
    pub fn cache_user_permissions(user_id: i64, permissions: Vec<String>) {
        let permission_cache = UserPermissionCache::new(permissions);
        PERMISSION_CACHE.set(user_id, permission_cache.clone());

        tracing::info!(
            "Cached {} permissions for user {} (expires in {}h)",
            permission_cache.permissions.len(),
            user_id,
            CACHE_EXPIRE_HOURS
        );
    }

    /// Get cached permissions with auto-refresh on expiration
    pub async fn get_cached_permissions(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Option<UserPermissionCache>, ServiceError> {
        if let Some(cache) = PERMISSION_CACHE.get(user_id) {
            if cache.is_expired() {
                tracing::info!("Cache expired for user {} - refreshing", user_id);
                let new_cache = Self::load_user_permissions_from_db(pool, user_id).await?;
                return Ok(Some(new_cache));
            }

            tracing::debug!(
                "Found valid cache for user {} ({} permissions, {}s remaining)",
                user_id,
                cache.permissions.len(),
                cache.remaining_seconds()
            );
            return Ok(Some(cache));
        }

        Ok(None)
    }

    /// Clear user cache (called during logout)
    pub fn clear_user_cache(user_id: i64) {
        PERMISSION_CACHE.remove(user_id);
        tracing::info!("Cleared cache for user {} (logout)", user_id);
    }

    /// Load permissions from database and update cache
    async fn load_user_permissions_from_db(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<UserPermissionCache, ServiceError> {
        tracing::info!("Loading permissions from DB for user {}", user_id);

        let permissions =
            AuthRepository::get_user_permissions(pool, user_id).await.map_err(|e| {
                tracing::error!("Failed to load permissions for user {}: {:?}", user_id, e);
                ServiceError::DatabaseQueryFailed
            })?;

        let permission_cache = UserPermissionCache::new(permissions);
        PERMISSION_CACHE.set(user_id, permission_cache.clone());

        tracing::info!(
            "Loaded {} permissions for user {} from DB",
            permission_cache.permissions.len(),
            user_id
        );

        Ok(permission_cache)
    }
}
