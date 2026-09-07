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

    let mut rejected_assignment_tx = assignment_pool.begin().await.expect("assignment transaction");
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
