use crate::common::error::ServiceError;

use chrono::{DateTime, Duration, Utc};
use once_cell::sync::Lazy;
use rustzen_core::{
    auth::CurrentUser,
    permission::{PermissionsCheck, take_registered_permission_codes},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Permission cache expiration time (1 hour)
const CACHE_EXPIRE_HOURS: i64 = 1;
const MENU_STATUS_VISIBLE: i16 = 1;
const MENU_TYPE_DIRECTORY: i16 = 1;
const MENU_TYPE_MENU: i16 = 2;
const MENU_TYPE_BUTTON: i16 = 3;
const SYSTEM_SUPER_ADMIN_CODE: &str = "*";

/// Seed record derived from route permissions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSeedRecord {
    pub permission_code: String,
    pub parent_code: Option<String>,
    pub title: String,
    pub path: Option<String>,
    pub sort_order: i32,
    pub status: i16,
    pub menu_type: i16,
    pub is_system: bool,
    pub is_manual: bool,
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
        let permission_count = permission_cache.permissions.len();
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(user_id, permission_cache);
            tracing::debug!(
                "Cached {} permissions for user {} (expires in {}h)",
                permission_count,
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
    /// Synchronize collected route permissions into the menus table.
    pub async fn sync_permissions(pool: &PgPool) -> Result<(), ServiceError> {
        let raw_codes = take_registered_permission_codes();
        let seed_records = build_menu_seed_records(&raw_codes);

        if seed_records.is_empty() {
            tracing::info!("No route permissions collected for menu sync");
            return Ok(());
        }

        tracing::info!(count = seed_records.len(), "Synchronizing route permissions into menus");

        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting permission sync transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        for record in &seed_records {
            upsert_menu_seed_record(&mut tx, record).await?;
        }

        for record in &seed_records {
            refresh_menu_parent_id(&mut tx, &record.permission_code).await?;
        }

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing permission sync transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(())
    }

    /// Check user permissions with simple caching
    pub async fn check_permissions(
        user_id: i64,
        permissions_check: &PermissionsCheck,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Checking {} for user {}", permissions_check.description(), user_id);
        let current_user = Self::load_current_user(user_id, "")?;
        let has_permission = permissions_check.check(&current_user);
        tracing::debug!(
            "Permission check {} for user {} ({})",
            if has_permission { "GRANTED" } else { "DENIED" },
            user_id,
            permissions_check.description()
        );
        Ok(has_permission)
    }

    /// Check whether a user has a specific permission code.
    pub async fn has_permission(
        user_id: i64,
        permission: &'static str,
    ) -> Result<bool, ServiceError> {
        Self::check_permissions(user_id, &PermissionsCheck::Require(permission)).await
    }

    /// Cache user permissions (called during login)
    pub fn cache_user_permissions(user_id: i64, permissions: &[String]) {
        let permission_cache = UserPermissionCache::new(permissions.to_vec());
        PERMISSION_CACHE.set(user_id, permission_cache);
        tracing::info!(
            "Cached {} permissions for user {} (expires in {}h)",
            permissions.len(),
            user_id,
            CACHE_EXPIRE_HOURS
        );
    }

    /// Clear user cache (called during logout)
    pub fn clear_user_cache(user_id: i64) {
        PERMISSION_CACHE.remove(user_id);
        tracing::info!("Cleared cache for user {} (logout)", user_id);
    }

    pub fn load_current_user(user_id: i64, username: &str) -> Result<CurrentUser, ServiceError> {
        let cache = match PERMISSION_CACHE.get(user_id) {
            Some(cache) => cache,
            None => {
                tracing::warn!("No permission cache for user {} - requiring re-auth", user_id);
                return Err(ServiceError::InvalidToken);
            }
        };

        if cache.is_expired() {
            tracing::info!("Cache expired for user {}", user_id);
            PERMISSION_CACHE.remove(user_id);
            return Err(ServiceError::InvalidToken);
        }

        Ok(CurrentUser::new(
            user_id,
            username,
            cache.permissions.iter().cloned(),
            cache.permissions.contains(SYSTEM_SUPER_ADMIN_CODE),
        ))
    }
}

fn build_menu_seed_records(raw_codes: &[String]) -> Vec<MenuSeedRecord> {
    expand_permission_codes(raw_codes)
        .into_iter()
        .filter(|code| code != SYSTEM_SUPER_ADMIN_CODE)
        .map(|permission_code| menu_seed_record(&permission_code))
        .collect()
}

fn expand_permission_codes(raw_codes: &[String]) -> Vec<String> {
    let mut expanded = BTreeSet::new();

    for raw_code in raw_codes {
        expand_permission_code(raw_code, &mut expanded);
    }

    expanded.into_iter().collect()
}

fn expand_permission_code(raw_code: &str, expanded: &mut BTreeSet<String>) {
    let mut current = raw_code.trim().to_string();
    if current.is_empty() {
        return;
    }

    loop {
        expanded.insert(current.clone());
        match parent_permission_code(&current) {
            Some(parent_code) => current = parent_code,
            None => break,
        }
    }
}

fn parent_permission_code(permission_code: &str) -> Option<String> {
    if permission_code == SYSTEM_SUPER_ADMIN_CODE {
        return None;
    }

    let segments: Vec<&str> = permission_code.split(':').collect();
    if segments.len() <= 1 {
        return None;
    }

    if segments.last() == Some(&"*") {
        if segments.len() <= 2 {
            None
        } else {
            Some(format!("{}:*", segments[..segments.len() - 2].join(":")))
        }
    } else {
        Some(format!("{}:*", segments[..segments.len() - 1].join(":")))
    }
}

fn menu_seed_record(permission_code: &str) -> MenuSeedRecord {
    MenuSeedRecord {
        permission_code: permission_code.to_string(),
        parent_code: parent_permission_code(permission_code),
        title: permission_title(permission_code),
        path: None,
        sort_order: 0,
        status: MENU_STATUS_VISIBLE,
        menu_type: menu_type(permission_code),
        is_system: true,
        is_manual: false,
    }
}

fn menu_type(permission_code: &str) -> i16 {
    if permission_code.ends_with(":*") {
        MENU_TYPE_DIRECTORY
    } else if permission_code.ends_with(":list") || permission_code.ends_with(":view") {
        MENU_TYPE_MENU
    } else {
        MENU_TYPE_BUTTON
    }
}

fn permission_title(permission_code: &str) -> String {
    if permission_code == SYSTEM_SUPER_ADMIN_CODE {
        return "All Permissions".to_string();
    }

    let segments: Vec<&str> = permission_code.split(':').collect();
    if segments.is_empty() {
        return String::new();
    }

    if permission_code.ends_with(":*") {
        let meaningful_segments = &segments[..segments.len().saturating_sub(1)];
        let base = humanize_segments(meaningful_segments);
        if base.is_empty() { "Management".to_string() } else { format!("{base} Management") }
    } else {
        humanize_segments(&segments)
    }
}

fn humanize_segment(segment: &str) -> String {
    segment
        .split(['_', '-', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn humanize_segments(segments: &[&str]) -> String {
    segments
        .iter()
        .map(|segment| humanize_segment(segment))
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

async fn upsert_menu_seed_record(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    record: &MenuSeedRecord,
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();
    sqlx::query(
        "INSERT INTO menus (parent_id, parent_code, name, code, menu_type, status, is_system, is_manual, created_at, updated_at)
         VALUES (0, $1, $2, $3, $4, $5, $6, $7, $8, $8)
         ON CONFLICT (code) DO UPDATE
         SET parent_code = EXCLUDED.parent_code,
             name = EXCLUDED.name,
             menu_type = EXCLUDED.menu_type,
             status = EXCLUDED.status,
             is_system = EXCLUDED.is_system,
             is_manual = EXCLUDED.is_manual,
             updated_at = EXCLUDED.updated_at
         WHERE menus.is_manual = FALSE",
    )
    .bind(record.parent_code.as_deref())
    .bind(&record.title)
    .bind(&record.permission_code)
    .bind(record.menu_type)
    .bind(record.status)
    .bind(record.is_system)
    .bind(record.is_manual)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!(
            "Database error upserting permission seed {}: {:?}",
            record.permission_code,
            e
        );
        ServiceError::DatabaseQueryFailed
    })?;

    Ok(())
}

async fn refresh_menu_parent_id(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    permission_code: &str,
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();
    sqlx::query(
        "UPDATE menus
         SET parent_id = COALESCE(
             (SELECT parent.id FROM menus parent WHERE parent.code = menus.parent_code AND parent.deleted_at IS NULL),
             0
         ),
         updated_at = $2
         WHERE code = $1 AND deleted_at IS NULL AND is_manual = FALSE",
    )
    .bind(permission_code)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!(
            "Database error refreshing parent_id for permission {}: {:?}",
            permission_code,
            e
        );
        ServiceError::DatabaseQueryFailed
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_parent_chain_and_dedupes_codes() {
        let codes = vec![
            "system:user:list".to_string(),
            "system:user:create".to_string(),
            "dashboard:view".to_string(),
            "system:user:*".to_string(),
        ];

        let expanded = expand_permission_codes(&codes);

        assert_eq!(
            expanded,
            vec![
                "dashboard:*".to_string(),
                "dashboard:view".to_string(),
                "system:*".to_string(),
                "system:user:*".to_string(),
                "system:user:create".to_string(),
                "system:user:list".to_string(),
            ]
        );
    }

    #[test]
    fn wildcard_permission_uses_clear_display_title() {
        assert_eq!(permission_title("*"), "All Permissions");
    }

    #[test]
    fn builds_menu_seed_records_with_titles_and_types() {
        let codes = vec![
            "dashboard:view".to_string(),
            "system:user:*".to_string(),
            "system:user:list".to_string(),
            "report:view".to_string(),
            "system:log:export".to_string(),
        ];

        let records = build_menu_seed_records(&codes);

        let dashboard_parent = records
            .iter()
            .find(|record| record.permission_code == "dashboard:*")
            .expect("dashboard parent");
        assert_eq!(dashboard_parent.title, "Dashboard Management");
        assert_eq!(dashboard_parent.menu_type, MENU_TYPE_DIRECTORY);
        assert_eq!(dashboard_parent.parent_code, None);

        let dashboard_view = records
            .iter()
            .find(|record| record.permission_code == "dashboard:view")
            .expect("dashboard view");
        assert_eq!(dashboard_view.title, "Dashboard View");
        assert_eq!(dashboard_view.menu_type, MENU_TYPE_MENU);
        assert_eq!(dashboard_view.parent_code.as_deref(), Some("dashboard:*"));

        let report_view = records
            .iter()
            .find(|record| record.permission_code == "report:view")
            .expect("report view");
        assert_eq!(report_view.title, "Report View");
        assert_eq!(report_view.menu_type, MENU_TYPE_MENU);
        assert_eq!(report_view.parent_code.as_deref(), Some("report:*"));

        let user_group = records
            .iter()
            .find(|record| record.permission_code == "system:user:*")
            .expect("user group");
        assert_eq!(user_group.title, "System User Management");
        assert_eq!(user_group.menu_type, MENU_TYPE_DIRECTORY);
        assert_eq!(user_group.parent_code.as_deref(), Some("system:*"));

        let user_list = records
            .iter()
            .find(|record| record.permission_code == "system:user:list")
            .expect("user list");
        assert_eq!(user_list.title, "System User List");
        assert_eq!(user_list.menu_type, MENU_TYPE_MENU);
        assert_eq!(user_list.parent_code.as_deref(), Some("system:user:*"));

        let log_export = records
            .iter()
            .find(|record| record.permission_code == "system:log:export")
            .expect("log export");
        assert_eq!(log_export.title, "System Log Export");
        assert_eq!(log_export.menu_type, MENU_TYPE_BUTTON);
        assert_eq!(log_export.parent_code.as_deref(), Some("system:log:*"));
    }

    #[test]
    fn seed_records_default_to_system_owned() {
        let record = menu_seed_record("system:user:list");

        assert!(!record.is_manual);
        assert!(record.is_system);
    }

    #[test]
    fn load_current_user_marks_super_from_cached_wildcard() {
        PermissionService::cache_user_permissions(42, &["*".to_string()]);

        let user = PermissionService::load_current_user(42, "root").expect("cached user");

        assert_eq!(user.user_id, 42);
        assert_eq!(user.username, "root");
        assert!(user.is_super);
        assert!(user.permissions.contains("*"));

        PermissionService::clear_user_cache(42);
    }
}
