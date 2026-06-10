use super::{
    repo::RoleRepository,
    types::{CreateRoleRequest, RoleItemResp, RoleListQuery, RoleQuery, UpdateRolePayload},
};
use crate::common::{
    api::{OptionItem, OptionsQuery},
    error::ServiceError,
    pagination::{Pagination, PaginationQuery},
    query::parse_optional_i16_filter,
};
use rustzen_core::capability::{SYSTEM_WILDCARD, is_deploy_capability_code};

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
        let success = RoleRepository::soft_delete(pool, id).await?;

        if success {
            tracing::info!("Successfully deleted role: {}", id);
            Ok(())
        } else {
            tracing::warn!("Role not found during deletion: {}", id);
            Err(ServiceError::NotFound("Role".to_string()))
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
        let menu_codes = RoleRepository::list_menu_codes_by_ids(pool, menu_ids).await?;
        ensure_menu_codes_assignable(&menu_codes)
    }

    /// Get role options for dropdowns
    pub async fn get_role_options(
        pool: &SqlitePool,
        query: OptionsQuery,
    ) -> Result<Vec<OptionItem<i64>>, ServiceError> {
        tracing::info!("Retrieving role options: {:?}", query);
        Ok(RoleRepository::list_role_options(pool, query.q.as_deref(), query.limit)
            .await?
            .into_iter()
            .map(|(id, name)| OptionItem { label: name, value: id })
            .collect())
    }
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
    code == SYSTEM_WILDCARD || is_deploy_capability_code(code) || wildcard_covers_deploy(code)
}

fn wildcard_covers_deploy(code: &str) -> bool {
    if !code.ends_with(":*") {
        return false;
    }

    let wildcard_prefix = code.trim_end_matches('*');
    "manage:deploy:".starts_with(wildcard_prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_roles_cannot_assign_wildcard_or_deploy_capabilities() {
        assert!(ensure_menu_codes_assignable(&["system:user:list".to_string()]).is_ok());

        let wildcard_error = ensure_menu_codes_assignable(&["*".to_string()])
            .expect_err("wildcard should be reserved");
        assert!(matches!(wildcard_error, ServiceError::InvalidOperation(_)));

        let deploy_error = ensure_menu_codes_assignable(&["manage:deploy:list".to_string()])
            .expect_err("deploy should be reserved");
        assert!(matches!(deploy_error, ServiceError::InvalidOperation(_)));

        let manage_wildcard_error = ensure_menu_codes_assignable(&["manage:*".to_string()])
            .expect_err("manage wildcard should cover deploy");
        assert!(matches!(manage_wildcard_error, ServiceError::InvalidOperation(_)));
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
}
