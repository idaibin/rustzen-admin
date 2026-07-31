use crate::common::error::ServiceError;

use chrono::Utc;
use once_cell::sync::Lazy;
use rustzen_auth::{
    auth::CurrentUser,
    capability::{
        BUILTIN_ADMIN_ROLE_CODE, BUILTIN_OWNER_ROLE_CODE, BUILTIN_VIEWER_ROLE_CODE, RolePolicy,
        SYSTEM_WILDCARD,
    },
    permission::{PermissionsCheck, take_registered_permission_codes},
};
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
    pub fn get(&self, user_id: i64) -> Option<Arc<HashSet<String>>> {
        self.cache.read().ok()?.get(&user_id).cloned()
    }

    /// Store user capabilities in cache.
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
        let presentation_overrides =
            sqlx::query_as::<_, (String, String, String, Option<String>, i32, i16)>(
                "SELECT module_menu_code, code, name, icon, sort_order, status
             FROM menus
             WHERE module_id = ?
               AND module_menu_code IS NOT NULL
               AND is_manual = TRUE
               AND is_active = TRUE
               AND deleted_at IS NULL",
            )
            .bind(&manifest.module)
            .fetch_all(&mut *tx)
            .await
            .map_err(database_error("loading module menu presentation overrides"))?;
        for record in &mut seed_records {
            let Some(menu_code) = record.module_menu_code.as_deref() else {
                continue;
            };
            if let Some((_, _, title, icon, sort_order, status)) =
                presentation_overrides.iter().find(|(code, ..)| code == menu_code)
            {
                record.title.clone_from(title);
                record.icon.clone_from(icon);
                record.sort_order = *sort_order;
                record.status = *status;
                record.is_manual = true;
            }
        }
        for (menu_code, source_permission, ..) in &presentation_overrides {
            let target_permission = seed_records
                .iter()
                .find(|record| record.module_menu_code.as_deref() == Some(menu_code))
                .map(|record| record.permission_code.as_str());
            if target_permission != Some(source_permission.as_str()) {
                sqlx::query(
                    "UPDATE menus SET is_manual = FALSE, updated_at = ?
                     WHERE code = ? AND module_id = ? AND deleted_at IS NULL",
                )
                .bind(Utc::now().naive_utc())
                .bind(source_permission)
                .bind(&manifest.module)
                .execute(&mut *tx)
                .await
                .map_err(database_error("releasing the previous module menu override"))?;
            }
        }
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
            if record.is_manual {
                sqlx::query(
                    "UPDATE menus
                     SET name = ?, icon = ?, sort_order = ?, status = ?, is_manual = TRUE,
                         updated_at = ?
                     WHERE code = ? AND deleted_at IS NULL",
                )
                .bind(&record.title)
                .bind(record.icon.as_deref())
                .bind(record.sort_order)
                .bind(record.status)
                .bind(Utc::now().naive_utc())
                .bind(&record.permission_code)
                .execute(&mut *tx)
                .await
                .map_err(database_error("transferring module menu presentation override"))?;
            }
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
        icon: None,
        sort_order: 0,
        status: MENU_STATUS_VISIBLE,
        menu_type: menu_type(permission_code),
        is_system: true,
        is_manual: false,
        module_id: None,
        module_menu_code: None,
        is_active: true,
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
        return "全部权限".to_string();
    }

    let segments: Vec<&str> = permission_code.split(':').collect();
    if segments.is_empty() {
        return String::new();
    }

    if permission_code.ends_with(":*") {
        let meaningful_segments = &segments[..segments.len().saturating_sub(1)];
        let base = localize_segments(meaningful_segments);
        if base.is_empty() || base.ends_with("管理") { base } else { format!("{base}管理") }
    } else {
        localize_segments(&segments)
    }
}

