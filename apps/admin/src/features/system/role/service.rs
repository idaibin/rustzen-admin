use super::{
    repo::{RoleRepository, SoftDeleteOutcome},
    types::{
        CreateRoleRequest, RoleItemResp, RoleListQuery, RoleOptionResp, RoleQuery,
        UpdateRolePayload,
    },
};
use crate::common::{
    api::OptionsQuery,
    error::ServiceError,
    pagination::{Pagination, PaginationQuery},
    query::parse_optional_i16_filter,
};
use rustzen_auth::capability::{RolePolicy, SYSTEM_WILDCARD};

use sqlx::SqlitePool;

const OWNER_ROLE_CODE: &str = "owner";
const BUILTIN_ROLE_CODES: &[&str] = &["owner", "admin", "viewer"];

pub struct RoleService;

impl RoleService {
    /// Get paginated role list with filtering
    pub async fn list_roles(
        pool: &SqlitePool,
        query: RoleQuery,
    ) -> Result<(Vec<RoleItemResp>, i64), ServiceError> {
        tracing::info!("Fetching role list with query: {:?}", query);

        let RoleQuery { current, page_size, role_name, role_code, status } = query;
        let pagination = Pagination::from_query(PaginationQuery { current, page_size });
        let limit = i64::from(pagination.limit);
        let offset = i64::from(pagination.offset);
        let status = parse_optional_i16_filter(status.as_deref(), "role status", None)?;
        let repo_query = RoleListQuery { role_name, role_code, status };

        let (roles, total) = RoleRepository::list_roles(pool, offset, limit, repo_query).await?;

        Ok((roles.into_iter().map(RoleItemResp::try_from).collect::<Result<Vec<_>, _>>()?, total))
    }

    /// Create new role with validation
    pub async fn create_role(
        pool: &SqlitePool,
        _current_user_id: i64,
        request: CreateRoleRequest,
    ) -> Result<(), ServiceError> {
        tracing::info!("Creating role: {}", request.name);
        ensure_builtin_role_code_is_reserved(&request.code)?;
        Self::ensure_role_menus_are_assignable(pool, &request.menu_ids).await?;
        RoleRepository::create(
            pool,
            &request.name,
            &request.code,
            request.description.as_deref(),
            request.status,
            &request.menu_ids,
        )
        .await?;
        crate::infra::permission::PermissionService::refresh_all_user_permissions(pool).await?;
        Ok(())
    }

    /// Update existing role with validation
    pub async fn update_role(
        pool: &SqlitePool,
        id: i64,
        _current_user_id: i64,
        request: UpdateRolePayload,
    ) -> Result<(), ServiceError> {
        tracing::info!("Updating role: {}", id);
        Self::ensure_role_is_mutable(pool, id).await?;
        ensure_builtin_role_code_is_reserved(&request.code)?;
        Self::ensure_role_menus_are_assignable(pool, &request.menu_ids).await?;
        RoleRepository::update(
            pool,
            id,
            &request.name,
            &request.code,
            request.description.as_deref(),
            request.status,
            &request.menu_ids,
        )
        .await?;
        crate::infra::permission::PermissionService::refresh_all_user_permissions(pool).await?;
        Ok(())
    }

    /// Delete role with user assignment validation
    pub async fn delete_role(
        pool: &SqlitePool,
        id: i64,
        _current_user_id: i64,
    ) -> Result<(), ServiceError> {
        tracing::info!("Attempting to delete role: {}", id);
        Self::ensure_role_is_mutable(pool, id).await?;

        // Check if role is still assigned to users
        let user_count = RoleRepository::get_role_user_count(pool, id).await?;
        if user_count > 0 {
            tracing::warn!("Cannot delete role {} - still assigned to {} users", id, user_count);
            return Err(ServiceError::InvalidOperation(format!(
                "Cannot delete role '{}' - it is still assigned to {} user(s). Please remove all user assignments before deleting the role.",
                id, user_count
            )));
        }

        // Perform the deletion
        match RoleRepository::soft_delete(pool, id).await? {
            SoftDeleteOutcome::Deleted => {
                crate::infra::permission::PermissionService::refresh_all_user_permissions(pool)
                    .await?;
                tracing::info!("Successfully deleted role: {}", id);
                Ok(())
            }
            SoftDeleteOutcome::AssignmentBlocked(user_count) => {
                tracing::warn!(
                    "Cannot delete role {} - still assigned to {} users",
                    id,
                    user_count
                );
                Err(ServiceError::InvalidOperation(format!(
                    "Cannot delete role '{}' - it is still assigned to {} user(s). Please remove all user assignments before deleting the role.",
                    id, user_count
                )))
            }
            SoftDeleteOutcome::NotFound => {
                tracing::warn!("Role not found during deletion: {}", id);
                Err(ServiceError::NotFound("Role".to_string()))
            }
        }
    }

    async fn ensure_role_is_mutable(pool: &SqlitePool, id: i64) -> Result<(), ServiceError> {
        match RoleRepository::get_role_identity(pool, id).await? {
            Some((code, is_system)) => ensure_role_identity_is_mutable(&code, is_system),
            None => Err(ServiceError::NotFound(format!("Role id: {}", id))),
        }
    }

    async fn ensure_role_menus_are_assignable(
        pool: &SqlitePool,
        menu_ids: &[i64],
    ) -> Result<(), ServiceError> {
        ensure_role_has_permissions(menu_ids)?;
        let menu_codes = RoleRepository::list_menu_codes_by_ids(pool, menu_ids).await?;
        ensure_menu_codes_assignable(&menu_codes)
    }

