use chrono::Utc;
use rustzen_auth::auth::JwtCodec;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use super::session::SessionRepository;

async fn policy_epoch(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT authz_epoch FROM access_policy_state WHERE id=1")
        .fetch_one(pool)
        .await
        .expect("policy epoch")
}

async fn assert_policy_step(pool: &SqlitePool, expected: &mut i64, statement: &'static str) {
    sqlx::query(statement).execute(pool).await.expect("policy mutation");
    *expected += 1;
    assert_eq!(policy_epoch(pool).await, *expected, "statement: {statement}");
}

#[tokio::test]
async fn fresh_schema_epochs_advance_once_and_immediately_invalidate_sessions() {
    let pool =
        SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    sqlx::query(
        "INSERT INTO users (id,username,email,password_hash,status) VALUES
             (70,'epoch-user','epoch@example.test','before',1),
             (71,'deleted-user','deleted@example.test','before',1);
         INSERT INTO roles (id,name,code,status) VALUES
             (7001,'Epoch one','epoch-one',1),(7002,'Epoch two','epoch-two',1);
         INSERT INTO menus (id,name,code,menu_type,status,is_active) VALUES
             (7101,'Epoch menu one','epoch:one',3,1,1),
             (7102,'Epoch menu two','epoch:two',3,1,1);",
    )
    .execute(&pool)
    .await
    .expect("fixtures");

    let now = Utc::now().timestamp();
    let codec = JwtCodec::new("test", 3600);
    let sid = uuid::Uuid::new_v4().to_string();
    let claims = codec.claims_at(70, "epoch-user", &sid, 1, now);
    SessionRepository::create(&pool, 70, &sid, 1, now, claims.exp as i64)
        .await
        .expect("current session");
    let policy_before_password = policy_epoch(&pool).await;
    sqlx::query("UPDATE users SET password_hash='after' WHERE id=70")
        .execute(&pool)
        .await
        .expect("password mutation");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT auth_epoch FROM users WHERE id=70")
            .fetch_one(&pool)
            .await
            .expect("password epoch"),
        2
    );
    assert_eq!(policy_epoch(&pool).await, policy_before_password);
    assert!(SessionRepository::load_authoritative_user(&pool, &claims, now).await.is_err());

    let replacement_sid = uuid::Uuid::new_v4().to_string();
    let replacement = codec.claims_at(70, "epoch-user", &replacement_sid, 2, now);
    SessionRepository::create(&pool, 70, &replacement_sid, 2, now, replacement.exp as i64)
        .await
        .expect("replacement session");
    let mut expected = policy_epoch(&pool).await;
    assert_policy_step(&pool, &mut expected, "UPDATE users SET status=2 WHERE id=70").await;
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT auth_epoch FROM users WHERE id=70")
            .fetch_one(&pool)
            .await
            .expect("disabled epoch"),
        3
    );
    assert!(SessionRepository::load_authoritative_user(&pool, &replacement, now).await.is_err());

    for statement in [
        "UPDATE users SET status=1 WHERE id=70",
        "UPDATE users SET deleted_at=CURRENT_TIMESTAMP WHERE id=71",
        "UPDATE roles SET status=2 WHERE id=7001",
        "UPDATE roles SET deleted_at=CURRENT_TIMESTAMP WHERE id=7002",
        "INSERT INTO user_roles (user_id,role_id) VALUES (70,7001)",
        "UPDATE user_roles SET role_id=7002 WHERE user_id=70 AND role_id=7001",
        "DELETE FROM user_roles WHERE user_id=70 AND role_id=7002",
        "INSERT INTO role_menus (role_id,menu_id) VALUES (7001,7101)",
        "UPDATE role_menus SET menu_id=7102 WHERE role_id=7001 AND menu_id=7101",
        "DELETE FROM role_menus WHERE role_id=7001 AND menu_id=7102",
        "UPDATE menus SET status=2 WHERE id=7101",
        "UPDATE menus SET is_active=0 WHERE id=7102",
        "UPDATE menus SET deleted_at=CURRENT_TIMESTAMP WHERE id=7101",
        "UPDATE modules SET enabled=NOT enabled WHERE id='monitor'",
    ] {
        assert_policy_step(&pool, &mut expected, statement).await;
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT auth_epoch FROM users WHERE id=71")
            .fetch_one(&pool)
            .await
            .expect("deleted epoch"),
        2
    );
}
