use crate::common::error::ServiceError;

use chrono::{DateTime, Duration, Utc};
use once_cell::sync::Lazy;
use rustzen_core::{
    auth::CurrentUser,
    capability::{SYSTEM_WILDCARD, is_deploy_capability_code},
    permission::{PermissionsCheck, take_registered_permission_codes},
};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, SqlitePool};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Capability cache expiration time (1 hour)
const CACHE_EXPIRE_HOURS: i64 = 1;
const MENU_STATUS_VISIBLE: i16 = 1;
const MENU_TYPE_DIRECTORY: i16 = 1;
const MENU_TYPE_MENU: i16 = 2;
const MENU_TYPE_BUTTON: i16 = 3;
const BUILTIN_OWNER_ROLE_CODE: &str = "owner";
const BUILTIN_ADMIN_ROLE_CODE: &str = "admin";
const BUILTIN_VIEWER_ROLE_CODE: &str = "viewer";
const LEGACY_SYSTEM_ADMIN_ROLE_CODE: &str = "SYSTEM_ADMIN";
const DEFAULT_OWNER_USERNAME: &str = "superadmin";
const VIEWER_ACTIONS: &[&str] = &["list", "view", "options"];

struct BuiltinRoleSeed {
    code: &'static str,
    name: &'static str,
    description: &'static str,
    sort_order: i32,
}

const BUILTIN_ROLE_SEEDS: &[BuiltinRoleSeed] = &[
    BuiltinRoleSeed {
        code: BUILTIN_OWNER_ROLE_CODE,
        name: "Owner",
        description: "Built-in owner role with the full wildcard grant.",
        sort_order: 1,
    },
    BuiltinRoleSeed {
        code: BUILTIN_ADMIN_ROLE_CODE,
        name: "Admin",
        description: "Built-in administrator role without deploy capabilities.",
        sort_order: 2,
    },
    BuiltinRoleSeed {
        code: BUILTIN_VIEWER_ROLE_CODE,
        name: "Viewer",
        description: "Built-in viewer role with read-only capabilities outside deploy.",
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
    pub sort_order: i32,
    pub status: i16,
    pub menu_type: i16,
    pub is_system: bool,
    pub is_manual: bool,
}

/// Cached user permissions with expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionCache {
    /// User's capability set
    pub permissions: HashSet<String>,
    /// Cache creation timestamp
    pub cached_at: DateTime<Utc>,
}

impl UserPermissionCache {
    /// Create new capability cache for user runtime checks
    pub fn new(permissions: Vec<String>) -> Self {
        Self { permissions: permissions.into_iter().collect(), cached_at: Utc::now() }
    }

    /// Check if cached capabilities have expired.
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let expire_time = self.cached_at + Duration::hours(CACHE_EXPIRE_HOURS);
        now > expire_time
    }
}

/// Thread-safe in-memory capability cache manager
pub struct PermissionCacheManager {
    cache: Arc<RwLock<HashMap<i64, UserPermissionCache>>>,
}

impl PermissionCacheManager {
    fn new() -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Get cached capabilities for user.
    pub fn get(&self, user_id: i64) -> Option<UserPermissionCache> {
        self.cache.read().ok()?.get(&user_id).cloned()
    }

    /// Store user capabilities in cache.
    pub fn set(&self, user_id: i64, permission_cache: UserPermissionCache) {
        let permission_count = permission_cache.permissions.len();
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(user_id, permission_cache);
            tracing::debug!(
                "Cached {} capabilities for user {} (expires in {}h)",
                permission_count,
                user_id,
                CACHE_EXPIRE_HOURS
            );
        }
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

/// Permission service with intelligent caching
pub struct PermissionService;

impl PermissionService {
    /// Synchronize collected route permissions into the menus table.
    pub async fn sync_permissions(pool: &SqlitePool) -> Result<(), ServiceError> {
        let raw_codes = take_registered_permission_codes();
        let seed_records = build_menu_seed_records(&raw_codes);

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

        canonicalize_legacy_manage_permissions(&mut tx).await?;

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

        Ok(())
    }