fn localize_segment(segment: &str) -> String {
    match segment {
        "dashboard" => "仪表盘",
        "system" => "系统",
        "user" => "用户",
        "role" => "角色",
        "menu" => "菜单",
        "module" => "模块",
        "status" => "状态",
        "manage" => "管理",
        "log" => "日志",
        "task" => "任务",
        "deploy" => "部署",
        "monitor" => "监控",
        "node" => "节点",
        "check" => "服务监控",
        "incident" => "事件",
        "settings" => "设置",
        "insights" => "分析",
        "overview" => "概览",
        "project" => "项目",
        "event" => "事件",
        "page" => "页面",
        "api" => "API",
        "report" | "reports" => "报表",
        "flow" => "流程",
        "run" => "执行",
        "schedule" => "计划",
        "template" => "模板",
        "create" => "新增",
        "delete" => "删除",
        "list" => "列表",
        "options" => "选项",
        "update" => "修改",
        "password" => "密码",
        "view" => "查看",
        "export" => "导出",
        "analyze" => "分析",
        "recover" => "恢复",
        "restart" => "重启",
        other => return humanize_segment(other),
    }
    .to_string()
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

fn localize_segments(segments: &[&str]) -> String {
    segments.iter().map(|segment| localize_segment(segment)).collect::<Vec<_>>().join("")
}

async fn retire_stale_core_permissions(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    current_records: &[MenuSeedRecord],
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();
    let mut query = sqlx::QueryBuilder::<Sqlite>::new(
        "UPDATE menus
         SET is_active = FALSE, updated_at = ",
    );
    query
        .push_bind(now)
        .push(
            " WHERE module_id IS NULL
               AND is_system = TRUE
               AND code <> ",
        )
        .push_bind(SYSTEM_WILDCARD)
        .push(" AND deleted_at IS NULL AND code NOT IN (");
    let mut codes = query.separated(", ");
    for record in current_records {
        codes.push_bind(&record.permission_code);
    }
    codes.push_unseparated(")");
    query
        .build()
        .execute(&mut **tx)
        .await
        .map_err(database_error("retiring stale core permissions"))?;

    Ok(())
}

async fn upsert_menu_seed_record(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    record: &MenuSeedRecord,
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();
    sqlx::query(
        "INSERT INTO menus (
             parent_id, parent_code, name, code, menu_type, status, is_system, is_manual,
             path, icon, sort_order, module_id, module_menu_code, is_active, created_at, updated_at
         )
         VALUES (0, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(code) WHERE deleted_at IS NULL DO UPDATE
         SET parent_code = EXCLUDED.parent_code,
             name = EXCLUDED.name,
             menu_type = EXCLUDED.menu_type,
             status = EXCLUDED.status,
             is_system = EXCLUDED.is_system,
             is_manual = EXCLUDED.is_manual,
             path = EXCLUDED.path,
             icon = EXCLUDED.icon,
             sort_order = EXCLUDED.sort_order,
             module_id = EXCLUDED.module_id,
             module_menu_code = EXCLUDED.module_menu_code,
             is_active = EXCLUDED.is_active,
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
    .bind(record.path.as_deref())
    .bind(record.icon.as_deref())
    .bind(record.sort_order)
    .bind(record.module_id.as_deref())
    .bind(record.module_menu_code.as_deref())
    .bind(record.is_active)
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

async fn expand_legacy_module_wildcard_grants(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    module: &str,
    capabilities: &[MenuSeedRecord],
) -> Result<(), ServiceError> {
    let wildcard = format!("{module}:*");
    let custom_role_ids = sqlx::query_scalar::<_, i64>(
        "SELECT DISTINCT rm.role_id
         FROM role_menus rm
         INNER JOIN roles r ON r.id = rm.role_id
             AND r.is_system = FALSE
             AND r.deleted_at IS NULL
         INNER JOIN menus m ON m.id = rm.menu_id
             AND m.deleted_at IS NULL
         WHERE m.code = ?",
    )
    .bind(&wildcard)
    .fetch_all(&mut **tx)
    .await
    .map_err(database_error("loading legacy module wildcard grants"))?;

    if custom_role_ids.is_empty() {
        return Ok(());
    }

    let mut capability_ids = Vec::with_capacity(capabilities.len());
    for capability in capabilities {
        let id = sqlx::query_scalar::<_, i64>(
            "SELECT id
             FROM menus
             WHERE module_id = ?
               AND code = ?
               AND is_active = TRUE
               AND deleted_at IS NULL",
        )
        .bind(module)
        .bind(&capability.permission_code)
        .fetch_one(&mut **tx)
        .await
        .map_err(database_error("loading a current module capability"))?;
        capability_ids.push(id);
    }

    if !capability_ids.is_empty() {
        let now = Utc::now().naive_utc();
        let grants = custom_role_ids
            .iter()
            .flat_map(|role_id| capability_ids.iter().map(move |menu_id| (*role_id, *menu_id)));
        let mut builder = sqlx::QueryBuilder::<Sqlite>::new(
            "INSERT OR IGNORE INTO role_menus (role_id, menu_id, created_at) ",
        );
        builder.push_values(grants, |mut row, (role_id, menu_id)| {
            row.push_bind(role_id).push_bind(menu_id).push_bind(now);
        });
        builder
            .build()
            .execute(&mut **tx)
            .await
            .map_err(database_error("expanding legacy module wildcard grants"))?;
    }

    sqlx::query(
        "DELETE FROM role_menus
         WHERE menu_id IN (
             SELECT id FROM menus WHERE code = ? AND deleted_at IS NULL
         )
           AND role_id IN (
             SELECT id FROM roles WHERE is_system = FALSE AND deleted_at IS NULL
         )",
    )
    .bind(&wildcard)
    .execute(&mut **tx)
    .await
    .map_err(database_error("retiring legacy module wildcard grants"))?;

    Ok(())
}

async fn sync_builtin_roles(tx: &mut sqlx::Transaction<'_, Sqlite>) -> Result<(), ServiceError> {
    upsert_all_capabilities_menu(tx).await?;

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
         VALUES (0, '全部权限', ?, ?, 1, ?, TRUE, FALSE, ?, ?)
         ON CONFLICT(code) WHERE deleted_at IS NULL DO UPDATE
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

async fn list_menu_code_rows(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
) -> Result<Vec<(i64, String)>, ServiceError> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT id, code
         FROM menus
         WHERE deleted_at IS NULL
           AND is_active = TRUE
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

async fn load_permission_cache_snapshot<'e, E>(
    executor: E,
) -> Result<PermissionCacheSnapshot, ServiceError>
where
    E: Executor<'e, Database = Sqlite>,
{
    let rows = sqlx::query_as::<_, (i64, Option<String>)>(
        "SELECT u.id, p.menu_code
         FROM users u
         LEFT JOIN user_permissions p ON p.user_id = u.id
         WHERE u.status = 1 AND u.deleted_at IS NULL
         ORDER BY u.id, p.menu_code",
    )
    .fetch_all(executor)
    .await
    .map_err(database_error("loading the permission cache snapshot"))?;
    let mut cache = HashMap::<i64, HashSet<String>>::new();
    for (user_id, permission) in rows {
        let entry = cache.entry(user_id).or_default();
        if let Some(permission) = permission {
            entry.insert(permission);
        }
    }
    Ok(cache.into_iter().map(|(user_id, permissions)| (user_id, Arc::new(permissions))).collect())
}

fn database_error(operation: &'static str) -> impl FnOnce(sqlx::Error) -> ServiceError + Copy {
    move |error| {
        tracing::error!(%error, operation, "Permission database operation failed");
        ServiceError::DatabaseQueryFailed
    }
}

fn builtin_role_permission_codes(role_code: &str, menu_codes: &[String]) -> Vec<String> {
    let policy = RolePolicy;
    menu_codes
        .iter()
        .filter(|code| policy.role_allows_capability(role_code, code))
        .cloned()
        .collect()
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
    use axum::http::Method;
    use rustzen_ipc::{MenuDefinition, ModuleManifest, RouteManifest};
    use tokio::sync::Barrier;

    use super::*;
    use crate::features::system::menu::{service::MenuService, types::UpdateMenuPayload};

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
        assert_eq!(permission_title("*"), "全部权限");
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
        assert_eq!(dashboard_parent.title, "仪表盘管理");
        assert_eq!(dashboard_parent.menu_type, MENU_TYPE_DIRECTORY);
        assert_eq!(dashboard_parent.parent_code, None);

        let dashboard_view = records
            .iter()
            .find(|record| record.permission_code == "dashboard:view")
            .expect("dashboard view");
        assert_eq!(dashboard_view.title, "仪表盘查看");
        assert_eq!(dashboard_view.menu_type, MENU_TYPE_MENU);
        assert_eq!(dashboard_view.parent_code.as_deref(), Some("dashboard:*"));

        let report_view = records
            .iter()
            .find(|record| record.permission_code == "report:view")
            .expect("report view");
        assert_eq!(report_view.title, "报表查看");
        assert_eq!(report_view.menu_type, MENU_TYPE_MENU);
        assert_eq!(report_view.parent_code.as_deref(), Some("report:*"));

        let user_group = records
            .iter()
            .find(|record| record.permission_code == "system:user:*")
            .expect("user group");
        assert_eq!(user_group.title, "系统用户管理");
        assert_eq!(user_group.menu_type, MENU_TYPE_DIRECTORY);
        assert_eq!(user_group.parent_code.as_deref(), Some("system:*"));

        let user_list = records
            .iter()
            .find(|record| record.permission_code == "system:user:list")
            .expect("user list");
        assert_eq!(user_list.title, "系统用户列表");
        assert_eq!(user_list.menu_type, MENU_TYPE_MENU);
        assert_eq!(user_list.parent_code.as_deref(), Some("system:user:*"));

        let log_export = records
            .iter()
            .find(|record| record.permission_code == "manage:log:export")
            .expect("log export");
        assert_eq!(log_export.title, "管理日志导出");
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
        ];

        let owner_codes = builtin_role_permission_codes(BUILTIN_OWNER_ROLE_CODE, &menu_codes);
        let admin_codes = builtin_role_permission_codes(BUILTIN_ADMIN_ROLE_CODE, &menu_codes);
        let viewer_codes = builtin_role_permission_codes(BUILTIN_VIEWER_ROLE_CODE, &menu_codes);

        assert_eq!(owner_codes, vec!["*".to_string()]);

        assert!(admin_codes.contains(&"system:user:create".to_string()));
        assert!(admin_codes.contains(&"manage:log:export".to_string()));
        assert!(!admin_codes.contains(&"manage:task:list".to_string()));
        assert!(!admin_codes.contains(&"manage:task:run".to_string()));
        assert!(!admin_codes.contains(&"manage:deploy:list".to_string()));
        assert!(!admin_codes.iter().any(|code| code == "*" || code.ends_with(":*")));
        assert!(!admin_codes.contains(&"manage:deploy:run".to_string()));

        assert!(viewer_codes.contains(&"system:user:list".to_string()));
        assert!(viewer_codes.contains(&"dashboard:view".to_string()));
        assert!(!viewer_codes.contains(&"manage:deploy:list".to_string()));
        assert!(!viewer_codes.contains(&"system:user:create".to_string()));
        assert!(!viewer_codes.contains(&"manage:task:run".to_string()));
        assert!(!viewer_codes.contains(&"manage:log:export".to_string()));
        assert!(!viewer_codes.iter().any(|code| code == "*" || code.ends_with(":*")));
        assert!(!viewer_codes.contains(&"manage:deploy:run".to_string()));
    }

    #[tokio::test]
    async fn sync_permissions_persists_builtin_roles_and_default_owner() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");

        sqlx::query(
            "INSERT INTO menus (
                 parent_id, name, code, menu_type, sort_order, status, is_system, is_manual,
                 is_active, created_at, updated_at
             ) VALUES
                 (0, '过期权限', 'manage:dict:options', 3, 1, 1, TRUE, TRUE, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                 (0, '当前手工覆盖', 'dashboard:view', 3, 1, 1, TRUE, TRUE, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                 (0, '手工权限', 'custom:manual:view', 3, 1, 1, FALSE, TRUE, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        )
        .execute(&pool)
        .await
        .expect("legacy and manual permissions");

        let stale_menu_id: i64 =
            sqlx::query_scalar("SELECT id FROM menus WHERE code = 'manage:dict:options'")
                .fetch_one(&pool)
                .await
                .expect("stale menu id");
        let custom_role_id: i64 = sqlx::query_scalar(
            "INSERT INTO roles (name, code, status, is_system)
             VALUES ('Legacy dictionary role', 'legacy_dictionary', 1, FALSE)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("legacy custom role");
        let custom_user_id: i64 = sqlx::query_scalar(
            "INSERT INTO users (username, email, password_hash, status)
             VALUES ('legacy-dictionary-user', 'legacy-dictionary@example.com', 'hash', 1)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("legacy custom user");
        sqlx::query("INSERT INTO role_menus (role_id, menu_id) VALUES (?, ?)")
            .bind(custom_role_id)
            .bind(stale_menu_id)
            .execute(&pool)
            .await
            .expect("legacy dictionary grant");
        sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
            .bind(custom_user_id)
            .bind(custom_role_id)
            .execute(&pool)
            .await
            .expect("legacy dictionary user role");

        let seeded_accounts = sqlx::query_as::<_, (String, String)>(
            "SELECT u.username, r.code
             FROM users u
             INNER JOIN user_roles ur ON ur.user_id = u.id
             INNER JOIN roles r ON r.id = ur.role_id
             WHERE u.is_system = TRUE
             ORDER BY u.username",
        )
        .fetch_all(&pool)
        .await
        .expect("seeded accounts");
        assert_eq!(
            seeded_accounts,
            vec![
                ("admin".to_string(), "admin".to_string()),
                ("owner".to_string(), "owner".to_string()),
                ("viewer".to_string(), "viewer".to_string()),
            ]
        );

        rustzen_auth::permission::register_permission_codes([
            "dashboard:view",
            "system:user:list",
            "system:user:create",
            "manage:task:list",
            "manage:task:run",
            "manage:deploy:list",
            "manage:deploy:run",
        ]);

        PermissionService::sync_permissions(&pool).await.expect("permission sync");

        let owner_permissions = role_permission_codes(&pool, BUILTIN_OWNER_ROLE_CODE).await;
        let admin_permissions = role_permission_codes(&pool, BUILTIN_ADMIN_ROLE_CODE).await;
        let viewer_permissions = role_permission_codes(&pool, BUILTIN_VIEWER_ROLE_CODE).await;

        assert_eq!(owner_permissions, vec!["*".to_string()]);
        assert!(admin_permissions.contains(&"system:user:create".to_string()));
        assert!(!admin_permissions.contains(&"manage:task:list".to_string()));
        assert!(!admin_permissions.contains(&"manage:task:run".to_string()));
        assert!(!admin_permissions.contains(&"manage:deploy:list".to_string()));
        assert!(!admin_permissions.contains(&"manage:deploy:run".to_string()));
        assert!(!admin_permissions.iter().any(|code| code == "*" || code.ends_with(":*")));

        assert!(viewer_permissions.contains(&"dashboard:view".to_string()));
        assert!(viewer_permissions.contains(&"system:user:list".to_string()));
        assert!(!viewer_permissions.contains(&"manage:deploy:list".to_string()));
        assert!(!viewer_permissions.contains(&"system:user:create".to_string()));
        assert!(!viewer_permissions.contains(&"manage:task:run".to_string()));
        assert!(!viewer_permissions.contains(&"manage:deploy:run".to_string()));
        assert!(!viewer_permissions.iter().any(|code| code == "*" || code.ends_with(":*")));

        let stale_core_active: bool =
            sqlx::query_scalar("SELECT is_active FROM menus WHERE code = 'manage:dict:options'")
                .fetch_one(&pool)
                .await
                .expect("stale core permission");
        let manual_active: bool =
            sqlx::query_scalar("SELECT is_active FROM menus WHERE code = 'custom:manual:view'")
                .fetch_one(&pool)
                .await
                .expect("manual permission");
        let current_core_active: bool =
            sqlx::query_scalar("SELECT is_active FROM menus WHERE code = 'dashboard:view'")
                .fetch_one(&pool)
                .await
                .expect("current core permission");
        assert!(!stale_core_active);
        assert!(manual_active);
        assert!(current_core_active);
        let current_core_name: String =
            sqlx::query_scalar("SELECT name FROM menus WHERE code = 'dashboard:view'")
                .fetch_one(&pool)
                .await
                .expect("current manual override");
        assert_eq!(current_core_name, "当前手工覆盖");

        let stale_effective_permissions = sqlx::query_scalar::<_, String>(
            "SELECT menu_code FROM user_permissions WHERE user_id = ? ORDER BY menu_code",
        )
        .bind(custom_user_id)
        .fetch_all(&pool)
        .await
        .expect("legacy dictionary effective permissions");
        assert!(stale_effective_permissions.is_empty());
        let cached_user =
            PermissionService::load_current_user(custom_user_id, "legacy-dictionary-user")
                .expect("legacy user permission cache");
        assert!(!cached_user.has_capability("manage:dict:options"));
        PermissionService::clear_user_cache(custom_user_id);

        let owner_wildcard_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM user_permissions
             WHERE username = ? AND menu_code = ?",
        )
        .bind(DEFAULT_OWNER_USERNAME)
        .bind(SYSTEM_WILDCARD)
        .fetch_one(&pool)
        .await
        .expect("owner permissions");
        assert_eq!(owner_wildcard_count, 1);
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

    #[tokio::test]
    async fn legacy_module_wildcard_grants_expand_once_to_current_manifest_capabilities() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");

        let wildcard_id: i64 = sqlx::query_scalar(
            "INSERT INTO menus (
                parent_id, name, code, menu_type, status, is_system, is_manual,
                module_id, is_active
             ) VALUES (0, 'Legacy Monitor wildcard', 'monitor:*', 3, 1, TRUE, FALSE,
                'monitor', FALSE)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("legacy wildcard menu");
        let custom_role: i64 = sqlx::query_scalar(
            "INSERT INTO roles (name, code, status, is_system)
             VALUES ('Legacy monitor role', 'legacy_monitor', 1, FALSE)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("legacy custom role");
        sqlx::query("INSERT INTO role_menus (role_id, menu_id) VALUES (?, ?)")
            .bind(custom_role)
            .bind(wildcard_id)
            .execute(&pool)
            .await
            .expect("legacy wildcard grant");
        let user_id: i64 = sqlx::query_scalar(
            "INSERT INTO users (username, email, password_hash, status)
             VALUES ('legacy-module-user', 'legacy-module@example.com', 'hash', 1)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("legacy module user");
        sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(custom_role)
            .execute(&pool)
            .await
            .expect("legacy user role");

        let initial = monitor_manifest(
            &["monitor:view", "monitor:manage"],
            "monitor:view",
            "Monitor",
            "/monitor",
            "monitor",
            10,
        );
        PermissionService::reconcile_module_manifest(&pool, &initial)
            .await
            .expect("first post-upgrade reconcile");
        assert_eq!(
            role_permission_codes(&pool, "legacy_monitor").await,
            vec!["monitor:manage".to_string(), "monitor:view".to_string()]
        );
        let effective_permissions = sqlx::query_scalar::<_, String>(
            "SELECT menu_code FROM user_permissions WHERE user_id = ? ORDER BY menu_code",
        )
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .expect("effective migrated permissions");
        assert_eq!(
            effective_permissions,
            vec!["monitor:manage".to_string(), "monitor:view".to_string()]
        );

        let changed = monitor_manifest(
            &["monitor:view", "monitor:manage", "monitor:restart"],
            "monitor:view",
            "Monitor",
            "/monitor",
            "monitor",
            10,
        );
        PermissionService::reconcile_module_manifest(&pool, &changed)
            .await
            .expect("later Manifest reconcile");
        let grants = role_permission_codes(&pool, "legacy_monitor").await;
        assert!(grants.contains(&"monitor:view".to_string()));
        assert!(grants.contains(&"monitor:manage".to_string()));
        assert!(!grants.contains(&"monitor:restart".to_string()));
        assert!(!grants.contains(&"monitor:*".to_string()));
    }

    #[tokio::test]
    async fn module_reconciliation_preserves_manual_overrides_and_custom_grants() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");

        let initial = monitor_manifest(
            &["monitor:view", "monitor:manage"],
            "monitor:view",
            "Monitor",
            "/monitor",
            "monitor",
            10,
        );
        PermissionService::reconcile_module_manifest(&pool, &initial)
            .await
            .expect("initial reconcile");
        let wildcard_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM menus WHERE code = 'monitor:*'")
                .fetch_one(&pool)
                .await
                .expect("wildcard count");
        assert_eq!(wildcard_count, 0, "Manifest reconciliation must persist exact capabilities");

        let view_id: i64 = sqlx::query_scalar("SELECT id FROM menus WHERE code = 'monitor:view'")
            .fetch_one(&pool)
            .await
            .expect("view menu");
        let custom_role: i64 = sqlx::query_scalar(
            "INSERT INTO roles (name, code, status) VALUES ('Monitor custom', 'monitor_custom', 1) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("custom role");
        sqlx::query("INSERT INTO role_menus (role_id, menu_id) VALUES (?, ?)")
            .bind(custom_role)
            .bind(view_id)
            .execute(&pool)
            .await
            .expect("custom grant");
        let user_id: i64 = sqlx::query_scalar(
            "INSERT INTO users (username, email, password_hash, status) VALUES ('module-user', 'module@example.com', 'hash', 1) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("custom user");
        sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(custom_role)
            .execute(&pool)
            .await
            .expect("user role");
        sqlx::query(
            "UPDATE menus SET name = 'Custom Monitor', icon = 'custom-icon', sort_order = 77,
                    status = 2, is_manual = TRUE WHERE id = ?",
        )
        .bind(view_id)
        .execute(&pool)
        .await
        .expect("manual override");
        crate::features::system::menu::repo::MenuRepository::update_module_override(
            &pool,
            "monitor",
            "monitor",
            "Custom Monitor",
            None,
            77,
            2,
        )
        .await
        .expect("null icon preserves the current presentation");

        let changed = monitor_manifest(
            &["monitor:view", "monitor:manage", "monitor:restart"],
            "monitor:view",
            "Changed default",
            "/monitor-v2",
            "changed-icon",
            1,
        );
        PermissionService::reconcile_module_manifest(&pool, &changed)
            .await
            .expect("changed reconcile");
        let row: (String, Option<String>, i32, i16, Option<String>, bool, bool) = sqlx::query_as(
            "SELECT name, icon, sort_order, status, path, is_manual, is_active
                 FROM menus WHERE id = ?",
        )
        .bind(view_id)
        .fetch_one(&pool)
        .await
        .expect("manual row");
        assert_eq!(
            row,
            (
                "Custom Monitor".to_string(),
                Some("custom-icon".to_string()),
                77,
                2,
                Some("/monitor-v2".to_string()),
                true,
                true,
            )
        );
        let permission_while_hidden: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_permissions WHERE user_id = ? AND menu_code = 'monitor:view'",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("hidden permission");
        assert_eq!(permission_while_hidden, 1, "menu visibility is not authorization");
        let custom_restart: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM role_menus rm
             INNER JOIN menus m ON m.id = rm.menu_id
             WHERE rm.role_id = ? AND m.code = 'monitor:restart'",
        )
        .bind(custom_role)
        .fetch_one(&pool)
        .await
        .expect("new custom grant");
        assert_eq!(custom_restart, 0, "new capabilities must not reach custom roles");
        assert!(
            role_permission_codes(&pool, BUILTIN_ADMIN_ROLE_CODE)
                .await
                .contains(&"monitor:restart".to_string())
        );
        assert!(
            !role_permission_codes(&pool, BUILTIN_VIEWER_ROLE_CODE)
                .await
                .contains(&"monitor:restart".to_string())
        );

        let removed = monitor_manifest(
            &["monitor:manage", "monitor:restart"],
            "monitor:manage",
            "Monitor",
            "/monitor",
            "monitor",
            10,
        );
        PermissionService::reconcile_module_manifest(&pool, &removed)
            .await
            .expect("removed reconcile");
        let active: bool = sqlx::query_scalar("SELECT is_active FROM menus WHERE id = ?")
            .bind(view_id)
            .fetch_one(&pool)
            .await
            .expect("inactive old capability");
        assert!(!active);
        let retained_grant: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM role_menus WHERE role_id = ? AND menu_id = ?")
                .bind(custom_role)
                .bind(view_id)
                .fetch_one(&pool)
                .await
                .expect("retained grant");
        assert_eq!(retained_grant, 1);
        let effective_grant: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM user_permissions WHERE user_id = ? AND menu_code = 'monitor:view'",
        )
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("effective removed grant");
        assert_eq!(effective_grant, 0);
        let transferred: (String, Option<String>, i32, i16, Option<String>, bool) = sqlx::query_as(
            "SELECT name, icon, sort_order, status, path, is_manual
                 FROM menus WHERE code = 'monitor:manage' AND is_active = TRUE",
        )
        .fetch_one(&pool)
        .await
        .expect("transferred module menu override");
        assert_eq!(
            transferred,
            (
                "Custom Monitor".to_string(),
                Some("custom-icon".to_string()),
                77,
                2,
                Some("/monitor".to_string()),
                true,
            )
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn module_menu_edits_never_succeed_against_an_inactive_manifest_row() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");
        let initial = monitor_manifest(
            &["monitor:view", "monitor:manage"],
            "monitor:view",
            "Monitor",
            "/monitor",
            "monitor",
            10,
        );
        PermissionService::reconcile_module_manifest(&pool, &initial)
            .await
            .expect("initial reconcile");
        let old_id: i64 = sqlx::query_scalar("SELECT id FROM menus WHERE code = 'monitor:view'")
            .fetch_one(&pool)
            .await
            .expect("old menu row");
        let override_request = UpdateMenuPayload {
            name: "Operations Monitor".to_string(),
            sort_order: 42,
            status: 2,
            icon: Some("activity".to_string()),
        };

        MenuService::update_menu(&pool, old_id, override_request.clone())
            .await
            .expect("edit before Manifest permission change");
        let changed = monitor_manifest(
            &["monitor:view", "monitor:manage"],
            "monitor:manage",
            "Changed default",
            "/monitor-v2",
            "server",
            1,
        );
        PermissionService::reconcile_module_manifest(&pool, &changed)
            .await
            .expect("changed reconcile");
        let active_after_edit_first: (String, String, i32, i16) = sqlx::query_as(
            "SELECT code, name, sort_order, status FROM menus
             WHERE module_id = 'monitor' AND module_menu_code = 'monitor' AND is_active = TRUE",
        )
        .fetch_one(&pool)
        .await
        .expect("active menu after edit-first ordering");
        assert_eq!(
            active_after_edit_first,
            ("monitor:manage".to_string(), "Operations Monitor".to_string(), 42, 2)
        );
        let inactive_row_before: (String, String) =
            sqlx::query_as("SELECT code, name FROM menus WHERE id = ?")
                .bind(old_id)
                .fetch_one(&pool)
                .await
                .expect("inactive row before rejected edit");
        assert!(matches!(
            MenuService::update_menu(&pool, old_id, override_request.clone()).await,
            Err(ServiceError::InvalidOperation(_))
        ));
        let inactive_row_after: (String, String) =
            sqlx::query_as("SELECT code, name FROM menus WHERE id = ?")
                .bind(old_id)
                .fetch_one(&pool)
                .await
                .expect("inactive row after rejected edit");
        assert_eq!(inactive_row_after, inactive_row_before);

        let race_pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("race sqlite pool");
        crate::infra::db::run_migrations(&race_pool).await.expect("race migrations");
        PermissionService::reconcile_module_manifest(&race_pool, &initial)
            .await
            .expect("race initial reconcile");
        let race_old_id: i64 =
            sqlx::query_scalar("SELECT id FROM menus WHERE code = 'monitor:view'")
                .fetch_one(&race_pool)
                .await
                .expect("race old menu row");
        let start = Arc::new(Barrier::new(3));
        let edit_pool = race_pool.clone();
        let edit_start = Arc::clone(&start);
        let edit_request = override_request;
        let edit = tokio::spawn(async move {
            edit_start.wait().await;
            MenuService::update_menu(&edit_pool, race_old_id, edit_request).await
        });
        let reconcile_pool = race_pool.clone();
        let reconcile_start = Arc::clone(&start);
        let reconcile = tokio::spawn(async move {
            reconcile_start.wait().await;
            PermissionService::reconcile_module_manifest(&reconcile_pool, &changed).await
        });
        start.wait().await;
        let edit_result = edit.await.expect("edit task");
        reconcile.await.expect("reconcile task").expect("race reconcile");

        let active_after_race: (String, String) = sqlx::query_as(
            "SELECT code, name FROM menus
             WHERE module_id = 'monitor' AND module_menu_code = 'monitor' AND is_active = TRUE",
        )
        .fetch_one(&race_pool)
        .await
        .expect("active menu after race");
        match edit_result {
            Ok(_) => assert_eq!(
                active_after_race,
                ("monitor:manage".to_string(), "Operations Monitor".to_string())
            ),
            Err(ServiceError::NotFound(_) | ServiceError::InvalidOperation(_)) => assert_eq!(
                active_after_race,
                ("monitor:manage".to_string(), "Changed default".to_string())
            ),
            Err(error) => panic!("unexpected concurrent edit error: {error}"),
        }
    }

    #[tokio::test]
    async fn module_menu_names_are_scoped_independently_from_manual_menus() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");
        sqlx::query(
            "INSERT INTO menus (name, code, menu_type, status, is_system, is_manual)
             VALUES ('Monitor', 'custom:monitor:list', 2, 1, FALSE, TRUE)",
        )
        .execute(&pool)
        .await
        .expect("conflicting manual menu");

        let manifest = monitor_manifest(
            &["monitor:view"],
            "monitor:view",
            "Monitor",
            "/monitor",
            "monitor",
            10,
        );
        PermissionService::reconcile_module_manifest(&pool, &manifest)
            .await
            .expect("module menu may reuse a title in its own navigation group");
        let module_rows: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM menus WHERE module_id = 'monitor'")
                .fetch_one(&pool)
                .await
                .expect("module rows after rollback");
        assert_eq!(module_rows, 1);
        let custom_active: bool =
            sqlx::query_scalar("SELECT is_active FROM menus WHERE code = 'custom:monitor:list'")
                .fetch_one(&pool)
                .await
                .expect("custom menu remains");
        assert!(custom_active);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_user_cache_fill_cannot_restore_a_revoked_capability() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");
        let menu_id: i64 = sqlx::query_scalar(
            "INSERT INTO menus (name, code, menu_type, status, is_system, is_manual)
             VALUES ('Concurrent capability', 'custom:concurrent:view', 2, 1, FALSE, TRUE)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("menu");
        let role_id: i64 = sqlx::query_scalar(
            "INSERT INTO roles (name, code, status) VALUES ('Concurrent role', 'concurrent_role', 1)
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("role");
        let user_id: i64 = sqlx::query_scalar(
            "INSERT INTO users (username, email, password_hash, status)
             VALUES ('concurrent-user', 'concurrent@example.com', 'hash', 1) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("user");
        sqlx::query("INSERT INTO role_menus (role_id, menu_id) VALUES (?, ?)")
            .bind(role_id)
            .bind(menu_id)
            .execute(&pool)
            .await
            .expect("role menu");
        sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
            .bind(user_id)
            .bind(role_id)
            .execute(&pool)
            .await
            .expect("user role");
        PermissionService::refresh_all_user_permissions(&pool).await.expect("initial cache");

        let start = Arc::new(tokio::sync::Barrier::new(2));
        let fill_pool = pool.clone();
        let fill_start = Arc::clone(&start);
        let fill = tokio::spawn(async move {
            fill_start.wait().await;
            for _ in 0..32 {
                PermissionService::refresh_user_permissions(&fill_pool, user_id)
                    .await
                    .expect("concurrent user fill");
                tokio::task::yield_now().await;
            }
        });
        start.wait().await;
        sqlx::query("DELETE FROM role_menus WHERE role_id = ? AND menu_id = ?")
            .bind(role_id)
            .bind(menu_id)
            .execute(&pool)
            .await
            .expect("revoke capability");
        PermissionService::refresh_all_user_permissions(&pool).await.expect("revoked snapshot");
        fill.await.expect("fill task");

        let user = PermissionService::load_current_user(user_id, "concurrent-user")
            .expect("cached active user");
        assert!(!user.permissions.contains("custom:concurrent:view"));
        PermissionService::clear_user_cache(user_id);
    }

    fn monitor_manifest(
        capabilities: &[&str],
        menu_permission: &str,
        title: &str,
        path: &str,
        icon: &str,
        sort_order: i32,
    ) -> ModuleManifest {
        ModuleManifest {
            module: "monitor".to_string(),
            name: "Monitor".to_string(),
            api_prefix: "/api/monitor".to_string(),
            contract_version: rustzen_ipc::CONTRACT_VERSION,
            release_version: env!("CARGO_PKG_VERSION").to_string(),
            menus: vec![MenuDefinition {
                code: "monitor".to_string(),
                title: title.to_string(),
                path: path.to_string(),
                icon: icon.to_string(),
                sort_order,
                permission: menu_permission.to_string(),
            }],
            routes: capabilities
                .iter()
                .map(|capability| {
                    RouteManifest::protected(
                        Method::GET,
                        format!("/{}", capability.replace(':', "-")),
                        *capability,
                    )
                })
                .collect(),
        }
    }
}
