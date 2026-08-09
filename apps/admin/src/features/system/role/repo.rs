use crate::common::{
    error::ServiceError,
    query::{count_with_filters, fetch_with_filters, push_eq, push_ilike},
};

use chrono::Utc;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use super::types::{RoleListQuery, RoleWithMenusRow};

pub struct RoleRepository;

#[derive(Debug, PartialEq, Eq)]
pub enum SoftDeleteOutcome {
    Deleted,
    AssignmentBlocked(i64),
    NotFound,
}

impl RoleRepository {
    fn format_query(query: &RoleListQuery, query_builder: &mut QueryBuilder<Sqlite>) {
        push_ilike(query_builder, "name", query.role_name.as_deref());
        push_ilike(query_builder, "code", query.role_code.as_deref());
        push_eq(query_builder, "status", query.status);
    }

    /// Queries roles with pagination
    pub async fn list_roles(
        pool: &SqlitePool,
        offset: i64,
        limit: i64,
        query: RoleListQuery,
    ) -> Result<(Vec<RoleWithMenusRow>, i64), ServiceError> {
        let total = count_with_filters(
            pool,
            "SELECT COUNT(*) FROM role_with_menus WHERE 1=1",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
        )
        .await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }
        let roles = fetch_with_filters(
            pool,
            "SELECT id, name, code, description, status, created_at, updated_at, is_system, menus,
                    (SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = role_with_menus.id)
                        AS assigned_user_count
             FROM role_with_menus
             WHERE 1=1",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
            Some("created_at DESC"),
            Some(limit),
            Some(offset),
        )
        .await?;

