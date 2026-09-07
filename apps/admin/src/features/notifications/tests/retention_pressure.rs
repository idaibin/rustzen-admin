use super::support::{TestDatabase, create_users, event, grant, policy};
use crate::features::notifications::{
    accounting::Accounting,
    admission::AdmissionService,
    admission_types::{AdmissionError, AdmissionResult, CapacityReason},
    pressure::SpaceProbe,
    retention,
};
use chrono::Utc;
use std::{io, path::Path, sync::Arc, time::Duration};

struct FixedSpace(u64);

impl SpaceProbe for FixedSpace {
    fn available(&self, _path: &Path) -> io::Result<u64> {
        Ok(self.0)
    }
}

async fn accounting(database: &TestDatabase) -> Accounting {
    let mut connection = database.primary.acquire().await.unwrap();
    Accounting::current(&mut connection).await.unwrap()
}

#[tokio::test]
async fn cleanup_preserves_receipt_horizon_and_prevents_expired_recreation() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let accepted_at = now - chrono::Duration::days(31);
    let original = event("retained-receipt", accepted_at, vec![2]);
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    service.admit(&original, accepted_at).await.unwrap();

    let mut transaction = database.primary.begin_with("BEGIN IMMEDIATE").await.unwrap();
    let cleaned = retention::cleanup(&mut transaction, now, Duration::from_secs(5)).await.unwrap();
    transaction.commit().await.unwrap();
    assert_eq!((cleaned.recipients, cleaned.messages, cleaned.receipts), (1, 1, 1));
    assert_eq!(accounting(&database).await.receipt_count, 0);
    assert!(matches!(service.admit(&original, now).await, Err(AdmissionError::Expired)));
    assert_eq!(accounting(&database).await.message_count, 0);
    database.close().await;
}

#[tokio::test]
async fn future_retain_until_survives_message_cleanup_and_accepts_duplicate() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let accepted_at = now - chrono::Duration::days(31);
    let mut original = event("future-receipt", accepted_at, vec![2]);
    original.expires_at = now + chrono::Duration::days(2);
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    service.admit(&original, accepted_at).await.unwrap();
    let mut transaction = database.primary.begin_with("BEGIN IMMEDIATE").await.unwrap();
    let cleaned = retention::cleanup(&mut transaction, now, Duration::from_secs(5)).await.unwrap();
    transaction.commit().await.unwrap();
    assert_eq!((cleaned.recipients, cleaned.messages, cleaned.receipts), (1, 1, 0));
    assert_eq!(accounting(&database).await.receipt_count, 1);
    assert_eq!(
        service.admit(&original, now).await.unwrap(),
        AdmissionResult::Duplicate { result: "stored".into() }
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notification_user_state")
            .fetch_one(&database.primary)
            .await
            .unwrap(),
        0
    );
    database.close().await;
}

#[tokio::test]
async fn saturated_admission_commits_cleanup_rounds_then_event_atomically() {
    let database = TestDatabase::new().await;
    create_users(&database.primary, 3, 702).await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let accepted_at = now - chrono::Duration::days(31);
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    service
        .admit(&event("bounded-cleanup", accepted_at, (3..=702).collect()), accepted_at)
        .await
        .unwrap();
    let saturated = AdmissionService::start(
        database.primary.clone(),
        database.path.clone(),
        crate::features::notifications::admission_types::AdmissionPolicy {
            message_limit: 1,
            recipient_limit: 1,
            receipt_limit: 1,
            charged_bytes_limit: 1920,
            ..policy()
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        saturated.admit(&event("after-cleanup", now, vec![2]), now).await.unwrap(),
        AdmissionResult::Stored { recipients: 1, .. }
    ));
    assert_eq!(
        accounting(&database).await,
        Accounting { message_count: 1, recipient_count: 1, receipt_count: 1, charged_bytes: 1920 }
    );
    let reopened = database.reopen().await;
    let mut connection = reopened.acquire().await.unwrap();
    Accounting::validate(&mut connection).await.unwrap();
    drop(connection);
    reopened.close().await;
    database.close().await;
}

