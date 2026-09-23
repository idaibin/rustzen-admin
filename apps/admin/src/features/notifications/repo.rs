use super::types::{NotificationRow, ReadStateRow};
use crate::common::error::ServiceError;
use chrono::NaiveDateTime;
use sqlx::SqliteConnection;

pub struct NotificationRepository;

impl NotificationRepository {
    pub async fn enabled_user(
        connection: &mut SqliteConnection,
        user_id: i64,
    ) -> Result<bool, ServiceError> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = ? AND status = 1 AND deleted_at IS NULL)",
        )
        .bind(user_id)
        .fetch_one(connection)
        .await
        .map_err(database_error)
    }

    pub async fn max_accessible_seq(
        connection: &mut SqliteConnection,
        user_id: i64,
        unread_only: bool,
        cutoff: NaiveDateTime,
    ) -> Result<i64, ServiceError> {
        sqlx::query_scalar::<_, Option<i64>>(
            "SELECT MAX(n.inbox_seq)
             FROM notifications n
             INNER JOIN notification_recipients nr ON nr.notification_id = n.id
             WHERE nr.user_id = ? AND (? = 0 OR nr.read_at IS NULL)
               AND n.accepted_at > ?
               AND EXISTS (
                   SELECT 1 FROM user_permissions up
                   WHERE up.user_id = nr.user_id
                     AND (up.menu_code = '*'
                          OR up.menu_code = n.required_capability
                          OR (substr(up.menu_code, -2) = ':*'
                              AND n.required_capability LIKE substr(up.menu_code, 1, length(up.menu_code) - 1) || '%'))
               )
               AND EXISTS (SELECT 1 FROM modules m WHERE m.id = n.producer AND m.enabled = 1)",
        )
        .bind(user_id)
        .bind(unread_only)
        .bind(cutoff)
        .fetch_one(connection)
        .await
        .map(|value| value.unwrap_or(0))
        .map_err(database_error)
    }

    pub async fn list(
        connection: &mut SqliteConnection,
        user_id: i64,
        unread_only: bool,
        max_seq: i64,
        before_seq: Option<i64>,
        limit: i64,
        cutoff: NaiveDateTime,
    ) -> Result<Vec<NotificationRow>, ServiceError> {
        sqlx::query_as::<_, NotificationRow>(
            "SELECT n.id, n.inbox_seq, n.producer, n.topic, n.subject_kind, n.subject_id,
                    n.subject_revision, n.occurred_at, n.accepted_at, n.title, n.summary, nr.read_at
             FROM notifications n
             INNER JOIN notification_recipients nr ON nr.notification_id = n.id
             WHERE nr.user_id = ?
               AND n.inbox_seq <= ?
               AND (? IS NULL OR n.inbox_seq < ?)
               AND (? = 0 OR nr.read_at IS NULL)
               AND n.accepted_at > ?
               AND EXISTS (
                   SELECT 1 FROM user_permissions up
                   WHERE up.user_id = nr.user_id
                     AND (up.menu_code = '*'
                          OR up.menu_code = n.required_capability
                          OR (substr(up.menu_code, -2) = ':*'
                              AND n.required_capability LIKE substr(up.menu_code, 1, length(up.menu_code) - 1) || '%'))
               )
               AND EXISTS (SELECT 1 FROM modules m WHERE m.id = n.producer AND m.enabled = 1)
             ORDER BY n.inbox_seq DESC
             LIMIT ?",
        )
        .bind(user_id)
        .bind(max_seq)
        .bind(before_seq)
        .bind(before_seq)
        .bind(unread_only)
        .bind(cutoff)
        .bind(limit)
        .fetch_all(connection)
        .await
        .map_err(database_error)
    }

    pub async fn detail(
        connection: &mut SqliteConnection,
        user_id: i64,
        notification_id: &str,
        cutoff: NaiveDateTime,
    ) -> Result<Option<NotificationRow>, ServiceError> {
        sqlx::query_as::<_, NotificationRow>(
            "SELECT n.id, n.inbox_seq, n.producer, n.topic, n.subject_kind, n.subject_id,
                    n.subject_revision, n.occurred_at, n.accepted_at, n.title, n.summary, nr.read_at
             FROM notifications n
             INNER JOIN notification_recipients nr ON nr.notification_id = n.id
             WHERE nr.user_id = ? AND n.id = ? AND n.accepted_at > ?
               AND EXISTS (
                   SELECT 1 FROM user_permissions up
                   WHERE up.user_id = nr.user_id
                     AND (up.menu_code = '*'
                          OR up.menu_code = n.required_capability
                          OR (substr(up.menu_code, -2) = ':*'
                              AND n.required_capability LIKE substr(up.menu_code, 1, length(up.menu_code) - 1) || '%'))
               )
               AND EXISTS (SELECT 1 FROM modules m WHERE m.id = n.producer AND m.enabled = 1)",
        )
        .bind(user_id)
        .bind(notification_id)
        .bind(cutoff)
        .fetch_optional(connection)
        .await
        .map_err(database_error)
    }

    pub async fn unread_count(
        connection: &mut SqliteConnection,
        user_id: i64,
        cutoff: NaiveDateTime,
    ) -> Result<i64, ServiceError> {
        sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM notifications n
             INNER JOIN notification_recipients nr ON nr.notification_id = n.id
             WHERE nr.user_id = ? AND nr.read_at IS NULL AND n.accepted_at > ?
               AND EXISTS (
                   SELECT 1 FROM user_permissions up
                   WHERE up.user_id = nr.user_id
                     AND (up.menu_code = '*'
                          OR up.menu_code = n.required_capability
                          OR (substr(up.menu_code, -2) = ':*'
                              AND n.required_capability LIKE substr(up.menu_code, 1, length(up.menu_code) - 1) || '%'))
               )
               AND EXISTS (SELECT 1 FROM modules m WHERE m.id = n.producer AND m.enabled = 1)",
        )
        .bind(user_id)
        .bind(cutoff)
        .fetch_one(connection)
        .await
        .map_err(database_error)
    }

    pub async fn read_state(
        connection: &mut SqliteConnection,
        user_id: i64,
        notification_id: &str,
        cutoff: NaiveDateTime,
    ) -> Result<Option<ReadStateRow>, ServiceError> {
        sqlx::query_as::<_, ReadStateRow>(
            "SELECT n.id, nr.read_at
             FROM notifications n
             INNER JOIN notification_recipients nr ON nr.notification_id = n.id
             WHERE nr.user_id = ? AND n.id = ? AND n.accepted_at > ?
               AND EXISTS (
                   SELECT 1 FROM user_permissions up
                   WHERE up.user_id = nr.user_id
                     AND (up.menu_code = '*'
                          OR up.menu_code = n.required_capability
                          OR (substr(up.menu_code, -2) = ':*'
                              AND n.required_capability LIKE substr(up.menu_code, 1, length(up.menu_code) - 1) || '%'))
               )
               AND EXISTS (SELECT 1 FROM modules m WHERE m.id = n.producer AND m.enabled = 1)",
        )
        .bind(user_id)
        .bind(notification_id)
        .bind(cutoff)
        .fetch_optional(connection)
        .await
        .map_err(database_error)
    }

    pub async fn mark_read(
        connection: &mut SqliteConnection,
        user_id: i64,
        notification_id: &str,
        read_at: NaiveDateTime,
    ) -> Result<u64, ServiceError> {
        sqlx::query(
            "UPDATE notification_recipients SET read_at = ?
             WHERE notification_id = ? AND user_id = ? AND read_at IS NULL",
        )
        .bind(read_at)
        .bind(notification_id)
        .bind(user_id)
        .execute(connection)
        .await
        .map(|result| result.rows_affected())
        .map_err(database_error)
    }

    pub async fn mark_all_read(
        connection: &mut SqliteConnection,
        user_id: i64,
        unread_only: bool,
        max_seq: i64,
        read_at: NaiveDateTime,
        cutoff: NaiveDateTime,
    ) -> Result<u64, ServiceError> {
        sqlx::query(
            "UPDATE notification_recipients
             SET read_at = ?
             WHERE user_id = ? AND read_at IS NULL AND notification_id IN (
                SELECT n.id
                FROM notifications n
                INNER JOIN notification_recipients nr ON nr.notification_id = n.id
                WHERE nr.user_id = ? AND n.inbox_seq <= ?
                  AND (? = 0 OR nr.read_at IS NULL)
                  AND n.accepted_at > ?
                  AND EXISTS (
                      SELECT 1 FROM user_permissions up
                      WHERE up.user_id = nr.user_id
                        AND (up.menu_code = '*'
                             OR up.menu_code = n.required_capability
                             OR (substr(up.menu_code, -2) = ':*'
                                 AND n.required_capability LIKE substr(up.menu_code, 1, length(up.menu_code) - 1) || '%'))
                  )
                  AND EXISTS (SELECT 1 FROM modules m WHERE m.id = n.producer AND m.enabled = 1)
             )",
        )
        .bind(read_at)
        .bind(user_id)
        .bind(user_id)
        .bind(max_seq)
        .bind(unread_only)
        .bind(cutoff)
        .execute(connection)
        .await
        .map(|result| result.rows_affected())
        .map_err(database_error)
    }

    pub async fn revision(
        connection: &mut SqliteConnection,
        user_id: i64,
    ) -> Result<i64, ServiceError> {
        sqlx::query_scalar::<_, Option<i64>>(
            "SELECT revision FROM notification_user_state WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(connection)
        .await
        .map(|value| value.flatten().unwrap_or(0))
        .map_err(database_error)
    }

    pub async fn increment_revision(
        connection: &mut SqliteConnection,
        user_id: i64,
    ) -> Result<i64, ServiceError> {
        sqlx::query_scalar(
            "INSERT INTO notification_user_state (user_id, revision) VALUES (?, 1)
             ON CONFLICT(user_id) DO UPDATE SET revision = revision + 1
             RETURNING revision",
        )
        .bind(user_id)
        .fetch_one(connection)
        .await
        .map_err(database_error)
    }
}

fn database_error(error: sqlx::Error) -> ServiceError {
    tracing::error!(%error, "Notification database operation failed");
    ServiceError::DatabaseQueryFailed
}