        Ok((roles, total))
    }

    /// Creates a new role
    pub async fn create(
        pool: &SqlitePool,
        role_name: &str,
        role_code: &str,
        description: Option<&str>,
        status: i16,
        menu_ids: &[i64],
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for role creation: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        let now = Utc::now().naive_utc();

        let role_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO roles (name, code, description, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             RETURNING id",
        )
        .bind(role_name)
        .bind(role_code)
        .bind(description)
        .bind(status)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating role: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Self::insert_role_menus(&mut tx, role_id, menu_ids).await?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing role creation transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(role_id)
    }

    /// Updates an existing role
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        role_name: &str,
        role_code: &str,
        description: Option<&str>,
        status: i16,
        menu_ids: &[i64],
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for role update: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        let id_opt = sqlx::query_scalar::<_, i64>(
            "UPDATE roles
                 SET name = ?, code = ?, description = ?, status = ?, updated_at = ?
                 WHERE id = ? AND deleted_at IS NULL
                 RETURNING id",
        )
        .bind(role_name)
        .bind(role_code)
        .bind(description)
        .bind(status)
        .bind(Utc::now().naive_utc())
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error updating role: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        if let Some(id) = id_opt {
            Self::insert_role_menus(&mut tx, id, menu_ids).await?;
            tx.commit().await.map_err(|e| {
                tracing::error!("Database error committing role update transaction: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
            Ok(id)
        } else {
            Err(ServiceError::NotFound(format!("Role id: {}", id)))
        }
    }

    /// Soft deletes a role
    pub async fn soft_delete(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<SoftDeleteOutcome, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting role soft-delete transaction {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;
        let result = sqlx::query(
            "UPDATE roles
             SET deleted_at = ?, updated_at = ?
             WHERE id = ?
               AND deleted_at IS NULL
               AND NOT EXISTS (SELECT 1 FROM user_roles WHERE role_id = roles.id)",
        )
        .bind(Utc::now().naive_utc())
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error soft deleting role {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        let outcome = if result.rows_affected() > 0 {
            SoftDeleteOutcome::Deleted
        } else {
            let (role_is_active, assignment_count): (bool, i64) = sqlx::query_as(
                "SELECT EXISTS(SELECT 1 FROM roles WHERE id = ? AND deleted_at IS NULL),
                        (SELECT COUNT(*) FROM user_roles WHERE role_id = ?)",
            )
            .bind(id)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error classifying role soft-delete {}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;

            if role_is_active && assignment_count > 0 {
                SoftDeleteOutcome::AssignmentBlocked(assignment_count)
            } else {
                SoftDeleteOutcome::NotFound
            }
        };

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing role soft-delete {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(outcome)
    }

    pub async fn get_role_identity(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<(String, bool)>, ServiceError> {
        sqlx::query_as::<_, (String, bool)>(
            "SELECT code, is_system FROM roles WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching role {} identity: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    /// insert role_menus
    async fn insert_role_menus(
        tx: &mut sqlx::Transaction<'_, Sqlite>,
        role_id: i64,
        menu_ids: &[i64],
    ) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM role_menus WHERE role_id = ?")
            .bind(role_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error deleting existing role_menus: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
        if menu_ids.is_empty() {
            return Ok(());
        }
        let now = Utc::now().naive_utc();
        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("INSERT INTO role_menus (role_id, menu_id, created_at) ");
        query_builder.push_values(menu_ids.iter(), |mut builder, menu_id| {
            builder.push_bind(role_id).push_bind(menu_id).push_bind(now);
        });

        query_builder.build().execute(&mut **tx).await.map_err(|e| {
            tracing::error!("Database error inserting role_menus: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(())
    }

    /// Retrieves role list for Options API
    pub async fn list_role_options(
        pool: &SqlitePool,
        search_query: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String, String, bool)>, ServiceError> {
        fetch_with_filters(
            pool,
            "SELECT id, name, code, is_system FROM roles WHERE status = 1 AND deleted_at IS NULL",
            |query_builder| {
                push_ilike(query_builder, "name", search_query);
            },
            Some("name ASC"),
            limit,
            None,
        )
        .await
    }

    pub async fn get_role_user_count(pool: &SqlitePool, role_id: i64) -> Result<i64, ServiceError> {
        let result =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_roles WHERE role_id = ?")
                .bind(role_id)
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error getting role user count: {:?}", e);
                    ServiceError::DatabaseQueryFailed
                })?;
        Ok(result)
    }

    pub async fn list_menu_codes_by_ids(
        pool: &SqlitePool,
        menu_ids: &[i64],
    ) -> Result<Vec<String>, ServiceError> {
        if menu_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT code FROM menus WHERE deleted_at IS NULL AND id IN (");
        let mut separated = query_builder.separated(", ");
        for menu_id in menu_ids {
            separated.push_bind(menu_id);
        }
        separated.push_unseparated(")");

        query_builder.build_query_scalar().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error listing menu codes by ids: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use rustzen_storage::sqlite::{
        DatabaseConnectionOptions, connect_sqlite_with_options, database_url_from_path,
    };
    use sqlx::Execute;

    use super::*;

    #[test]
    fn format_query_uses_role_view_column_names() {
        let query = RoleListQuery {
            role_name: Some("admin".to_string()),
            role_code: Some("system".to_string()),
            status: Some(1),
        };
        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT id FROM role_with_menus WHERE 1=1");

        RoleRepository::format_query(&query, &mut query_builder);

        let query = query_builder.build();
        let sql_text = query.sql();
        let sql = sql_text.as_str();
        assert!(sql.contains("LOWER(name)"));
        assert!(sql.contains("LOWER(code)"));
        assert!(!sql.contains("role_name"));
        assert!(!sql.contains("role_code"));
    }

    #[tokio::test]
    async fn soft_delete_rechecks_assignments_after_stale_count() {
        let database_path = std::env::temp_dir().join(format!(
            "rustzen-admin-role-race-{}-{}.db",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock").as_nanos()
        ));
        let database_url = database_url_from_path(&database_path);
        let options = DatabaseConnectionOptions {
            max_connections: 1,
            min_connections: 1,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: None,
        };
        let assignment_pool = connect_sqlite_with_options(&database_url, options.clone())
            .await
            .expect("assignment database");
        let delete_pool =
            connect_sqlite_with_options(&database_url, options).await.expect("delete database");
        crate::infra::db::run_migrations(&assignment_pool).await.expect("migrations");

        let role_id: i64 = sqlx::query_scalar(
            "INSERT INTO roles (name, code, status, is_system)
             VALUES ('Race role', 'race_role', 1, FALSE)
             RETURNING id",
        )
        .fetch_one(&assignment_pool)
        .await
        .expect("race role");

        // This models the service-level check before the concurrent assignment commits.
        assert_eq!(RoleRepository::get_role_user_count(&delete_pool, role_id).await.unwrap(), 0);

        let mut assignment_tx = assignment_pool.begin().await.expect("assignment transaction");
        crate::features::system::user::repo::UserRepository::insert_user_roles(
            &mut assignment_tx,
            1,
            &[role_id],
        )
        .await
        .expect("assignment transaction");
        assignment_tx.commit().await.expect("assignment commit");

        // The stale zero count must not allow the role soft-delete to commit.
        assert_eq!(
            RoleRepository::soft_delete(&delete_pool, role_id).await.unwrap(),
            SoftDeleteOutcome::AssignmentBlocked(1)
        );
        let (deleted_at, assignment_count): (Option<chrono::NaiveDateTime>, i64) = sqlx::query_as(
            "SELECT r.deleted_at, COUNT(ur.role_id)
                 FROM roles r
                 LEFT JOIN user_roles ur ON ur.role_id = r.id
                 WHERE r.id = ?
                 GROUP BY r.id, r.deleted_at",
        )
        .bind(role_id)
        .fetch_one(&assignment_pool)
        .await
        .expect("role invariant");
        assert!(deleted_at.is_none());
        assert_eq!(assignment_count, 1);

        let deleted_role_id: i64 = sqlx::query_scalar(
            "INSERT INTO roles (name, code, status, is_system)
             VALUES ('Deleted role', 'deleted_role', 1, FALSE)
             RETURNING id",
        )
        .fetch_one(&assignment_pool)
        .await
        .expect("deleted role");
        assert_eq!(
            RoleRepository::soft_delete(&delete_pool, deleted_role_id).await.unwrap(),
            SoftDeleteOutcome::Deleted
        );

        let mut rejected_assignment_tx =
            assignment_pool.begin().await.expect("assignment transaction");
        let error = crate::features::system::user::repo::UserRepository::insert_user_roles(
            &mut rejected_assignment_tx,
            1,
            &[deleted_role_id],
        )
        .await
        .expect_err("deleted role assignment");
        assert!(matches!(error, ServiceError::NotFound(_)));
        rejected_assignment_tx.rollback().await.expect("assignment rollback");
        let orphan_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM user_roles ur
             INNER JOIN roles r ON r.id = ur.role_id
             WHERE r.deleted_at IS NOT NULL",
        )
        .fetch_one(&assignment_pool)
        .await
        .expect("orphan assignment count");
        assert_eq!(orphan_count, 0);

        assignment_pool.close().await;
        delete_pool.close().await;
        for suffix in ["", "-wal", "-shm"] {
            let path = PathBuf::from(format!("{}{suffix}", database_path.display()));
            let _ = fs::remove_file(path);
        }
    }
}
