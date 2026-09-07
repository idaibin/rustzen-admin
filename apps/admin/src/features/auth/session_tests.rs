use rustzen_auth::auth::JwtCodec;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use super::SessionRepository;
use crate::features::system::user::{repo::UserRepository, types::CreateUserCommand};
use crate::{features::auth::service::AuthService, infra::password::PasswordUtils};

async fn user_fixture() -> SqlitePool {
    let pool =
        SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    let password = PasswordUtils::hash_password("SessionPassw0rd!").expect("password");
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, status)
         VALUES (40, 'session-user', 'session@example.com', ?, 1)",
    )
    .bind(password)
    .execute(&pool)
    .await
    .expect("user");
    let role_id: i64 = sqlx::query_scalar(
        "INSERT INTO roles (name, code, status) VALUES ('Session role', 'session-role', 1)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("role");
    let menu_id: i64 = sqlx::query_scalar(
        "INSERT INTO menus (name, code, menu_type, status, is_active)
         VALUES ('Session read', 'session:read', 3, 1, 1) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("menu");
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (40, ?)")
        .bind(role_id)
        .execute(&pool)
        .await
        .expect("membership");
    sqlx::query("INSERT INTO role_menus (role_id, menu_id) VALUES (?, ?)")
        .bind(role_id)
        .bind(menu_id)
        .execute(&pool)
        .await
        .expect("grant");
    pool
}

#[tokio::test]
async fn login_persists_two_sessions_and_logout_revokes_only_one_sid() {
    let pool = user_fixture().await;
    let first =
        AuthService::login(&pool, "session-user", "SessionPassw0rd!").await.expect("first login");
    let second =
        AuthService::login(&pool, "session-user", "SessionPassw0rd!").await.expect("second login");
    let first_claims = crate::infra::auth_runtime::jwt_codec().decode(&first.token).expect("first");
    let second_claims =
        crate::infra::auth_runtime::jwt_codec().decode(&second.token).expect("second");
    let now = chrono::Utc::now().timestamp();
    assert!(
        SessionRepository::load_authoritative_user(&pool, &first_claims, now)
            .await
            .expect("first authority")
            .has_capability("session:read")
    );
    AuthService::logout(&pool, 40, &first_claims.sid).await.expect("logout first");
    assert!(SessionRepository::load_authoritative_user(&pool, &first_claims, now).await.is_err());
    assert!(SessionRepository::load_authoritative_user(&pool, &second_claims, now).await.is_ok());
}

#[tokio::test]
async fn password_status_and_revoke_all_invalidate_old_auth_epochs() {
    let pool = user_fixture().await;
    let login = AuthService::login(&pool, "session-user", "SessionPassw0rd!").await.expect("login");
    let claims = crate::infra::auth_runtime::jwt_codec().decode(&login.token).expect("claims");
    let now = chrono::Utc::now().timestamp();
    sqlx::query("UPDATE users SET password_hash = 'changed' WHERE id = 40")
        .execute(&pool)
        .await
        .expect("password reset");
    assert!(SessionRepository::load_authoritative_user(&pool, &claims, now).await.is_err());

    let epoch: i64 = sqlx::query_scalar("SELECT auth_epoch FROM users WHERE id = 40")
        .fetch_one(&pool)
        .await
        .expect("epoch");
    let codec = JwtCodec::new("test", 3600);
    let sid = uuid::Uuid::new_v4().to_string();
    let replacement = codec.claims_at(40, "session-user", &sid, epoch, now);
    SessionRepository::create(&pool, 40, &sid, epoch, now, replacement.exp as i64)
        .await
        .expect("replacement session");
    SessionRepository::revoke_all(&pool, 40, now + 1).await.expect("revoke all");
    assert!(
        SessionRepository::load_authoritative_user(&pool, &replacement, now + 1).await.is_err()
    );

    sqlx::query("UPDATE users SET status = 2 WHERE id = 40").execute(&pool).await.expect("disable");
    let disabled_epoch: i64 = sqlx::query_scalar("SELECT auth_epoch FROM users WHERE id = 40")
        .fetch_one(&pool)
        .await
        .expect("disabled epoch");
    assert!(disabled_epoch > epoch);
}

#[tokio::test]
async fn current_grants_are_reloaded_for_an_existing_session() {
    let pool = user_fixture().await;
    let login = AuthService::login(&pool, "session-user", "SessionPassw0rd!").await.expect("login");
    let claims = crate::infra::auth_runtime::jwt_codec().decode(&login.token).expect("claims");
    let now = chrono::Utc::now().timestamp();
    assert!(
        SessionRepository::load_authoritative_user(&pool, &claims, now)
            .await
            .expect("authority")
            .has_capability("session:read")
    );
    sqlx::query(
        "DELETE FROM role_menus WHERE menu_id = (SELECT id FROM menus WHERE code = 'session:read')",
    )
    .execute(&pool)
    .await
    .expect("revoke grant");
    let user = SessionRepository::load_authoritative_user(&pool, &claims, now)
        .await
        .expect("refreshed authority");
    assert!(!user.has_capability("session:read"));
}

#[tokio::test]
async fn active_session_limit_revokes_the_oldest_and_database_failure_is_closed() {
    let pool = user_fixture().await;
    let codec = JwtCodec::new("test", 3600);
    let now = chrono::Utc::now().timestamp();
    let mut claims = Vec::new();
    for offset in 0..11 {
        let sid = uuid::Uuid::new_v4().to_string();
        let item = codec.claims_at(40, "session-user", &sid, 1, now + offset);
        SessionRepository::create(&pool, 40, &sid, 1, now + offset, item.exp as i64)
            .await
            .expect("session");
        claims.push(item);
    }
    assert!(SessionRepository::load_authoritative_user(&pool, &claims[0], now + 11).await.is_err());
    assert!(SessionRepository::load_authoritative_user(&pool, &claims[10], now + 11).await.is_ok());
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM access_sessions
             WHERE user_id = 40 AND revoked_at IS NULL AND expires_at > ?",
        )
        .bind(now + 11)
        .fetch_one(&pool)
        .await
        .expect("active count"),
        10
    );
    pool.close().await;
    assert!(
        SessionRepository::load_authoritative_user(&pool, &claims[10], now + 11).await.is_err()
    );
}

