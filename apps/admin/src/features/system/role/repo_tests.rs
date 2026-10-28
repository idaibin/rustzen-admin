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
        keyword: None,
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

async fn keyword_fixture() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("keyword fixture pool");
    crate::infra::db::run_migrations(&pool).await.expect("keyword fixture migrations");
    for (index, (name, code, status, deleted)) in [
        ("Needle by name", "alpha", 1_i16, false),
        ("Other enabled", "needle_code", 1, false),
        ("Needle disabled", "beta", 2, false),
        ("Other disabled", "needle_disabled", 2, false),
        ("Needle both", "needle_both", 1, false),
        ("Literal %_\\' 中文", "literal_special", 1, false),
        ("Ordinary", "plain", 1, false),
        ("Needle removed", "needle_deleted", 1, true),
    ]
    .into_iter()
    .enumerate()
    {
        sqlx::query(
            "INSERT INTO roles (name, code, status, is_system, created_at, deleted_at)
             VALUES (?, ?, ?, FALSE, ?, ?)",
        )
        .bind(name)
        .bind(code)
        .bind(status)
        .bind(format!("2026-01-{:02} 00:00:00", index + 1))
        .bind(deleted.then_some("2026-02-01 00:00:00"))
        .execute(&pool)
        .await
        .expect("keyword fixture role");
    }
    pool
}

fn keyword_query(keyword: Option<&str>, status: Option<i16>) -> RoleListQuery {
    RoleListQuery { keyword: keyword.map(str::to_owned), role_name: None, role_code: None, status }
}

fn sorted_codes(rows: &[RoleWithMenusRow]) -> Vec<&str> {
    let mut codes: Vec<_> = rows.iter().map(|row| row.code.as_str()).collect();
    codes.sort_unstable();
    codes
}

#[tokio::test]
async fn keyword_matches_name_or_code_once_and_excludes_deleted_roles() {
    let pool = keyword_fixture().await;
    let (rows, total) =
        RoleRepository::list_roles(&pool, 0, 20, keyword_query(Some("  NeEdLe  "), None))
            .await
            .expect("name or code query");
    assert_eq!(total, 5);
    assert_eq!(
        sorted_codes(&rows),
        ["alpha", "beta", "needle_both", "needle_code", "needle_disabled"]
    );
    pool.close().await;
}

#[tokio::test]
async fn keyword_status_intersects_both_name_and_code_matches() {
    let pool = keyword_fixture().await;
    for (status, expected) in
        [(1, vec!["alpha", "needle_both", "needle_code"]), (2, vec!["beta", "needle_disabled"])]
    {
        let (rows, total) =
            RoleRepository::list_roles(&pool, 0, 20, keyword_query(Some("needle"), Some(status)))
                .await
                .expect("keyword and status query");
        assert_eq!(total, expected.len() as i64);
        assert_eq!(sorted_codes(&rows), expected);
        assert!(rows.iter().all(|row| row.status == status));
    }
    pool.close().await;
}

#[tokio::test]
async fn keyword_special_characters_are_literal_and_bound() {
    let pool = keyword_fixture().await;
    for keyword in ["%", "_\\", "\\", "'", "中文"] {
        let (rows, total) =
            RoleRepository::list_roles(&pool, 0, 20, keyword_query(Some(keyword), None))
                .await
                .expect("literal keyword query");
        assert_eq!(total, 1, "keyword {keyword:?}");
        assert_eq!(sorted_codes(&rows), ["literal_special"]);
    }
    let (rows, total) =
        RoleRepository::list_roles(&pool, 0, 20, keyword_query(Some("' OR 1=1 --"), None))
            .await
            .expect("SQL-looking keyword is data");
    assert_eq!(total, 0);
    assert!(rows.is_empty());
    let (rows, total) =
        RoleRepository::list_roles(&pool, 0, 20, keyword_query(Some("needle_"), None))
            .await
            .expect("literal underscore in code");
    assert_eq!(total, 3);
    assert_eq!(sorted_codes(&rows), ["needle_both", "needle_code", "needle_disabled"]);
    pool.close().await;
}

#[tokio::test]
async fn keyword_empty_and_absent_are_equivalent_and_legacy_filters_still_intersect() {
    let pool = keyword_fixture().await;
    let (unfiltered, total) =
        RoleRepository::list_roles(&pool, 0, 50, keyword_query(None, None)).await.unwrap();
    for keyword in ["", " \t\n "] {
        let (rows, count) =
            RoleRepository::list_roles(&pool, 0, 50, keyword_query(Some(keyword), None))
                .await
                .unwrap();
        assert_eq!(count, total);
        assert_eq!(sorted_codes(&rows), sorted_codes(&unfiltered));
    }
    for keyword in [None, Some("NEEDLE")] {
        let query = RoleListQuery {
            keyword: keyword.map(str::to_owned),
            role_name: Some("other".to_owned()),
            role_code: Some("code".to_owned()),
            status: Some(1),
        };
        let (rows, count) = RoleRepository::list_roles(&pool, 0, 20, query).await.unwrap();
        assert_eq!(count, 1);
        assert_eq!(sorted_codes(&rows), ["needle_code"]);
    }
    let query = RoleListQuery { role_name: Some("%".to_owned()), ..keyword_query(None, None) };
    let (rows, count) = RoleRepository::list_roles(&pool, 0, 50, query).await.unwrap();
    assert_eq!(count, total, "legacy roleName keeps its LIKE semantics");
    assert_eq!(sorted_codes(&rows), sorted_codes(&unfiltered));
    pool.close().await;
}

#[tokio::test]
async fn keyword_count_and_paginated_rows_share_the_same_predicate() {
    let pool = keyword_fixture().await;
    let mut codes = Vec::new();
    for offset in [0, 2, 4, 6] {
        let (rows, total) =
            RoleRepository::list_roles(&pool, offset, 2, keyword_query(Some("needle"), None))
                .await
                .unwrap();
        assert_eq!(total, 5);
        assert_eq!(rows.len(), (5_i64 - offset).clamp(0, 2) as usize);
        codes.extend(rows.into_iter().map(|row| row.code));
    }
    assert_eq!(codes, ["needle_both", "needle_disabled", "beta", "needle_code", "alpha"]);
    pool.close().await;
}
