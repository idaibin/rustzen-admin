use std::collections::BTreeMap;

use sqlx::SqlitePool;

use crate::common::error::ServiceError;

use super::types::RuntimeMenuResponse;

pub struct ModuleRepository;

impl ModuleRepository {
    pub async fn load_enabled(pool: &SqlitePool) -> Result<BTreeMap<String, bool>, ServiceError> {
        let rows =
            sqlx::query_as::<_, (String, bool)>("SELECT id, enabled FROM modules ORDER BY id")
                .fetch_all(pool)
                .await
                .map_err(database_error("loading fixed module state"))?;
        Ok(rows.into_iter().collect())
    }

    #[cfg(feature = "full")]
    pub async fn set_enabled(
        pool: &SqlitePool,
        module: &str,
        enabled: bool,
    ) -> Result<(), ServiceError> {
        let updated = sqlx::query_scalar::<_, String>(
            "UPDATE modules SET enabled = ? WHERE id = ? RETURNING id",
        )
        .bind(enabled)
        .bind(module)
        .fetch_optional(pool)
        .await
        .map_err(database_error("updating fixed module state"))?;
        if updated.is_none() {
            return Err(ServiceError::NotFound(format!("Module {module}")));
        }
        Ok(())
    }

    pub async fn list_navigation(
        pool: &SqlitePool,
    ) -> Result<Vec<RuntimeMenuResponse>, ServiceError> {
        sqlx::query_as::<_, NavigationMenuRow>(
            "SELECT module_id, module_menu_code, name, path, icon, sort_order, code
             FROM module_navigation
             WHERE module_id IS NOT NULL
               AND module_menu_code IS NOT NULL
               AND path IS NOT NULL
               AND icon IS NOT NULL
               AND is_active = TRUE
               AND status = 1
             ORDER BY sort_order, module_menu_code",
        )
        .fetch_all(pool)
        .await
        .map(|rows| rows.into_iter().map(RuntimeMenuResponse::from).collect())
        .map_err(database_error("loading effective module navigation"))
    }
}

#[derive(sqlx::FromRow)]
struct NavigationMenuRow {
    module_id: String,
    module_menu_code: String,
    name: String,
    path: String,
    icon: String,
    sort_order: i32,
    code: String,
}

impl From<NavigationMenuRow> for RuntimeMenuResponse {
    fn from(row: NavigationMenuRow) -> Self {
        Self {
            module: row.module_id,
            module_name: String::new(),
            code: row.module_menu_code,
            title: row.name,
            path: row.path,
            icon: row.icon,
            sort_order: row.sort_order,
            permission: row.code,
        }
    }
}

fn database_error(operation: &'static str) -> impl FnOnce(sqlx::Error) -> ServiceError + Copy {
    move |error| {
        tracing::error!(%error, operation, "Module database operation failed");
        ServiceError::DatabaseQueryFailed
    }
}
