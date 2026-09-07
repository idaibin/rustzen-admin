use super::support::TestDatabase;
use crate::notifications::diagnostics::{inspect_after_commit_at, test_limiter};
use crate::notifications::relay::{Claim, DeliveryResult, Transport, claim, finish, run_once_at};
use chrono::{Duration, TimeZone, Utc};
use sha2::{Digest, Sha256};
use std::{future::Future, pin::Pin};

struct Stored;

impl Transport for Stored {
    fn send<'a>(
        &'a self,
        _claim: &'a Claim,
    ) -> Pin<Box<dyn Future<Output = DeliveryResult> + Send + 'a>> {
        Box::pin(async { DeliveryResult::Stored })
    }
}

async fn insert_outbox(
    database: &TestDatabase,
    event_id: &str,
    occurred_at: chrono::DateTime<Utc>,
    expires_at: chrono::DateTime<Utc>,
    charged_bytes: i64,
) {
    let payload = format!(r#"{{"eventId":"{event_id}"}}"#);
    let digest = hex::encode(Sha256::digest(payload.as_bytes()));
    sqlx::query(
        "INSERT INTO notification_outbox
         (event_id,topic,subject_kind,subject_id,subject_revision,payload_json,payload_sha256,
          occurred_at,expires_at,state,next_attempt_at,charged_bytes)
         VALUES(?,'monitor.incident.opened','monitor-incident',?,1,?,?,?,?,'pending',?,?)",
    )
    .bind(event_id)
    .bind(event_id)
    .bind(payload)
    .bind(digest)
    .bind(occurred_at.to_rfc3339())
    .bind(expires_at.to_rfc3339())
    .bind(occurred_at.to_rfc3339())
    .bind(charged_bytes)
    .execute(&database.primary)
    .await
    .unwrap();
}

#[tokio::test]
async fn committed_gap_warning_is_rate_limited_recovers_and_ignores_rollback() {
    let database = TestDatabase::new().await;
    let limiter = test_limiter();
    let now = Utc.with_ymd_and_hms(2026, 9, 7, 9, 0, 0).unwrap();
    sqlx::query(
        "UPDATE notification_delivery_status SET pending_count=100000,pending_bytes=67108864",
    )
    .execute(&database.primary)
    .await
    .unwrap();
    for node in ["rolled-back-node", "committed-node"] {
        sqlx::query(
            "INSERT INTO monitor_nodes
             (node_id,hostname,agent_version,current_boot_id,last_sequence,last_report_at,
              last_received_at,cpu_percent,memory_used_bytes,memory_total_bytes,created_at,updated_at)
             VALUES(?,?,'test','boot',1,?,?,0,1,1,?,?)",
        )
        .bind(node)
        .bind(node)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&database.primary)
        .await
        .unwrap();
    }

    let mut rolled_back = database.primary.begin().await.unwrap();
    crate::notifications::outbox::open(
        &mut rolled_back,
        "rolled-back-node",
        "cpuHigh",
        "cpu",
        "rolled back",
        Some(90.0),
        Some(95.0),
        &now.to_rfc3339(),
    )
    .await
    .unwrap();
    rolled_back.rollback().await.unwrap();
    assert!(!inspect_after_commit_at(&database.peer, &limiter, now).await.unwrap());

    let mut committed = database.primary.begin().await.unwrap();
    crate::notifications::outbox::open(
        &mut committed,
        "committed-node",
        "cpuHigh",
        "cpu",
        "committed",
        Some(90.0),
        Some(95.0),
        &now.to_rfc3339(),
    )
    .await
    .unwrap();
    committed.commit().await.unwrap();
    assert!(inspect_after_commit_at(&database.peer, &limiter, now).await.unwrap());
    sqlx::query("UPDATE notification_delivery_status SET expired_count=1 WHERE id=1")
        .execute(&database.primary)
        .await
        .unwrap();
    assert!(
        !inspect_after_commit_at(&database.peer, &limiter, now + Duration::seconds(59))
            .await
            .unwrap()
    );
    sqlx::query("UPDATE notification_delivery_status SET omitted_count=2 WHERE id=1")
        .execute(&database.primary)
        .await
        .unwrap();
    assert!(
        inspect_after_commit_at(&database.peer, &limiter, now + Duration::seconds(60))
            .await
            .unwrap()
    );
    database.close().await;
}