    /// Get role options for dropdowns
    pub async fn get_role_options(
        pool: &SqlitePool,
        current_user_id: i64,
        query: OptionsQuery,
    ) -> Result<Vec<RoleOptionResp>, ServiceError> {
        tracing::info!("Retrieving role options: {:?}", query);
        let can_assign_owner = crate::infra::permission::PermissionService::has_permission(
            current_user_id,
            SYSTEM_WILDCARD,
        )
        .await?;
        Ok(RoleRepository::list_role_options(pool, query.q.as_deref(), query.limit)
            .await?
            .into_iter()
            .filter(|(_, _, code, _)| can_assign_owner || code != OWNER_ROLE_CODE)
            .map(|(id, name, code, is_system)| RoleOptionResp {
                label: name,
                value: id,
                code,
                is_system,
            })
            .collect())
    }
}

fn ensure_role_has_permissions(menu_ids: &[i64]) -> Result<(), ServiceError> {
    if menu_ids.is_empty() {
        return Err(ServiceError::InvalidOperation(
            "A custom role requires at least one permission.".to_string(),
        ));
    }
    Ok(())
}

fn ensure_menu_codes_assignable(menu_codes: &[String]) -> Result<(), ServiceError> {
    if let Some(code) = menu_codes.iter().find(|code| is_reserved_role_menu_code(code)) {
        return Err(ServiceError::InvalidOperation(format!(
            "Permission '{}' can only be assigned to the built-in owner role.",
            code
        )));
    }

    Ok(())
}

fn ensure_role_identity_is_mutable(role_code: &str, is_system: bool) -> Result<(), ServiceError> {
    if role_code == OWNER_ROLE_CODE {
        return Err(ServiceError::InvalidOperation(
            "Owner role cannot be modified or deleted.".to_string(),
        ));
    }

    if is_system {
        return Err(ServiceError::RoleIsSystem);
    }

    Ok(())
}

fn ensure_builtin_role_code_is_reserved(role_code: &str) -> Result<(), ServiceError> {
    if BUILTIN_ROLE_CODES.contains(&role_code) {
        return Err(ServiceError::InvalidOperation(format!(
            "Built-in role code '{}' is reserved.",
            role_code
        )));
    }

    Ok(())
}

fn is_reserved_role_menu_code(code: &str) -> bool {
    code == SYSTEM_WILDCARD || RolePolicy.is_owner_only_capability_or_wildcard(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_roles_cannot_assign_owner_only_capabilities() {
        assert!(ensure_menu_codes_assignable(&["system:user:list".to_string()]).is_ok());

        let wildcard_error = ensure_menu_codes_assignable(&["*".to_string()])
            .expect_err("wildcard should be reserved");
        assert!(matches!(wildcard_error, ServiceError::InvalidOperation(_)));

        for code in [
            "system:module:list",
            "system:status:view",
            "manage:task:list",
            "manage:deploy:list",
            "system:*",
            "manage:*",
        ] {
            let error = ensure_menu_codes_assignable(&[code.to_string()])
                .expect_err("owner-only capability should be reserved");
            assert!(matches!(error, ServiceError::InvalidOperation(_)));
        }
    }

    #[test]
    fn custom_roles_require_at_least_one_permission() {
        let error = ensure_role_has_permissions(&[]).expect_err("empty role should be rejected");
        assert!(matches!(error, ServiceError::InvalidOperation(_)));
        assert!(ensure_role_has_permissions(&[1]).is_ok());
    }

    #[test]
    fn builtin_role_codes_cannot_be_used_by_generic_role_forms() {
        let reserved_error =
            ensure_builtin_role_code_is_reserved(OWNER_ROLE_CODE).expect_err("owner is reserved");
        assert!(matches!(reserved_error, ServiceError::InvalidOperation(_)));

        assert!(ensure_builtin_role_code_is_reserved("ops_viewer").is_ok());
    }

    #[test]
    fn builtin_roles_cannot_be_modified_or_deleted() {
        let owner_error = ensure_role_identity_is_mutable(OWNER_ROLE_CODE, true)
            .expect_err("owner role should be immutable");
        assert!(matches!(owner_error, ServiceError::InvalidOperation(_)));

        assert!(matches!(
            ensure_role_identity_is_mutable("admin", true),
            Err(ServiceError::RoleIsSystem)
        ));
        assert!(ensure_role_identity_is_mutable("ops_viewer", false).is_ok());
    }

    #[tokio::test]
    async fn role_options_only_include_roles_assignable_by_the_current_user() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");

        let admin_user_id = 90_001;
        crate::infra::permission::PermissionService::cache_user_permissions(
            admin_user_id,
            &["system:role:options".to_string()],
        );
        let admin_options = RoleService::get_role_options(
            &pool,
            admin_user_id,
            OptionsQuery { q: None, limit: None },
        )
        .await
        .expect("admin options");
        assert!(!admin_options.iter().any(|role| role.code == OWNER_ROLE_CODE));

        let owner_user_id = 90_002;
        crate::infra::permission::PermissionService::cache_user_permissions(
            owner_user_id,
            &[SYSTEM_WILDCARD.to_string()],
        );
        let owner_options = RoleService::get_role_options(
            &pool,
            owner_user_id,
            OptionsQuery { q: None, limit: None },
        )
        .await
        .expect("owner options");
        assert!(owner_options.iter().any(|role| role.code == OWNER_ROLE_CODE));
    }
}
