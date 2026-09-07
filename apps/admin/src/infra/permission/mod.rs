use crate::common::error::ServiceError;

use chrono::Utc;
use once_cell::sync::Lazy;
#[cfg(test)]
use rustzen_auth::auth::CurrentUser;
use rustzen_auth::capability::{
    BUILTIN_ADMIN_ROLE_CODE, BUILTIN_OWNER_ROLE_CODE, BUILTIN_VIEWER_ROLE_CODE, RolePolicy,
    SYSTEM_WILDCARD,
};
#[cfg(test)]
use rustzen_auth::permission::take_registered_permission_codes;
use rustzen_ipc::ModuleManifest;
use sqlx::{Executor, Sqlite, SqlitePool};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tokio::sync::{Mutex, MutexGuard};

const MENU_STATUS_VISIBLE: i16 = 1;
const MENU_TYPE_DIRECTORY: i16 = 1;
const MENU_TYPE_MENU: i16 = 2;
const MENU_TYPE_BUTTON: i16 = 3;
const DEFAULT_OWNER_USERNAME: &str = "owner";

mod capabilities;
mod navigation;
mod roles;
#[cfg(all(test, feature = "full"))]
mod tests;

use capabilities::{
    build_menu_seed_records, expand_legacy_module_wildcard_grants, menu_seed_record,
    refresh_menu_parent_id, retire_stale_core_permissions, upsert_menu_seed_record,
};
use navigation::reconcile_navigation;
use roles::{database_error, load_permission_cache_snapshot, sync_builtin_roles};

#[cfg(all(test, feature = "full"))]
use capabilities::*;
#[cfg(all(test, feature = "full"))]
use roles::builtin_role_permission_codes;

struct BuiltinRoleSeed {
    code: &'static str,
    name: &'static str,
    description: &'static str,
    sort_order: i32,
}

const BUILTIN_ROLE_SEEDS: &[BuiltinRoleSeed] = &[
    BuiltinRoleSeed {
        code: BUILTIN_OWNER_ROLE_CODE,
        name: "所有者",
        description: "内置所有者角色，拥有全部权限。",
        sort_order: 1,
    },
    BuiltinRoleSeed {
        code: BUILTIN_ADMIN_ROLE_CODE,
        name: "管理员",
        description: "内置管理员角色，拥有日常管理权限。",
        sort_order: 2,
    },
    BuiltinRoleSeed {
        code: BUILTIN_VIEWER_ROLE_CODE,
        name: "查看者",
        description: "内置查看者角色，仅拥有只读权限。",
        sort_order: 3,
    },
];

/// Seed record derived from route-level capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSeedRecord {
    pub permission_code: String,
    pub parent_code: Option<String>,
    pub title: String,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub status: i16,
    pub menu_type: i16,
    pub is_system: bool,
    pub is_manual: bool,
    pub module_id: Option<String>,
    pub module_menu_code: Option<String>,
    pub is_active: bool,
}

pub(crate) type PermissionCacheSnapshot = HashMap<i64, Arc<HashSet<String>>>;

/// Thread-safe in-memory capability cache manager
pub struct PermissionCacheManager {
    cache: Arc<RwLock<PermissionCacheSnapshot>>,
}

impl PermissionCacheManager {
    fn new() -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Get cached capabilities for user.
    #[cfg(test)]
    pub fn get(&self, user_id: i64) -> Option<Arc<HashSet<String>>> {
        self.cache.read().ok()?.get(&user_id).cloned()
    }

    /// Store user capabilities in cache.
    #[cfg(test)]
    pub fn set(&self, user_id: i64, permissions: Arc<HashSet<String>>) {
        let permission_count = permissions.len();
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(user_id, permissions);
            tracing::debug!(permission_count, user_id, "Cached user capabilities");
        }
    }

    pub fn replace_all(&self, replacement: PermissionCacheSnapshot) {
        let user_count = replacement.len();
        match self.cache.write() {
            Ok(mut cache) => *cache = replacement,
            Err(poisoned) => *poisoned.into_inner() = replacement,
        }
        tracing::info!(user_count, "Replaced the in-memory capability snapshot");
    }

    /// Remove user capability cache.
    pub fn remove(&self, user_id: i64) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(&user_id);
            tracing::debug!("Removed capability cache for user {}", user_id);
        }
    }
}

/// Global capability cache instance
static PERMISSION_CACHE: Lazy<PermissionCacheManager> = Lazy::new(PermissionCacheManager::new);
static PERMISSION_CACHE_POPULATION: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static MODULE_MENU_MUTATION: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// Permission service with intelligent caching
pub struct PermissionService;