#[tokio::test]
async fn relay_and_cleanup_commit_each_gap_without_a_followup_incident() {
    let database = TestDatabase::new().await;
    let limiter = test_limiter();
    let base = Utc.with_ymd_and_hms(2026, 9, 7, 10, 0, 0).unwrap();

    insert_outbox(&database, "terminal-expired", base, base + Duration::days(1), 512).await;
    let terminal = claim(&database.primary, base).await.unwrap().unwrap();
    finish(&database.primary, &terminal, DeliveryResult::Expired, base).await.unwrap();
    assert!(inspect_after_commit_at(&database.peer, &limiter, base).await.unwrap());

    let pending_at = base + Duration::minutes(2);
    let pending_expiry = pending_at + Duration::seconds(10);
    insert_outbox(&database, "cleanup-expired", pending_at, pending_expiry, 512).await;
    assert!(
        !run_once_at(&database.primary, &Stored, pending_expiry + Duration::seconds(1))
            .await
            .unwrap()
    );
    assert!(
        inspect_after_commit_at(&database.peer, &limiter, pending_expiry + Duration::seconds(1),)
            .await
            .unwrap()
    );

    let ambiguous_at = base + Duration::minutes(4);
    let ambiguous_expiry = ambiguous_at + Duration::seconds(10);
    insert_outbox(&database, "cleanup-unconfirmed", ambiguous_at, ambiguous_expiry, 512).await;
    let ambiguous = claim(&database.primary, ambiguous_at).await.unwrap().unwrap();
    finish(&database.primary, &ambiguous, DeliveryResult::Timeout, ambiguous_at).await.unwrap();
    let reconcile_expiry = ambiguous_expiry + Duration::seconds(61);
    assert!(!run_once_at(&database.primary, &Stored, reconcile_expiry).await.unwrap());
    assert!(inspect_after_commit_at(&database.peer, &limiter, reconcile_expiry).await.unwrap());

    let quarantine_at = base + Duration::minutes(7);
    insert_outbox(&database, "quarantined", quarantine_at, quarantine_at + Duration::days(1), 512)
        .await;
    let quarantined = claim(&database.primary, quarantine_at).await.unwrap().unwrap();
    finish(&database.primary, &quarantined, DeliveryResult::Conflict, quarantine_at).await.unwrap();
    assert!(inspect_after_commit_at(&database.peer, &limiter, quarantine_at).await.unwrap());

    let eviction_at = base + Duration::minutes(9);
    insert_outbox(
        &database,
        "quarantine-evicted",
        quarantine_at - Duration::seconds(1),
        eviction_at + Duration::days(1),
        4 * 1024 * 1024 + 1,
    )
    .await;
    let evicted = claim(&database.primary, eviction_at).await.unwrap().unwrap();
    finish(&database.primary, &evicted, DeliveryResult::Invalid, eviction_at).await.unwrap();
    assert!(inspect_after_commit_at(&database.peer, &limiter, eviction_at).await.unwrap());

    let status = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
        "SELECT omitted_count,expired_count,unconfirmed_count,quarantined_count,
                quarantine_evicted_count
         FROM notification_delivery_status WHERE id=1",
    )
    .fetch_one(&database.peer)
    .await
    .unwrap();
    assert_eq!(status, (0, 2, 1, 2, 1));
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM notification_outbox WHERE state='quarantined'",
        )
        .fetch_one(&database.peer)
        .await
        .unwrap(),
        1
    );
    database.close().await;
}
