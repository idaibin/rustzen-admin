//! One-time offline owner credential replacement for a fresh Monitor Admin DB.
use crate::infra::{db, password::PasswordUtils};
use std::io::Read;

fn read_secret() -> Result<String, String> {
    let mut secret = String::new();
    std::io::stdin()
        .read_to_string(&mut secret)
        .map_err(|_| "bootstrap owner secret is unreadable")?;
    if secret.len() > 1024 {
        return Err("bootstrap owner secret is invalid".into());
    }
    Ok(secret)
}

pub async fn replace_seed_owner() -> Result<(), String> {
    let secret = read_secret()?;
    let password = secret.strip_suffix('\n').unwrap_or(&secret);
    if password.len() < 12 || password.contains(['\n', '\r']) || password == "replace-me" {
        return Err("bootstrap owner secret is invalid".into());
    }
    let pool =
        db::create_default_pool().await.map_err(|_| "bootstrap owner database is unavailable")?;
    db::run_migrations(&pool).await.map_err(|_| "bootstrap owner migration failed")?;
    let hash =
        PasswordUtils::hash_password(password).map_err(|_| "bootstrap owner hashing failed")?;
    let changed = sqlx::query("UPDATE users SET password_hash = ?, status = 1, updated_at = CURRENT_TIMESTAMP WHERE username = 'owner' AND status = 2 AND deleted_at IS NULL")
        .bind(hash).execute(&pool).await.map_err(|_| "bootstrap owner update failed")?.rows_affected();
    if changed != 1 {
        return Err("bootstrap owner record is unavailable".into());
    }
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&pool)
        .await
        .map_err(|_| "bootstrap owner checkpoint failed")?;
    pool.close().await;
    Ok(())
}

pub async fn verify_owner() -> Result<(), String> {
    let secret = read_secret()?;
    let password = secret.strip_suffix('\n').unwrap_or(&secret);
    let pool = db::create_default_pool().await.map_err(|_| "owner database is unavailable")?;
    let row = sqlx::query_as::<_, (String, i64)>(
        "SELECT password_hash, status FROM users WHERE username = 'owner' AND deleted_at IS NULL",
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| "owner record is unavailable")?
    .ok_or("owner record is unavailable")?;
    if row.1 != 1 || !PasswordUtils::verify_password(password, &row.0) {
        return Err("owner credential differs from requested tuple".into());
    }
    Ok(())
}