impl PermissionService {
    pub(crate) async fn lock_module_menu_mutation() -> MutexGuard<'static, ()> {
        MODULE_MENU_MUTATION.lock().await
    }

    /// Synchronize collected route permissions into the menus table.
    #[cfg(test)]
    pub async fn sync_permissions(pool: &SqlitePool) -> Result<(), ServiceError> {
        let raw_codes = take_registered_permission_codes();
        Self::sync_permission_codes(pool, &raw_codes).await
    }

    pub(crate) async fn sync_permission_codes(
        pool: &SqlitePool,
        raw_codes: &[String],
    ) -> Result<(), ServiceError> {
        let seed_records = build_menu_seed_records(raw_codes);

        if seed_records.is_empty() {
            tracing::info!("No route permissions collected for menu sync");
        } else {
            tracing::info!(
                count = seed_records.len(),
                "Synchronizing route permissions into menus"
            );
        }

        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting permission sync transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        if !seed_records.is_empty() {
            retire_stale_core_permissions(&mut tx, &seed_records).await?;
        }

        for record in &seed_records {
            upsert_menu_seed_record(&mut tx, record).await?;
        }

        for record in &seed_records {
            refresh_menu_parent_id(&mut tx, &record.permission_code).await?;
        }

        sync_builtin_roles(&mut tx).await?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing permission sync transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Self::refresh_all_user_permissions(pool).await?;

        Ok(())
    }

    pub async fn reconcile_module_manifest(
        pool: &SqlitePool,
        manifest: &ModuleManifest,
    ) -> Result<(), ServiceError> {
        manifest.validate().map_err(|error| {
            tracing::warn!(%error, module = %manifest.module, "Rejected invalid module Manifest");
            ServiceError::InvalidOperation("invalid module Manifest".to_string())
        })?;
        let raw_codes = manifest
            .routes
            .iter()
            .filter_map(|route| route.permission.clone())
            .collect::<BTreeSet<_>>();
        let mut seed_records = raw_codes
            .into_iter()
            .map(|permission| {
                let mut record = menu_seed_record(&permission);
                record.parent_code = None;
                record.menu_type = MENU_TYPE_BUTTON;
                record
            })
            .collect::<Vec<_>>();
        for record in &mut seed_records {
            record.module_id = Some(manifest.module.clone());
            if let Some(menu) =
                manifest.menus.iter().find(|menu| menu.permission == record.permission_code)
            {
                record.title.clone_from(&menu.title);
                record.path = Some(menu.path.clone());
                record.icon = Some(menu.icon.clone());
                record.sort_order = menu.sort_order;
                record.menu_type = MENU_TYPE_MENU;
                record.module_menu_code = Some(menu.code.clone());
            }
        }

        let _module_menu_guard = Self::lock_module_menu_mutation().await;
        let _cache_guard = PERMISSION_CACHE_POPULATION.lock().await;
        let mut tx = pool.begin().await.map_err(database_error("starting module sync"))?;
        reconcile_navigation(&mut tx, manifest).await?;
        sqlx::query(
            "UPDATE menus
             SET is_active = FALSE, module_menu_code = NULL, updated_at = ?
             WHERE module_id = ? AND is_system = TRUE AND deleted_at IS NULL",
        )
        .bind(Utc::now().naive_utc())
        .bind(&manifest.module)
        .execute(&mut *tx)
        .await
        .map_err(database_error("marking removed module capabilities inactive"))?;

        for record in &seed_records {
            upsert_menu_seed_record(&mut tx, record).await?;
            sqlx::query(
                "UPDATE menus
                 SET is_active = TRUE, is_system = TRUE, module_id = ?, module_menu_code = ?,
                     path = ?, menu_type = ?, updated_at = ?
                 WHERE code = ? AND deleted_at IS NULL",
            )
            .bind(&manifest.module)
            .bind(record.module_menu_code.as_deref())
            .bind(record.path.as_deref())
            .bind(record.menu_type)
            .bind(Utc::now().naive_utc())
            .bind(&record.permission_code)
            .execute(&mut *tx)
            .await
            .map_err(database_error("activating module capabilities"))?;
        }
        for record in &seed_records {
            refresh_menu_parent_id(&mut tx, &record.permission_code).await?;
        }
        expand_legacy_module_wildcard_grants(&mut tx, &manifest.module, &seed_records).await?;
        sync_builtin_roles(&mut tx).await?;
        let cache = load_permission_cache_snapshot(&mut *tx).await?;
        tx.commit().await.map_err(database_error("committing module sync"))?;
        Self::install_cache_snapshot(cache);
        Ok(())
    }

    pub async fn refresh_all_user_permissions(pool: &SqlitePool) -> Result<(), ServiceError> {
        let _cache_guard = PERMISSION_CACHE_POPULATION.lock().await;
        let cache = load_permission_cache_snapshot(pool).await?;
        Self::install_cache_snapshot(cache);
        Ok(())
    }

    fn install_cache_snapshot(cache: PermissionCacheSnapshot) {
        PERMISSION_CACHE.replace_all(cache);
    }

    #[cfg(all(test, feature = "full"))]
    pub async fn refresh_user_permissions(
        pool: &SqlitePool,
        user_id: i64,
    ) -> Result<Vec<String>, ServiceError> {
        let _cache_guard = PERMISSION_CACHE_POPULATION.lock().await;
        let permissions = sqlx::query_scalar(
            "SELECT menu_code FROM user_permissions WHERE user_id = ? ORDER BY menu_code",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(database_error("loading one user's permissions"))?;
        PERMISSION_CACHE.set(user_id, Arc::new(permissions.iter().cloned().collect()));
        Ok(permissions)
    }

    /// Cache user capabilities for isolated gateway and permission tests.
    #[cfg(test)]
    pub fn cache_user_permissions(user_id: i64, permissions: &[String]) {
        PERMISSION_CACHE.set(user_id, Arc::new(permissions.iter().cloned().collect()));
        tracing::info!(permission_count = permissions.len(), user_id, "Cached user permissions");
    }

    /// Clear user cache (called during logout)
    pub fn clear_user_cache(user_id: i64) {
        PERMISSION_CACHE.remove(user_id);
        tracing::info!("Cleared cache for user {} (logout)", user_id);
    }

    #[cfg(test)]
    pub fn load_current_user(user_id: i64, username: &str) -> Result<CurrentUser, ServiceError> {
        let cache = match PERMISSION_CACHE.get(user_id) {
            Some(cache) => cache,
            None => {
                tracing::warn!("No capability cache for user {} - requiring re-auth", user_id);
                return Err(ServiceError::InvalidToken);
            }
        };

        let is_super = cache.contains(SYSTEM_WILDCARD);
        Ok(CurrentUser { user_id, username: username.to_string(), permissions: cache, is_super })
    }
}
