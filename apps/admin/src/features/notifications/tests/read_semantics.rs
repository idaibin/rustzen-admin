use super::support::{SECRET, TestDatabase, grant, insert_notification, query, revoke_all};
use crate::features::notifications::{
    service::NotificationService,
    types::{InboxListQuery, ReadAllRequest},
};
use std::sync::Arc;
use tokio::sync::Barrier;

#[tokio::test]
async fn reads_recheck_user_grant_and_module_in_one_database_snapshot() {
    let database = TestDatabase::new().await;
    insert_notification(&database.primary, "only-owner", 1).await;
    insert_notification(&database.primary, "admin-message", 2).await;

    // A cached/supplied wildcard cannot make user 2 visible without a current DB grant.
    assert!(
        NotificationService::list(&database.primary, 2, query(None, None), SECRET)
            .await
            .unwrap()
            .items
            .is_empty()
    );
    grant(&database.primary, "monitor:incident:*").await;
    let visible =
        NotificationService::list(&database.primary, 2, query(None, None), SECRET).await.unwrap();
    assert_eq!(
        visible.items.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(),
        ["admin-message"]
    );
    assert!(NotificationService::detail(&database.primary, 2, "only-owner").await.is_err());

    revoke_all(&database.primary).await;
    assert_eq!(NotificationService::unread_count(&database.primary, 2).await.unwrap().count, 0);
    assert!(NotificationService::mark_read(&database.primary, 2, "admin-message").await.is_err());
    grant(&database.primary, "monitor:incident:*").await;
    let snapshot = NotificationService::list(&database.primary, 2, query(None, None), SECRET)
        .await
        .unwrap()
        .snapshot;
    sqlx::query("UPDATE modules SET enabled = 0 WHERE id = 'monitor'")
        .execute(&database.primary)
        .await
        .unwrap();
    assert!(NotificationService::detail(&database.primary, 1, "only-owner").await.is_err());
    assert!(NotificationService::mark_read(&database.primary, 1, "only-owner").await.is_err());
    assert_eq!(
        NotificationService::mark_all_read(
            &database.primary,
            2,
            ReadAllRequest { snapshot },
            SECRET,
        )
        .await
        .unwrap()
        .changed,
        0
    );
    assert!(
        sqlx::query_scalar::<_, Option<String>>(
            "SELECT read_at FROM notification_recipients
             WHERE notification_id = 'admin-message' AND user_id = 2",
        )
        .fetch_one(&database.primary)
        .await
        .unwrap()
        .is_none()
    );
    sqlx::query("UPDATE users SET status = 2 WHERE id = 2")
        .execute(&database.primary)
        .await
        .unwrap();
    assert!(matches!(
        NotificationService::list(&database.primary, 2, query(None, None), SECRET).await,
        Err(crate::common::error::ServiceError::UserIsDisabled)
    ));
    database.close().await;
}
#[tokio::test]
async fn encrypted_cursor_binds_user_filter_integrity_and_stable_page_boundary() {
    let database = TestDatabase::new().await;
    for id in ["first", "second", "third"] {
        insert_notification(&database.primary, id, 1).await;
    }
    let first = NotificationService::list(&database.primary, 1, query(Some(2), None), SECRET)
        .await
        .unwrap();
    assert_eq!(
        first.items.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(),
        ["third", "second"]
    );
    let cursor = first.next_cursor.clone().expect("next cursor");
    let mut tampered = cursor.clone().into_bytes();
    let middle = tampered.len() / 2;
    tampered[middle] = if tampered[middle] == b'a' { b'b' } else { b'a' };
    assert!(
        NotificationService::list(
            &database.primary,
            1,
            query(Some(2), Some(String::from_utf8(tampered).unwrap())),
            SECRET,
        )
        .await
        .is_err()
    );
    assert!(
        NotificationService::list(
            &database.primary,
            1,
            InboxListQuery {
                cursor: Some(cursor.clone()),
                limit: Some(2),
                unread_only: Some(true),
            },
            SECRET,
        )
        .await
        .is_err()
    );
    grant(&database.primary, "monitor:incident:*").await;
    assert!(
        NotificationService::list(
            &database.primary,
            2,
            query(Some(2), Some(cursor.clone())),
            SECRET,
        )
        .await
        .is_err()
    );
    insert_notification(&database.peer, "newer", 1).await;
    let second = NotificationService::list(&database.peer, 1, query(Some(2), Some(cursor)), SECRET)
        .await
        .unwrap();
    assert_eq!(second.items.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(), ["first"]);
    assert!(
        NotificationService::list(
            &database.primary,
            2,
            query(Some(2), Some(first.snapshot)),
            SECRET,
        )
        .await
        .is_err()
    );
    database.close().await;
}

#[tokio::test]
async fn concurrent_single_read_preserves_first_timestamp_and_increments_once() {
    let database = TestDatabase::new().await;
    insert_notification(&database.primary, "race", 1).await;
    let barrier = Arc::new(Barrier::new(3));
    let left_pool = database.primary.clone();
    let left_barrier = barrier.clone();
    let left = tokio::spawn(async move {
        left_barrier.wait().await;
        NotificationService::mark_read(&left_pool, 1, "race").await
    });
    let right_pool = database.peer.clone();
    let right_barrier = barrier.clone();
    let right = tokio::spawn(async move {
        right_barrier.wait().await;
        NotificationService::mark_read(&right_pool, 1, "race").await
    });
    barrier.wait().await;
    let left = left.await.unwrap().unwrap();
    let right = right.await.unwrap().unwrap();
    assert_eq!(left.read_at, right.read_at);
    assert_eq!(left.revision, 2);
    assert_eq!(right.revision, 2);
    database.close().await;
}

#[tokio::test]
async fn concurrent_read_all_does_not_consume_arrivals_after_its_snapshot() {
    let database = TestDatabase::new().await;
    insert_notification(&database.primary, "old", 1).await;
    let snapshot = NotificationService::list(&database.primary, 1, query(None, None), SECRET)
        .await
        .unwrap()
        .snapshot;
    let barrier = Arc::new(Barrier::new(3));
    let read_pool = database.primary.clone();
    let read_barrier = barrier.clone();
    let read = tokio::spawn(async move {
        read_barrier.wait().await;
        NotificationService::mark_all_read(&read_pool, 1, ReadAllRequest { snapshot }, SECRET).await
    });
    let insert_pool = database.peer.clone();
    let insert_barrier = barrier.clone();
    let insert = tokio::spawn(async move {
        insert_barrier.wait().await;
        insert_notification(&insert_pool, "new", 1).await;
    });
    barrier.wait().await;
    assert_eq!(read.await.unwrap().unwrap().changed, 1);
    insert.await.unwrap();
    assert_eq!(NotificationService::unread_count(&database.primary, 1).await.unwrap().count, 1);
    assert!(
        NotificationService::detail(&database.primary, 1, "old").await.unwrap().read_at.is_some()
    );
    assert!(
        NotificationService::detail(&database.primary, 1, "new").await.unwrap().read_at.is_none()
    );
    database.close().await;
}
