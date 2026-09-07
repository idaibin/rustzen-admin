use super::{AccountRepository, AccountService};
use crate::{
    common::error::ServiceError,
    features::account::types::ChangeAccountPasswordRequest,
    infra::{password::PasswordUtils, permission::PermissionService},
};

#[test]
fn build_password_hash_requires_current_password_and_confirmation() {
    let current_hash = PasswordUtils::hash_password("current-password").expect("hash");

    let new_hash = AccountService::build_password_hash(
        "current-password",
        &current_hash,
        "new-password",
        "new-password",
    )
    .expect("password hash");

    assert!(PasswordUtils::verify_password("new-password", &new_hash));
    assert!(!PasswordUtils::verify_password("current-password", &new_hash));
    assert!(
        AccountService::build_password_hash(
            "wrong-password",
            &current_hash,
            "new-password",
            "new-password",
        )
        .is_err()
    );
    assert!(
        AccountService::build_password_hash(
            "current-password",
            &current_hash,
            "new-password",
            "different-password",
        )
        .is_err()
    );
}

#[tokio::test]
async fn password_change_rejects_a_user_deleted_between_read_and_write() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    let original_hash = PasswordUtils::hash_password("current-password").expect("hash");
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (id, username, email, password_hash, status, is_system)
         VALUES (900001, ?, ?, ?, 1, FALSE) RETURNING id",
    )
    .bind("password-race")
    .bind("password-race@example.test")
    .bind(&original_hash)
    .fetch_one(&pool)
    .await
    .expect("user");
    sqlx::query(
        "CREATE TRIGGER delete_before_password_write
         BEFORE UPDATE OF password_hash ON users
         BEGIN
             UPDATE users SET deleted_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
             SELECT RAISE(IGNORE);
         END",
    )
    .execute(&pool)
    .await
    .expect("trigger");
    PermissionService::cache_user_permissions(user_id, &["account:profile:update".to_string()]);

    let error = AccountService::change_password(
        &pool,
        user_id,
        ChangeAccountPasswordRequest {
            current_password: "current-password".to_string(),
            new_password: "new-password".to_string(),
            confirm_password: "new-password".to_string(),
        },
    )
    .await
    .expect_err("deleted user must not report a successful password change");

    assert!(matches!(error, ServiceError::NotFound(resource) if resource == "User"));
    let password_hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("password hash");
    assert!(PasswordUtils::verify_password("current-password", &password_hash));
    assert!(!PasswordUtils::verify_password("new-password", &password_hash));
    assert!(matches!(
        PermissionService::load_current_user(user_id, "password-race"),
        Err(ServiceError::InvalidToken)
    ));
}

#[tokio::test]
async fn password_updates_compare_and_swap_the_verified_hash() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    let original_hash = PasswordUtils::hash_password("current-password").expect("hash");
    let first_hash = PasswordUtils::hash_password("first-password").expect("hash");
    let second_hash = PasswordUtils::hash_password("second-password").expect("hash");
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, email, password_hash, status, is_system)
         VALUES ('password-cas', 'password-cas@example.test', ?, 1, FALSE) RETURNING id",
    )
    .bind(&original_hash)
    .fetch_one(&pool)
    .await
    .expect("user");

    let (first, second) = tokio::join!(
        AccountRepository::update_password(&pool, user_id, &original_hash, &first_hash),
        AccountRepository::update_password(&pool, user_id, &original_hash, &second_hash),
    );
    let first = first.expect("first update");
    let second = second.expect("second update");
    assert_ne!(first, second, "exactly one update must win the password CAS");

    let password_hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("password hash");
    let expected_hash = if first { &first_hash } else { &second_hash };
    assert_eq!(&password_hash, expected_hash);
}
