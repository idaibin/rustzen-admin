//! One-time offline owner credential replacement for a fresh Monitor Admin DB.
use crate::infra::config::CONFIG;
use crate::infra::password::PasswordUtils;

const BOOTSTRAP_INPUT_FILE: &str = "data/db/admin/bootstrap-owner-password";
const DEFAULT_DEV_OWNER_PASSWORD: &str = "rustzen@123";

/// Consumes the installer-created, Admin-only input before the full Admin server listens.
/// The separately retained initial-credential file is the operator handoff.
pub async fn consume_installer_owner_secret(pool: &sqlx::SqlitePool) -> Result<(), String> {
    let path = CONFIG.runtime_root_dir().join(BOOTSTRAP_INPUT_FILE);
    let development_password =
        (!CONFIG.runtime.requires_production_secrets()).then_some(DEFAULT_DEV_OWNER_PASSWORD);
    consume_owner_secret_at(pool, &path, development_password).await
}

async fn consume_owner_secret_at(
    pool: &sqlx::SqlitePool,
    path: &std::path::Path,
    development_password: Option<&str>,
) -> Result<(), String> {
    let secret = match std::fs::read_to_string(path) {
        Ok(secret) => Some(secret),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => return Err("bootstrap owner secret is unreadable".into()),
    };

    let status = sqlx::query_scalar::<_, i64>(
        "SELECT status FROM users WHERE username = 'owner' AND deleted_at IS NULL",
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| "bootstrap owner record is unavailable")?
    .ok_or("bootstrap owner record is unavailable")?;
    if secret.is_none() && status == 1 {
        return Ok(());
    }
    let password = match secret.as_deref() {
        Some(secret) => secret.strip_suffix('\n').unwrap_or(secret),
        None => development_password.ok_or("bootstrap owner secret is missing")?,
    };
    if (secret.is_some() && password.len() < 12)
        || password.contains(['\n', '\r'])
        || password == "replace-me"
    {
        return Err("bootstrap owner secret is invalid".into());
    }
    if status == 2 {
        let hash =
            PasswordUtils::hash_password(password).map_err(|_| "bootstrap owner hashing failed")?;
        let changed = sqlx::query("UPDATE users SET password_hash = ?, status = 1, updated_at = CURRENT_TIMESTAMP WHERE username = 'owner' AND status = 2 AND deleted_at IS NULL")
            .bind(hash)
            .execute(pool)
            .await
            .map_err(|_| "bootstrap owner update failed")?
            .rows_affected();
        if changed != 1 {
            return Err("bootstrap owner record is unavailable".into());
        }
    } else if status != 1 {
        return Err("bootstrap owner record is unavailable".into());
    }
    if secret.is_some() {
        std::fs::remove_file(path).map_err(|_| "bootstrap owner secret could not be consumed")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{PasswordUtils, consume_owner_secret_at};
    use crate::infra::db::run_migrations;

    fn test_password() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    fn documented_local_password() -> String {
        ["rustzen", "@123"].concat()
    }

    #[tokio::test]
    async fn fresh_full_owner_rejects_the_public_default_and_consumes_the_installer_secret() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("migrations");
        let owner = sqlx::query_as::<_, (String, i64)>(
            "SELECT password_hash, status FROM users WHERE username = 'owner'",
        )
        .fetch_one(&pool)
        .await
        .expect("seed owner");
        assert_eq!(owner.1, 2);
        assert!(!PasswordUtils::verify_password(&documented_local_password(), &owner.0));
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE username IN ('admin', 'viewer')",
            )
            .fetch_one(&pool)
            .await
            .expect("default accounts"),
            0
        );

        let path =
            std::env::temp_dir().join(format!("rz-bootstrap-owner-{}.txt", uuid::Uuid::new_v4()));
        let password = test_password();
        std::fs::write(&path, format!("{password}\n")).expect("credential input");
        consume_owner_secret_at(&pool, &path, None).await.expect("consume credential");
        assert!(!path.exists());
        let updated = sqlx::query_scalar::<_, String>(
            "SELECT password_hash FROM users WHERE username = 'owner'",
        )
        .fetch_one(&pool)
        .await
        .expect("updated owner");
        assert!(PasswordUtils::verify_password(&password, &updated));
        consume_owner_secret_at(&pool, &path, None).await.expect("one-time retry");
    }

    #[tokio::test]
    async fn fresh_full_owner_requires_the_installer_secret() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("migrations");
        let missing = std::env::temp_dir()
            .join(format!("rz-bootstrap-owner-missing-{}.txt", uuid::Uuid::new_v4()));
        assert_eq!(
            consume_owner_secret_at(&pool, &missing, None).await.unwrap_err(),
            "bootstrap owner secret is missing"
        );
    }

    #[tokio::test]
    async fn fresh_development_owner_uses_the_documented_local_password() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.expect("pool");
        run_migrations(&pool).await.expect("migrations");
        let missing = std::env::temp_dir()
            .join(format!("rz-bootstrap-owner-dev-{}.txt", uuid::Uuid::new_v4()));

        let password = documented_local_password();
        consume_owner_secret_at(&pool, &missing, Some(&password))
            .await
            .expect("activate development owner");

        let updated = sqlx::query_scalar::<_, String>(
            "SELECT password_hash FROM users WHERE username = 'owner' AND status = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("active development owner");
        assert!(PasswordUtils::verify_password(&password, &updated));
    }
}
