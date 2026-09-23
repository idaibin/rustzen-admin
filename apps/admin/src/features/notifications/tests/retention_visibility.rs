use super::support::{SECRET, TestDatabase, event, grant, policy, query};
use crate::{
    common::error::ServiceError,
    features::notifications::{
        admission::AdmissionService, maintenance, service::NotificationService,
        types::ReadAllRequest,
    },
};
use chrono::Utc;
use std::time::Duration;

async fn row_counts(database: &TestDatabase) -> (i64, i64, i64, i64) {
    sqlx::query_as(
        "SELECT
           (SELECT COUNT(*) FROM notification_receipts),
           (SELECT COUNT(*) FROM notifications),
           (SELECT COUNT(*) FROM notification_recipients),
           (SELECT COUNT(*) FROM notification_user_state)",
    )
    .fetch_one(&database.primary)
    .await
    .unwrap()
}

#[tokio::test]
async fn all_five_read_entries_hide_expired_rows_before_physical_cleanup() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let old = now - chrono::Duration::days(31);
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    let stored = service.admit(&event("quiet-expired", old, vec![2]), old).await.unwrap();
    let notification_id = match stored {
        crate::features::notifications::admission_types::AdmissionResult::Stored {
            notification_id,
            ..
        } => notification_id,
        other => panic!("unexpected result: {other:?}"),
    };

    let list = NotificationService::list_at(&database.primary, 2, query(None, None), SECRET, now)
        .await
        .unwrap();
    assert!(list.items.is_empty());
    assert_eq!(
        NotificationService::unread_count_at(&database.primary, 2, now).await.unwrap().count,
        0
    );
    assert!(matches!(
        NotificationService::detail_at(&database.primary, 2, &notification_id, now).await,
        Err(ServiceError::NotFound(_))
    ));
    assert!(matches!(
        NotificationService::mark_read_at(&database.primary, 2, &notification_id, now).await,
        Err(ServiceError::NotFound(_))
    ));
    let read_all = NotificationService::mark_all_read_at(
        &database.primary,
        2,
        ReadAllRequest { snapshot: list.snapshot },
        SECRET,
        now,
    )
    .await
    .unwrap();
    assert_eq!(read_all.changed, 0);
    assert_eq!(row_counts(&database).await, (1, 1, 1, 1));
    database.close().await;
}

#[tokio::test]
async fn selected_runtime_reclaims_on_startup_and_periodic_ticks() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let old = now - chrono::Duration::days(31);
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    service.admit(&event("startup-expired", old, vec![2]), old).await.unwrap();

    let task =
        maintenance::start_for_test(database.primary.clone(), Duration::from_millis(20), now, None)
            .await
            .unwrap();
    assert_eq!(row_counts(&database).await, (0, 0, 0, 1));

    service.admit(&event("periodic-expired", old, vec![2]), old).await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if row_counts(&database).await == (0, 0, 0, 1) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("periodic cleanup must reclaim expired rows");
    task.shutdown().await;
    database.close().await;
}
