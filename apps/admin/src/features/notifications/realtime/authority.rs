use rustzen_auth::auth::{AuthClaims, CurrentUser};
use sqlx::SqlitePool;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Snapshot {
    pub revision: i64,
    authz_epoch: i64,
    permissions: Vec<String>,
    modules: Vec<String>,
}

impl Snapshot {
    pub(super) fn same_authority(&self, expected: &Self) -> bool {
        self.authz_epoch == expected.authz_epoch
            && self.permissions == expected.permissions
            && self.modules == expected.modules
    }
}

#[derive(Debug, thiserror::Error)]
pub(super) enum Error {
    #[error("invalid notification stream identity")]
    Unauthorized,
    #[error("notification stream is not currently authorized")]
    Forbidden,
    #[error("notification stream authority is unavailable")]
    Unavailable,
}

pub(super) async fn load(
    pool: &SqlitePool,
    claims: &AuthClaims,
    now: i64,
) -> Result<Snapshot, Error> {
    if claims.exp as i64 <= now
        || claims.user_auth_epoch < 1
        || uuid::Uuid::parse_str(&claims.sid).is_err()
    {
        return Err(Error::Unauthorized);
    }
    let mut tx = pool.begin().await.map_err(|_| Error::Unavailable)?;
    let identity = sqlx::query_as::<_, (String, i64)>(
        "SELECT u.username, p.authz_epoch FROM access_sessions s
         JOIN users u ON u.id=s.user_id CROSS JOIN access_policy_state p
         WHERE p.id=1 AND s.sid=? AND s.user_id=? AND s.revoked_at IS NULL
           AND s.expires_at>? AND s.auth_epoch_at_issue=? AND u.auth_epoch=?
           AND u.status=1 AND u.deleted_at IS NULL",
    )
    .bind(&claims.sid)
    .bind(claims.user_id)
    .bind(now)
    .bind(claims.user_auth_epoch)
    .bind(claims.user_auth_epoch)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| Error::Unavailable)?
    .filter(|(username, _)| username == &claims.username)
    .ok_or(Error::Unauthorized)?;
    let permissions = sqlx::query_scalar::<_, String>(
        "SELECT menu_code FROM user_permissions WHERE user_id=? ORDER BY menu_code",
    )
    .bind(claims.user_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| Error::Unavailable)?;
    let modules = sqlx::query_scalar::<_, String>(
        "SELECT id FROM modules WHERE id IN ('monitor','reports') AND enabled=1 ORDER BY id",
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| Error::Unavailable)?;
    let revision = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE((SELECT revision FROM notification_user_state WHERE user_id=?),0)",
    )
    .bind(claims.user_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| Error::Unavailable)?;
    tx.commit().await.map_err(|_| Error::Unavailable)?;
    let user = CurrentUser::new(claims.user_id, &claims.username, permissions.clone(), false);
    let permitted = modules.iter().any(|module| match module.as_str() {
        "monitor" => user.has_capability("monitor:incident:view"),
        "reports" => user.has_capability("reports:run:view"),
        _ => false,
    });
    if !permitted {
        return Err(Error::Forbidden);
    }
    Ok(Snapshot { revision, authz_epoch: identity.1, permissions, modules })
}
