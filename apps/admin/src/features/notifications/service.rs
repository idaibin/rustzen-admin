use super::{
    cursor::{self, CursorKind, CursorState},
    repo::NotificationRepository,
    types::{
        InboxListQuery, InboxListResponse, NotificationItem, ReadAllRequest, ReadAllResponse,
        ReadResponse, UnreadCountResponse,
    },
};
use crate::common::error::ServiceError;
use chrono::Utc;
use sqlx::SqlitePool;

const DEFAULT_PAGE_SIZE: u16 = 20;
const MAX_PAGE_SIZE: u16 = 100;
const RETENTION_DAYS: u16 = 30;

pub struct NotificationService;

impl NotificationService {
    pub async fn list(
        pool: &SqlitePool,
        user_id: i64,
        query: InboxListQuery,
        cursor_secret: &[u8],
    ) -> Result<InboxListResponse, ServiceError> {
        let limit = query.limit.unwrap_or(DEFAULT_PAGE_SIZE);
        if !(1..=MAX_PAGE_SIZE).contains(&limit) {
            return Err(ServiceError::InvalidOperation(
                "Inbox limit must be between 1 and 100".into(),
            ));
        }
        let unread_only = query.unread_only.unwrap_or(false);
        let mut transaction = pool.begin().await.map_err(database_error)?;
        ensure_enabled_user(&mut transaction, user_id).await?;
        let (max_seq, before_seq) = if let Some(token) = query.cursor.as_deref() {
            let state = cursor::decode(token, cursor_secret)?;
            if state.kind != CursorKind::Page
                || state.user_id != user_id
                || state.unread_only != unread_only
            {
                return Err(ServiceError::InvalidOperation(
                    "Inbox cursor does not match the current user or filter".into(),
                ));
            }
            (state.max_seq, state.before_seq)
        } else {
            (
                NotificationRepository::max_accessible_seq(&mut transaction, user_id, unread_only)
                    .await?,
                None,
            )
        };
        let mut rows = NotificationRepository::list(
            &mut transaction,
            user_id,
            unread_only,
            max_seq,
            before_seq,
            i64::from(limit) + 1,
        )
        .await?;
        let has_more = rows.len() > usize::from(limit);
        if has_more {
            rows.truncate(usize::from(limit));
        }
        let next_cursor = if has_more {
            rows.last()
                .map(|item| {
                    cursor::encode(
                        &CursorState {
                            version: 1,
                            kind: CursorKind::Page,
                            user_id,
                            unread_only,
                            max_seq,
                            before_seq: Some(item.inbox_seq),
                        },
                        cursor_secret,
                    )
                })
                .transpose()?
        } else {
            None
        };
        let snapshot = cursor::encode(
            &CursorState {
                version: 1,
                kind: CursorKind::Snapshot,
                user_id,
                unread_only,
                max_seq,
                before_seq: None,
            },
            cursor_secret,
        )?;
        let revision = NotificationRepository::revision(&mut transaction, user_id).await?;
        transaction.commit().await.map_err(database_error)?;
        let items = rows.into_iter().map(NotificationItem::from).collect();
        Ok(InboxListResponse {
            items,
            next_cursor,
            snapshot,
            revision,
            retention_days: RETENTION_DAYS,
        })
    }

    pub async fn unread_count(
        pool: &SqlitePool,
        user_id: i64,
    ) -> Result<UnreadCountResponse, ServiceError> {
        let mut transaction = pool.begin().await.map_err(database_error)?;
        ensure_enabled_user(&mut transaction, user_id).await?;
        let count = NotificationRepository::unread_count(&mut transaction, user_id).await?;
        let revision = NotificationRepository::revision(&mut transaction, user_id).await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(UnreadCountResponse { count, revision })
    }

    pub async fn detail(
        pool: &SqlitePool,
        user_id: i64,
        notification_id: &str,
    ) -> Result<NotificationItem, ServiceError> {
        let mut transaction = pool.begin().await.map_err(database_error)?;
        ensure_enabled_user(&mut transaction, user_id).await?;
        let item = NotificationRepository::detail(&mut transaction, user_id, notification_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Notification".into()))?;
        transaction.commit().await.map_err(database_error)?;
        Ok(item.into())
    }

    pub async fn mark_read(
        pool: &SqlitePool,
        user_id: i64,
        notification_id: &str,
    ) -> Result<ReadResponse, ServiceError> {
        let mut connection = pool.acquire().await.map_err(database_error)?;
        sqlx::query("BEGIN IMMEDIATE").execute(&mut *connection).await.map_err(database_error)?;
        let result = async {
            ensure_enabled_user(&mut connection, user_id).await?;
            let current =
                NotificationRepository::read_state(&mut connection, user_id, notification_id)
                    .await?
                    .ok_or_else(|| ServiceError::NotFound("Notification".into()))?;
            let (read_at, revision) = if let Some(read_at) = current.read_at {
                (read_at, NotificationRepository::revision(&mut connection, user_id).await?)
            } else {
                let read_at = Utc::now().naive_utc();
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
                (read_at, revision)
            };
            Ok(ReadResponse { id: current.id, read_at, revision })
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
                Utc::now().naive_utc(),
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

async fn ensure_enabled_user(
    connection: &mut sqlx::SqliteConnection,
    user_id: i64,
) -> Result<(), ServiceError> {
    if NotificationRepository::enabled_user(connection, user_id).await? {
        Ok(())
    } else {
        Err(ServiceError::UserIsDisabled)
    }
}

async fn finish_write(
    connection: &mut sqlx::SqliteConnection,
    commit: bool,
) -> Result<(), ServiceError> {
    sqlx::query(if commit { "COMMIT" } else { "ROLLBACK" })
        .execute(connection)
        .await
        .map(|_| ())
        .map_err(database_error)
}

fn database_error(error: sqlx::Error) -> ServiceError {
    tracing::error!(%error, "Notification transaction failed");
    ServiceError::DatabaseQueryFailed
}
