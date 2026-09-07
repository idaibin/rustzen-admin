use sqlx::sqlite::SqlitePoolOptions;

use super::session::{DELETE_EXPIRED_SQL, DELETE_REVOKED_SQL};

#[tokio::test]
async fn cleanup_uses_bounded_indexes_with_many_active_sessions() {
    let pool =
        SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    sqlx::query(
        "WITH RECURSIVE n(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM n WHERE x<2000)
         INSERT INTO access_sessions
             (sid,user_id,auth_epoch_at_issue,expires_at,created_at)
         SELECT printf('active-%030d',x),1,1,2000000000+x,x FROM n",
    )
    .execute(&pool)
    .await
    .expect("active sessions");

    let expiry_plan = sqlx::query_as::<_, (i64, i64, i64, String)>(
        "EXPLAIN QUERY PLAN SELECT sid FROM access_sessions
         WHERE expires_at <= 100 ORDER BY expires_at LIMIT 100",
    )
    .fetch_all(&pool)
    .await
    .expect("expiry plan");
    assert!(expiry_plan.iter().any(|row| row.3.contains("idx_access_sessions_expiry")));
    let revoked_plan = sqlx::query_as::<_, (i64, i64, i64, String)>(
        "EXPLAIN QUERY PLAN SELECT sid FROM access_sessions
         WHERE revoked_at IS NOT NULL ORDER BY revoked_at LIMIT 100",
    )
    .fetch_all(&pool)
    .await
    .expect("revoked plan");
    assert!(revoked_plan.iter().any(|row| row.3.contains("idx_access_sessions_revoked")));

    sqlx::query(DELETE_EXPIRED_SQL)
        .bind(100_i64)
        .bind(100_i64)
        .execute(&pool)
        .await
        .expect("bounded expiry cleanup");
    sqlx::query(DELETE_REVOKED_SQL)
        .bind(100_i64)
        .execute(&pool)
        .await
        .expect("bounded revoked cleanup");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM access_sessions")
            .fetch_one(&pool)
            .await
            .expect("remaining sessions"),
        2000
    );
}
