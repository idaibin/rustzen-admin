use super::support::{TestDatabase, create_users, event, grant, policy};
use crate::features::notifications::{
    accounting::Accounting,
    admission::AdmissionService,
    admission_types::{AdmissionError, Failpoint},
    retention,
};
use chrono::Utc;
use std::time::Duration;

async fn accounting(database: &TestDatabase) -> Accounting {
    let mut connection = database.primary.acquire().await.unwrap();
    Accounting::current(&mut connection).await.unwrap()
}

#[tokio::test]
async fn cleanup_increments_each_user_once_and_preserves_current_state() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let old = now - chrono::Duration::days(31);
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    service.admit(&event("current", now, vec![2]), now).await.unwrap();
    service.admit(&event("old-a", old, vec![2]), old).await.unwrap();
    service.admit(&event("old-b", old, vec![2]), old).await.unwrap();
    let mut transaction = database.primary.begin_with("BEGIN IMMEDIATE").await.unwrap();
    let cleaned = retention::cleanup(&mut transaction, now, Duration::from_secs(5)).await.unwrap();
    transaction.commit().await.unwrap();
    assert_eq!((cleaned.recipients, cleaned.messages, cleaned.receipts), (2, 2, 2));
    assert_eq!(cleaned.states, 0);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT revision FROM notification_user_state WHERE user_id = 2"
        )
        .fetch_one(&database.primary)
        .await
        .unwrap(),
        4
    );
    database.close().await;
}

#[tokio::test]
async fn cleanup_preserves_durable_revision_state_until_user_cascade() {
    let database = TestDatabase::new().await;
    sqlx::query("INSERT INTO notification_user_state (user_id, revision) VALUES (2, 1)")
        .execute(&database.primary)
        .await
        .unwrap();
    let mut transaction = database.primary.begin_with("BEGIN IMMEDIATE").await.unwrap();
    let cleaned =
        retention::cleanup(&mut transaction, Utc::now().naive_utc(), Duration::from_secs(5))
            .await
            .unwrap();
    transaction.commit().await.unwrap();
    assert_eq!((cleaned.states, cleaned.charged_bytes), (0, 0));
    assert_eq!(accounting(&database).await.charged_bytes, 128);
    sqlx::query("DELETE FROM users WHERE id=2").execute(&database.primary).await.unwrap();
    assert_eq!(accounting(&database).await, Accounting::default());
    database.close().await;
}

#[tokio::test]
async fn committed_cleanup_survives_later_atomic_admission_failure() {
    let database = TestDatabase::new().await;
    create_users(&database.primary, 3, 702).await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let old = now - chrono::Duration::days(31);
    let seed = AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
        .await
        .unwrap();
    seed.admit(&event("old-fanout", old, (3..=702).collect()), old).await.unwrap();

    let failing =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap()
            .with_failpoint(Failpoint::AfterReceipt);
    assert!(matches!(
        failing.admit(&event("new-fails", now, vec![2]), now).await,
        Err(AdmissionError::Database(_))
    ));
    assert_eq!(accounting(&database).await.charged_bytes, 700 * 128);
    assert_eq!(
        sqlx::query_as::<_, (i64, i64, i64, i64)>(
            "SELECT
               (SELECT COUNT(*) FROM notification_receipts),
               (SELECT COUNT(*) FROM notifications),
               (SELECT COUNT(*) FROM notification_recipients),
               (SELECT COUNT(*) FROM notification_user_state)",
        )
        .fetch_one(&database.primary)
        .await
        .unwrap(),
        (0, 0, 0, 700)
    );
    database.close().await;
}
