use super::{
    cursor::{self, CursorKind},
    repo::NotificationRepository,
    service::{
        NotificationService, database_error, ensure_enabled_user, finish_write, retention_cutoff,
    },
    types::{ReadAllRequest, ReadAllResponse, ReadResponse},
};
use crate::common::error::ServiceError;
use chrono::{NaiveDateTime, Utc};
use sqlx::SqlitePool;

impl NotificationService {
    #[cfg(test)]
    pub async fn mark_read(
        pool: &SqlitePool,
        user_id: i64,
        notification_id: &str,
    ) -> Result<ReadResponse, ServiceError> {
        Self::mark_read_for_http(pool, user_id, notification_id).await.map(|(response, _)| response)
    }

    pub(crate) async fn mark_read_for_http(
        pool: &SqlitePool,
        user_id: i64,
        notification_id: &str,
    ) -> Result<(ReadResponse, bool), ServiceError> {
        Self::mark_read_at_with_change(pool, user_id, notification_id, Utc::now().naive_utc()).await
    }

    #[cfg(test)]
    pub(super) async fn mark_read_at(
        pool: &SqlitePool,
        user_id: i64,
        notification_id: &str,
        now: NaiveDateTime,
    ) -> Result<ReadResponse, ServiceError> {
        Self::mark_read_at_with_change(pool, user_id, notification_id, now)
            .await
            .map(|(response, _)| response)
    }

    async fn mark_read_at_with_change(
        pool: &SqlitePool,
        user_id: i64,
        notification_id: &str,
        now: NaiveDateTime,
    ) -> Result<(ReadResponse, bool), ServiceError> {
        let mut connection = pool.acquire().await.map_err(database_error)?;
        sqlx::query("BEGIN IMMEDIATE").execute(&mut *connection).await.map_err(database_error)?;
        let result = async {
            ensure_enabled_user(&mut connection, user_id).await?;
            let current = NotificationRepository::read_state(
                &mut connection,
                user_id,
                notification_id,
                retention_cutoff(now),
            )
            .await?
            .ok_or_else(|| ServiceError::NotFound("Notification".into()))?;
            let (read_at, revision, changed) = if let Some(read_at) = current.read_at {
                (read_at, NotificationRepository::revision(&mut connection, user_id).await?, false)
            } else {
                let read_at = now;
                if NotificationRepository::mark_read(
                    &mut connection,
                    user_id,
                    notification_id,
                    read_at,
                )
                .await?
                    != 1
                {
                    return Err(ServiceError::InvalidOperation("Inbox state changed".into()));
                }
                let revision =
                    NotificationRepository::increment_revision(&mut connection, user_id).await?;
                (read_at, revision, true)
            };
            Ok((ReadResponse { id: current.id, read_at, revision }, changed))
        }
        .await;
        finish_write(&mut connection, result.is_ok()).await?;
        result
    }

    pub async fn mark_all_read(
        pool: &SqlitePool,
        user_id: i64,
        request: ReadAllRequest,
        cursor_secret: &[u8],
    ) -> Result<ReadAllResponse, ServiceError> {
        Self::mark_all_read_at(pool, user_id, request, cursor_secret, Utc::now().naive_utc()).await
    }

    pub(super) async fn mark_all_read_at(
        pool: &SqlitePool,
        user_id: i64,
        request: ReadAllRequest,
        cursor_secret: &[u8],
        now: NaiveDateTime,
    ) -> Result<ReadAllResponse, ServiceError> {
        let snapshot = cursor::decode(&request.snapshot, cursor_secret)?;
        if snapshot.kind != CursorKind::Snapshot || snapshot.user_id != user_id {
            return Err(ServiceError::InvalidOperation(
                "Inbox snapshot does not match the current user".into(),
            ));
        }
        let mut connection = pool.acquire().await.map_err(database_error)?;
        sqlx::query("BEGIN IMMEDIATE").execute(&mut *connection).await.map_err(database_error)?;
        let result = async {
            ensure_enabled_user(&mut connection, user_id).await?;
            let changed = NotificationRepository::mark_all_read(
                &mut connection,
                user_id,
                snapshot.unread_only,
                snapshot.max_seq,
                now,
                retention_cutoff(now),
            )
            .await?;
            let revision = if changed > 0 {
                NotificationRepository::increment_revision(&mut connection, user_id).await?
            } else {
                NotificationRepository::revision(&mut connection, user_id).await?
            };
            Ok(ReadAllResponse { changed, revision })
        }
        .await;
        finish_write(&mut connection, result.is_ok()).await?;
        result
    }
}