#[tokio::test]
async fn revoked_actor_cannot_commit_an_access_write_after_request_authentication() {
    let pool = user_fixture().await;
    let owner_menu: i64 = sqlx::query_scalar("SELECT id FROM menus WHERE code = '*'")
        .fetch_one(&pool)
        .await
        .expect("owner capability");
    sqlx::query(
        "INSERT INTO role_menus (role_id, menu_id)
         SELECT role_id, ? FROM user_roles WHERE user_id = 40",
    )
    .bind(owner_menu)
    .execute(&pool)
    .await
    .expect("owner grant");
    let epoch: i64 = sqlx::query_scalar("SELECT auth_epoch FROM users WHERE id = 40")
        .fetch_one(&pool)
        .await
        .expect("owner epoch");
    let codec = JwtCodec::new("test", 3600);
    let sid = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let actor = codec.claims_at(40, "session-user", &sid, epoch, now);
    SessionRepository::create(&pool, 40, &sid, epoch, now, actor.exp as i64)
        .await
        .expect("owner session");
    SessionRepository::revoke_sid(&pool, 40, &sid, now).await.expect("revoke actor");

    let command = CreateUserCommand {
        username: "must-not-write".into(),
        email: "must-not-write@example.com".into(),
        password_hash: "unused".into(),
        real_name: None,
        status: Some(1),
        role_ids: vec![1],
    };
    assert!(UserRepository::create_user(&pool, &command, &actor).await.is_err());
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE username = 'must-not-write'"
        )
        .fetch_one(&pool)
        .await
        .expect("write count"),
        0
    );
}

#[tokio::test]
async fn signed_claim_with_non_random_session_id_is_rejected() {
    let pool = user_fixture().await;
    let now = chrono::Utc::now().timestamp();
    let claims =
        JwtCodec::new("test", 3600).claims_at(40, "session-user", "predictable-sid!", 1, now);
    sqlx::query(
        "INSERT INTO access_sessions
         (sid, user_id, auth_epoch_at_issue, expires_at, created_at)
         VALUES (?, 40, 1, ?, ?)",
    )
    .bind(&claims.sid)
    .bind(claims.exp as i64)
    .bind(now)
    .execute(&pool)
    .await
    .expect("malformed session fixture");
    assert!(SessionRepository::load_authoritative_user(&pool, &claims, now).await.is_err());
}

#[tokio::test]
async fn post_insert_failure_rolls_back_session_eviction_and_last_login() {
    let pool = user_fixture().await;
    let now = chrono::Utc::now().timestamp();
    for offset in 0..10 {
        let sid = uuid::Uuid::new_v4().to_string();
        SessionRepository::create(&pool, 40, &sid, 1, now + offset, now + 3600 + offset)
            .await
            .expect("existing session");
    }
    sqlx::query(
        "CREATE TEMP TRIGGER fail_after_login_session
         AFTER INSERT ON access_sessions
         BEGIN SELECT RAISE(ABORT, 'post-insert login failpoint'); END",
    )
    .execute(&pool)
    .await
    .expect("failpoint");

    assert!(AuthService::login(&pool, "session-user", "SessionPassw0rd!").await.is_err());
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM access_sessions
             WHERE user_id=40 AND revoked_at IS NULL AND expires_at>?",
        )
        .bind(now)
        .fetch_one(&pool)
        .await
        .expect("active sessions"),
        10
    );
    assert_eq!(
        sqlx::query_scalar::<_, Option<String>>("SELECT last_login_at FROM users WHERE id=40")
            .fetch_one(&pool)
            .await
            .expect("last login"),
        None
    );
}
