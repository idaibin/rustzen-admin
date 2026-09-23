use super::*;

pub(super) async fn sync_builtin_roles(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
) -> Result<(), ServiceError> {
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

pub(super) async fn load_permission_cache_snapshot<'e, E>(
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

pub(super) fn database_error(
    operation: &'static str,
) -> impl FnOnce(sqlx::Error) -> ServiceError + Copy {
    move |error| {
        tracing::error!(%error, operation, "Permission database operation failed");
        ServiceError::DatabaseQueryFailed
    }
}

pub(super) fn builtin_role_permission_codes(role_code: &str, menu_codes: &[String]) -> Vec<String> {
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