    /// Check whether a user has a specific capability code.
    pub async fn has_permission(
        user_id: i64,
        capability_code: &'static str,
    ) -> Result<bool, ServiceError> {
        tracing::debug!("Checking required capability '{}' for user {}", capability_code, user_id);
        let current_user = Self::load_current_user(user_id, "")?;
        let has_permission = PermissionsCheck::Require(capability_code).check(&current_user);
        tracing::debug!(
            "Capability check {} for user {} (required capability '{}')",
            if has_permission { "GRANTED" } else { "DENIED" },
            user_id,
            capability_code
        );
        Ok(has_permission)
    }

    /// Cache user capabilities (called during login).
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
                tracing::warn!("No capability cache for user {} - requiring re-auth", user_id);
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
            cache.permissions.contains(SYSTEM_WILDCARD),
        ))
    }
}

async fn canonicalize_legacy_manage_permissions(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
) -> Result<(), ServiceError> {
    canonicalize_menu_code_prefix(tx, "system:dict:", "manage:dict:").await?;
    canonicalize_menu_code_prefix(tx, "system:log:", "manage:log:").await?;

    Ok(())
}

async fn canonicalize_menu_code_prefix(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    legacy_prefix: &str,
    current_prefix: &str,
) -> Result<(), ServiceError> {
    let like_pattern = format!("{}%", legacy_prefix);
    let now = Utc::now().naive_utc();

    sqlx::query(
        "UPDATE menus
         SET code = REPLACE(code, ?, ?),
             parent_code = CASE
                 WHEN parent_code LIKE ? THEN REPLACE(parent_code, ?, ?)
                 ELSE parent_code
             END,
             updated_at = ?
         WHERE is_manual = FALSE
           AND deleted_at IS NULL
           AND code LIKE ?",
    )
    .bind(legacy_prefix)
    .bind(current_prefix)
    .bind(&like_pattern)
    .bind(legacy_prefix)
    .bind(current_prefix)
    .bind(now)
    .bind(&like_pattern)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!(
            "Database error normalizing legacy permission menu code {} -> {}: {:?}",
            legacy_prefix,
            current_prefix,
            e
        );
        ServiceError::DatabaseQueryFailed
    })?;

    Ok(())
}