#[tokio::test]
async fn cleanup_honors_charge_and_time_limits() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let accepted_at = now - chrono::Duration::days(31);
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    service.admit(&event("bounded-resources", accepted_at, vec![1, 2]), accepted_at).await.unwrap();
    let mut transaction = database.primary.begin_with("BEGIN IMMEDIATE").await.unwrap();
    let none = retention::cleanup_with_limits(
        &mut transaction,
        now,
        retention::CleanupLimits {
            rows: 500,
            charged_bytes: 4 * 1024 * 1024,
            time: Duration::ZERO,
        },
    )
    .await
    .unwrap();
    assert_eq!(none, retention::CleanupResult::default());
    let one = retention::cleanup_with_limits(
        &mut transaction,
        now,
        retention::CleanupLimits { rows: 500, charged_bytes: 256, time: Duration::from_secs(5) },
    )
    .await
    .unwrap();
    transaction.commit().await.unwrap();
    assert_eq!((one.recipients, one.messages, one.charged_bytes), (1, 0, 256));
    assert_eq!(accounting(&database).await.recipient_count, 1);
    database.close().await;
}

#[tokio::test]
async fn user_cascade_releases_recipient_and_state_accounting() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    service.admit(&event("cascade", now, vec![2]), now).await.unwrap();
    let before = accounting(&database).await;
    sqlx::query("DELETE FROM users WHERE id = 2").execute(&database.primary).await.unwrap();
    let after = accounting(&database).await;
    assert_eq!(after.recipient_count, 0);
    assert_eq!(after.message_count, 1);
    assert_eq!(after.charged_bytes, before.charged_bytes - 256 - 128);
    let mut connection = database.primary.acquire().await.unwrap();
    Accounting::validate(&mut connection).await.unwrap();
    drop(connection);
    assert!(
        sqlx::query("UPDATE notifications SET charged_bytes = 2048")
            .execute(&database.primary)
            .await
            .is_err()
    );
    database.close().await;
}

#[tokio::test]
async fn free_space_reserve_refuses_only_notification_admission() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let service = AdmissionService::start_for_test(
        database.primary.clone(),
        database.path.clone(),
        crate::features::notifications::admission_types::AdmissionPolicy {
            free_space_reserve_bytes: 1_000,
            ..policy()
        },
        Arc::new(FixedSpace(2_919)),
    )
    .await
    .unwrap();
    let now = Utc::now().naive_utc();
    assert!(matches!(
        service.admit(&event("no-space", now, vec![2]), now).await,
        Err(AdmissionError::Capacity { reason: CapacityReason::FreeSpace, .. })
    ));
    sqlx::query("UPDATE users SET real_name = 'unrelated writer' WHERE id = 1")
        .execute(&database.peer)
        .await
        .expect("other feature writer is not gated by notification admission");
    assert_eq!(accounting(&database).await, Accounting::default());
    database.close().await;
}

#[tokio::test]
async fn long_reader_creates_checkpoint_pressure_without_global_write_gate() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    sqlx::query("PRAGMA wal_autocheckpoint = 0").execute(&database.primary).await.unwrap();
    sqlx::query("UPDATE users SET real_name = 'wal-base' WHERE id = 1")
        .execute(&database.primary)
        .await
        .unwrap();
    let mut reader = database.peer.begin().await.unwrap();
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *reader)
        .await
        .unwrap();
    for index in 0..8 {
        sqlx::query("UPDATE users SET real_name = ? WHERE id = 1")
            .bind(format!("wal-{index}"))
            .execute(&database.primary)
            .await
            .unwrap();
    }
    let service = AdmissionService::start(
        database.primary.clone(),
        database.path.clone(),
        crate::features::notifications::admission_types::AdmissionPolicy {
            wal_pressure_frames: 1,
            wal_pressure_observations: 1,
            ..policy()
        },
    )
    .await
    .unwrap();
    let now = Utc::now().naive_utc();
    assert!(matches!(
        service.admit(&event("wal-pressure", now, vec![2]), now).await,
        Err(AdmissionError::Capacity { reason: CapacityReason::Checkpoint, .. })
    ));
    sqlx::query("UPDATE users SET real_name = 'still-writable' WHERE id = 2")
        .execute(&database.primary)
        .await
        .expect("non-notification writer remains outside the admission gate");
    reader.rollback().await.unwrap();
    database.close().await;
}
