use super::support::{TestDatabase, create_users, event, grant, policy};
use crate::features::notifications::{
    accounting::Accounting,
    admission::AdmissionService,
    admission_types::{AdmissionError, AdmissionPolicy, AdmissionResult, CapacityReason},
    pressure::SpaceProbe,
};
use chrono::Utc;
use std::{
    io,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::sync::Barrier;

async fn pair(
    database: &TestDatabase,
    left: crate::features::notifications::admission_types::AdmissionEvent,
    right: crate::features::notifications::admission_types::AdmissionEvent,
    admission_policy: AdmissionPolicy,
) -> (Result<AdmissionResult, AdmissionError>, Result<AdmissionResult, AdmissionError>) {
    let left_service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), admission_policy)
            .await
            .unwrap();
    let right_service =
        AdmissionService::start(database.peer.clone(), database.path.clone(), admission_policy)
            .await
            .unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let now = Utc::now().naive_utc();
    let left_barrier = barrier.clone();
    let left_task = tokio::spawn(async move {
        left_barrier.wait().await;
        left_service.admit(&left, now).await
    });
    let right_barrier = barrier.clone();
    let right_task = tokio::spawn(async move {
        right_barrier.wait().await;
        right_service.admit(&right, now).await
    });
    barrier.wait().await;
    (left_task.await.unwrap(), right_task.await.unwrap())
}

async fn assert_durable(database: &TestDatabase, expected: Accounting, revision: i64) {
    let counts = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        "SELECT
           (SELECT COUNT(*) FROM notification_receipts),
           (SELECT COUNT(*) FROM notifications),
           (SELECT COUNT(*) FROM notification_recipients),
           (SELECT COUNT(*) FROM notification_user_state)",
    )
    .fetch_one(&database.primary)
    .await
    .unwrap();
    assert_eq!(
        counts,
        (expected.receipt_count, expected.message_count, expected.recipient_count, 1)
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT revision FROM notification_user_state WHERE user_id = 2"
        )
        .fetch_one(&database.primary)
        .await
        .unwrap(),
        revision
    );
    let reopened = database.reopen().await;
    let mut connection = reopened.acquire().await.unwrap();
    assert_eq!(Accounting::current(&mut connection).await.unwrap(), expected);
    Accounting::validate(&mut connection).await.unwrap();
    drop(connection);
    reopened.close().await;
}

#[tokio::test]
async fn two_pools_same_digest_store_once_and_return_duplicate() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let input = event("concurrent-same", now, vec![2]);
    let (left, right) = pair(&database, input.clone(), input, policy()).await;
    assert!(matches!(
        (&left, &right),
        (Ok(AdmissionResult::Stored { .. }), Ok(AdmissionResult::Duplicate { .. }))
            | (Ok(AdmissionResult::Duplicate { .. }), Ok(AdmissionResult::Stored { .. }))
    ));
    assert_durable(
        &database,
        Accounting { message_count: 1, recipient_count: 1, receipt_count: 1, charged_bytes: 1920 },
        1,
    )
    .await;
    database.close().await;
}

#[tokio::test]
async fn two_pools_different_digest_store_once_and_conflict() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let left = event("concurrent-conflict", now, vec![2]);
    let mut right = left.clone();
    right.payload_sha256 = "b".repeat(64);
    let (left, right) = pair(&database, left, right, policy()).await;
    assert!(matches!(
        (&left, &right),
        (Ok(AdmissionResult::Stored { .. }), Err(AdmissionError::Conflict))
            | (Err(AdmissionError::Conflict), Ok(AdmissionResult::Stored { .. }))
    ));
    assert_durable(
        &database,
        Accounting { message_count: 1, recipient_count: 1, receipt_count: 1, charged_bytes: 1920 },
        1,
    )
    .await;
    database.close().await;
}

#[tokio::test]
async fn two_pools_at_n_minus_one_admit_exactly_one_event() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let constrained = AdmissionPolicy {
        message_limit: 1,
        recipient_limit: 1,
        receipt_limit: 1,
        charged_bytes_limit: 1920,
        ..policy()
    };
    let (left, right) = pair(
        &database,
        event("capacity-left", now, vec![2]),
        event("capacity-right", now, vec![2]),
        constrained,
    )
    .await;
    assert!(matches!(
        (&left, &right),
        (Ok(AdmissionResult::Stored { .. }), Err(AdmissionError::Capacity { .. }))
            | (Err(AdmissionError::Capacity { .. }), Ok(AdmissionResult::Stored { .. }))
    ));
    assert_durable(
        &database,
        Accounting { message_count: 1, recipient_count: 1, receipt_count: 1, charged_bytes: 1920 },
        1,
    )
    .await;
    database.close().await;
}

struct BoundarySpace {
    calls: AtomicUsize,
}

impl SpaceProbe for BoundarySpace {
    fn available(&self, _path: &Path) -> io::Result<u64> {
        Ok(if self.calls.fetch_add(1, Ordering::SeqCst) == 0 { 2_920 } else { 2_919 })
    }
}

#[tokio::test]
async fn two_pools_recheck_projected_space_while_serialized_by_write_lock() {
    let database = TestDatabase::new().await;
    create_users(&database.primary, 3, 3).await;
    grant(&database.primary, "monitor:incident:view").await;
    let probe = Arc::new(BoundarySpace { calls: AtomicUsize::new(0) });
    let admission_policy = AdmissionPolicy { free_space_reserve_bytes: 1_000, ..policy() };
    let left = AdmissionService::start_for_test(
        database.primary.clone(),
        database.path.clone(),
        admission_policy,
        probe.clone(),
    )
    .await
    .unwrap();
    let right = AdmissionService::start_for_test(
        database.peer.clone(),
        database.path.clone(),
        admission_policy,
        probe,
    )
    .await
    .unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let now = Utc::now().naive_utc();
    let left_barrier = barrier.clone();
    let left_task = tokio::spawn(async move {
        left_barrier.wait().await;
        left.admit(&event("space-left", now, vec![2]), now).await
    });
    let right_barrier = barrier.clone();
    let right_task = tokio::spawn(async move {
        right_barrier.wait().await;
        right.admit(&event("space-right", now, vec![3]), now).await
    });
    barrier.wait().await;
    let outcomes = (left_task.await.unwrap(), right_task.await.unwrap());
    assert!(matches!(
        outcomes,
        (
            Ok(AdmissionResult::Stored { .. }),
            Err(AdmissionError::Capacity { reason: CapacityReason::FreeSpace, .. })
        ) | (
            Err(AdmissionError::Capacity { reason: CapacityReason::FreeSpace, .. }),
            Ok(AdmissionResult::Stored { .. })
        )
    ));
    let mut connection = database.primary.acquire().await.unwrap();
    Accounting::validate(&mut connection).await.unwrap();
    assert_eq!(Accounting::current(&mut connection).await.unwrap().charged_bytes, 1920);
    drop(connection);
    database.close().await;
}