fn build_menu_seed_records(raw_codes: &[String]) -> Vec<MenuSeedRecord> {
    expand_permission_codes(raw_codes)
        .into_iter()
        .filter(|code| code != SYSTEM_WILDCARD)
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
    if permission_code == SYSTEM_WILDCARD {
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
    if permission_code == SYSTEM_WILDCARD {
        return "All Capabilities".to_string();
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
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    record: &MenuSeedRecord,
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();
    sqlx::query(
        "INSERT INTO menus (parent_id, parent_code, name, code, menu_type, status, is_system, is_manual, created_at, updated_at)
         VALUES (0, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT DO UPDATE
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
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    permission_code: &str,
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();
    sqlx::query(
        "UPDATE menus
         SET parent_id = COALESCE(
             (SELECT parent.id FROM menus parent WHERE parent.code = menus.parent_code AND parent.deleted_at IS NULL),
             0
         ),
         updated_at = ?
         WHERE code = ? AND deleted_at IS NULL AND is_manual = FALSE",
    )
    .bind(now)
    .bind(permission_code)
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

async fn sync_builtin_roles(tx: &mut sqlx::Transaction<'_, Sqlite>) -> Result<(), ServiceError> {
    upsert_all_capabilities_menu(tx).await?;
    canonicalize_legacy_owner_role(tx).await?;

    let menu_code_rows = list_menu_code_rows(tx).await?;
    let menu_codes = menu_code_rows.iter().map(|(_, code)| code.clone()).collect::<Vec<_>>();

    for role in BUILTIN_ROLE_SEEDS {
        let role_id = upsert_builtin_role(tx, role).await?;
        let role_codes = builtin_role_permission_codes(role.code, &menu_codes);
        let menu_ids = menu_code_rows
            .iter()
            .filter_map(|(id, code)| role_codes.contains(code).then_some(*id))
            .collect::<Vec<_>>();
        replace_builtin_role_menus(tx, role_id, &menu_ids).await?;
    }

    bind_default_owner_user(tx).await?;

    Ok(())
}

async fn upsert_all_capabilities_menu(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();
    sqlx::query(
        "INSERT INTO menus (parent_id, name, code, menu_type, sort_order, status, is_system, is_manual, created_at, updated_at)
         VALUES (0, 'All Capabilities', ?, ?, 1, ?, TRUE, FALSE, ?, ?)
         ON CONFLICT DO UPDATE
         SET name = EXCLUDED.name,
             menu_type = EXCLUDED.menu_type,
             is_system = EXCLUDED.is_system,
             is_manual = EXCLUDED.is_manual,
             updated_at = EXCLUDED.updated_at
         WHERE menus.is_manual = FALSE",
    )
    .bind(SYSTEM_WILDCARD)
    .bind(MENU_TYPE_DIRECTORY)
    .bind(MENU_STATUS_VISIBLE)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error upserting owner wildcard menu: {:?}", e);
        ServiceError::DatabaseQueryFailed
    })?;

    Ok(())
}

async fn canonicalize_legacy_owner_role(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();

    sqlx::query(
        "UPDATE roles
         SET name = 'Owner',
             code = ?,
             description = 'Built-in owner role with the full wildcard grant.',
             status = 1,
             is_system = TRUE,
             sort_order = 1,
             updated_at = ?
         WHERE code = ?
           AND deleted_at IS NULL
           AND NOT EXISTS (
               SELECT 1 FROM roles owner_role
               WHERE owner_role.code = ? AND owner_role.deleted_at IS NULL
           )",
    )
    .bind(BUILTIN_OWNER_ROLE_CODE)
    .bind(now)
    .bind(LEGACY_SYSTEM_ADMIN_ROLE_CODE)
    .bind(BUILTIN_OWNER_ROLE_CODE)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error canonicalizing legacy owner role: {:?}", e);
        ServiceError::DatabaseQueryFailed
    })?;

    sqlx::query(
        "INSERT OR IGNORE INTO user_roles (user_id, role_id, created_at)
         SELECT ur.user_id, owner_role.id, ?
         FROM user_roles ur
         INNER JOIN roles legacy_role ON legacy_role.id = ur.role_id
         INNER JOIN roles owner_role ON owner_role.code = ? AND owner_role.deleted_at IS NULL
         WHERE legacy_role.code = ?
           AND legacy_role.deleted_at IS NULL",
    )
    .bind(now)
    .bind(BUILTIN_OWNER_ROLE_CODE)
    .bind(LEGACY_SYSTEM_ADMIN_ROLE_CODE)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error moving legacy owner user bindings: {:?}", e);
        ServiceError::DatabaseQueryFailed
    })?;

    sqlx::query(
        "UPDATE roles
         SET deleted_at = ?,
             updated_at = ?
         WHERE code = ?
           AND deleted_at IS NULL
           AND EXISTS (
               SELECT 1 FROM roles owner_role
               WHERE owner_role.code = ? AND owner_role.deleted_at IS NULL
           )",
    )
    .bind(now)
    .bind(now)
    .bind(LEGACY_SYSTEM_ADMIN_ROLE_CODE)
    .bind(BUILTIN_OWNER_ROLE_CODE)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error disabling duplicate legacy owner role: {:?}", e);
        ServiceError::DatabaseQueryFailed
    })?;

    Ok(())
}

async fn list_menu_code_rows(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
) -> Result<Vec<(i64, String)>, ServiceError> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT id, code
         FROM menus
         WHERE deleted_at IS NULL
           AND code IS NOT NULL
         ORDER BY id",
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error listing menu codes for built-in role sync: {:?}", e);
        ServiceError::DatabaseQueryFailed
    })
}

fn builtin_role_permission_codes(role_code: &str, menu_codes: &[String]) -> Vec<String> {
    menu_codes.iter().filter(|code| builtin_role_allows_code(role_code, code)).cloned().collect()
}

fn builtin_role_allows_code(role_code: &str, code: &str) -> bool {
    match role_code {
        BUILTIN_OWNER_ROLE_CODE => code == SYSTEM_WILDCARD,
        BUILTIN_ADMIN_ROLE_CODE => is_assignable_leaf_capability(code),
        BUILTIN_VIEWER_ROLE_CODE => is_assignable_leaf_capability(code) && is_view_capability(code),
        _ => false,
    }
}

fn is_assignable_leaf_capability(code: &str) -> bool {
    code != SYSTEM_WILDCARD && !code.ends_with(":*") && !is_deploy_capability_code(code)
}

fn is_view_capability(code: &str) -> bool {
    code.rsplit(':').next().is_some_and(|action| VIEWER_ACTIONS.contains(&action))
}

async fn upsert_builtin_role(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    role: &BuiltinRoleSeed,
) -> Result<i64, ServiceError> {
    let now = Utc::now().naive_utc();
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO roles (name, code, description, status, is_system, sort_order, created_at, updated_at)
         VALUES (?, ?, ?, 1, TRUE, ?, ?, ?)
         ON CONFLICT DO UPDATE
         SET name = EXCLUDED.name,
             description = EXCLUDED.description,
             status = EXCLUDED.status,
             is_system = EXCLUDED.is_system,
             sort_order = EXCLUDED.sort_order,
             updated_at = EXCLUDED.updated_at
         RETURNING id",
    )
    .bind(role.name)
    .bind(role.code)
    .bind(role.description)
    .bind(role.sort_order)
    .bind(now)
    .bind(now)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error upserting built-in role {}: {:?}", role.code, e);
        ServiceError::DatabaseQueryFailed
    })
}

async fn replace_builtin_role_menus(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    role_id: i64,
    menu_ids: &[i64],
) -> Result<(), ServiceError> {
    sqlx::query("DELETE FROM role_menus WHERE role_id = ?")
        .bind(role_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error clearing built-in role menus: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

    if menu_ids.is_empty() {
        return Ok(());
    }

    let now = Utc::now().naive_utc();
    let mut query_builder: sqlx::QueryBuilder<Sqlite> =
        sqlx::QueryBuilder::new("INSERT INTO role_menus (role_id, menu_id, created_at) ");
    query_builder.push_values(menu_ids.iter(), |mut builder, menu_id| {
        builder.push_bind(role_id).push_bind(menu_id).push_bind(now);
    });

    query_builder.build().execute(&mut **tx).await.map_err(|e| {
        tracing::error!("Database error inserting built-in role menus: {:?}", e);
        ServiceError::DatabaseQueryFailed
    })?;

    Ok(())
}

async fn bind_default_owner_user(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
) -> Result<(), ServiceError> {
    sqlx::query(
        "INSERT OR IGNORE INTO user_roles (user_id, role_id, created_at)
         SELECT u.id, r.id, ?
         FROM users u
         INNER JOIN roles r ON r.code = ? AND r.deleted_at IS NULL
         WHERE u.username = ?
           AND u.deleted_at IS NULL",
    )
    .bind(Utc::now().naive_utc())
    .bind(BUILTIN_OWNER_ROLE_CODE)
    .bind(DEFAULT_OWNER_USERNAME)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error binding default owner user: {:?}", e);
        ServiceError::DatabaseQueryFailed
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_capability_chain_and_dedupes_codes() {
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
    fn wildcard_capability_uses_clear_display_title() {
        assert_eq!(permission_title("*"), "All Capabilities");
    }

    #[test]
    fn builds_menu_seed_records_with_capability_titles_and_types() {
        let codes = vec![
            "dashboard:view".to_string(),
            "system:user:*".to_string(),
            "system:user:list".to_string(),
            "report:view".to_string(),
            "manage:log:export".to_string(),
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
            .find(|record| record.permission_code == "manage:log:export")
            .expect("log export");
        assert_eq!(log_export.title, "Manage Log Export");
        assert_eq!(log_export.menu_type, MENU_TYPE_BUTTON);
        assert_eq!(log_export.parent_code.as_deref(), Some("manage:log:*"));
    }

    #[test]
    fn seed_records_default_to_system_owned_capabilities() {
        let record = menu_seed_record("system:user:list");

        assert!(!record.is_manual);
        assert!(record.is_system);
    }

    #[test]
    fn builtin_role_policy_keeps_owner_as_only_wildcard_grant() {
        let menu_codes = vec![
            "*".to_string(),
            "system:*".to_string(),
            "system:user:*".to_string(),
            "system:user:list".to_string(),
            "system:user:create".to_string(),
            "dashboard:*".to_string(),
            "dashboard:view".to_string(),
            "manage:*".to_string(),
            "manage:task:*".to_string(),
            "manage:task:list".to_string(),
            "manage:task:run".to_string(),
            "manage:deploy:*".to_string(),
            "manage:deploy:list".to_string(),
            "manage:log:export".to_string(),
            "manage:dict:options".to_string(),
        ];

        let owner_codes = builtin_role_permission_codes(BUILTIN_OWNER_ROLE_CODE, &menu_codes);
        let admin_codes = builtin_role_permission_codes(BUILTIN_ADMIN_ROLE_CODE, &menu_codes);
        let viewer_codes = builtin_role_permission_codes(BUILTIN_VIEWER_ROLE_CODE, &menu_codes);

        assert_eq!(owner_codes, vec!["*".to_string()]);

        assert!(admin_codes.contains(&"system:user:create".to_string()));
        assert!(admin_codes.contains(&"manage:task:run".to_string()));
        assert!(admin_codes.contains(&"manage:log:export".to_string()));
        assert!(!admin_codes.iter().any(|code| code == "*" || code.ends_with(":*")));
        assert!(!admin_codes.iter().any(|code| is_deploy_capability_code(code)));

        assert!(viewer_codes.contains(&"system:user:list".to_string()));
        assert!(viewer_codes.contains(&"dashboard:view".to_string()));
        assert!(viewer_codes.contains(&"manage:dict:options".to_string()));
        assert!(!viewer_codes.contains(&"system:user:create".to_string()));
        assert!(!viewer_codes.contains(&"manage:task:run".to_string()));
        assert!(!viewer_codes.contains(&"manage:log:export".to_string()));
        assert!(!viewer_codes.iter().any(|code| code == "*" || code.ends_with(":*")));
        assert!(!viewer_codes.iter().any(|code| is_deploy_capability_code(code)));
    }

    #[tokio::test]
    async fn sync_permissions_persists_builtin_roles_and_default_owner() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");

        rustzen_core::permission::register_permission_codes([
            "dashboard:view",
            "system:user:list",
            "system:user:create",
            "manage:task:list",
            "manage:task:run",
            "manage:deploy:list",
            "manage:deploy:run",
            "manage:dict:options",
        ]);

        PermissionService::sync_permissions(&pool).await.expect("permission sync");

        let owner_permissions = role_permission_codes(&pool, BUILTIN_OWNER_ROLE_CODE).await;
        let admin_permissions = role_permission_codes(&pool, BUILTIN_ADMIN_ROLE_CODE).await;
        let viewer_permissions = role_permission_codes(&pool, BUILTIN_VIEWER_ROLE_CODE).await;

        assert_eq!(owner_permissions, vec!["*".to_string()]);
        assert!(admin_permissions.contains(&"system:user:create".to_string()));
        assert!(admin_permissions.contains(&"manage:task:run".to_string()));
        assert!(!admin_permissions.iter().any(|code| is_deploy_capability_code(code)));
        assert!(!admin_permissions.iter().any(|code| code == "*" || code.ends_with(":*")));

        assert!(viewer_permissions.contains(&"dashboard:view".to_string()));
        assert!(viewer_permissions.contains(&"system:user:list".to_string()));
        assert!(viewer_permissions.contains(&"manage:dict:options".to_string()));
        assert!(!viewer_permissions.contains(&"system:user:create".to_string()));
        assert!(!viewer_permissions.contains(&"manage:task:run".to_string()));
        assert!(!viewer_permissions.iter().any(|code| is_deploy_capability_code(code)));
        assert!(!viewer_permissions.iter().any(|code| code == "*" || code.ends_with(":*")));

        let superadmin_wildcard_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM user_permissions
             WHERE username = ? AND menu_code = ?",
        )
        .bind(DEFAULT_OWNER_USERNAME)
        .bind(SYSTEM_WILDCARD)
        .fetch_one(&pool)
        .await
        .expect("superadmin permissions");
        assert_eq!(superadmin_wildcard_count, 1);
    }

    async fn role_permission_codes(pool: &SqlitePool, role_code: &str) -> Vec<String> {
        sqlx::query_scalar::<_, String>(
            "SELECT m.code
             FROM roles r
             INNER JOIN role_menus rm ON rm.role_id = r.id
             INNER JOIN menus m ON m.id = rm.menu_id
             WHERE r.code = ?
               AND r.deleted_at IS NULL
               AND m.deleted_at IS NULL
             ORDER BY m.code",
        )
        .bind(role_code)
        .fetch_all(pool)
        .await
        .expect("role permission codes")
    }

    #[test]
    fn load_current_user_marks_super_from_cached_capability_wildcard() {
        PermissionService::cache_user_permissions(42, &["*".to_string()]);

        let user = PermissionService::load_current_user(42, "root").expect("cached user");

        assert_eq!(user.user_id, 42);
        assert_eq!(user.username, "root");
        assert!(user.is_super);
        assert!(user.permissions.contains("*"));

        PermissionService::clear_user_cache(42);
    }
}
