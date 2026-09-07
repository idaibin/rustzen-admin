use super::support::{TestDatabase, create_users, event, grant, policy};
use crate::features::notifications::{
    accounting::Accounting,
    admission::AdmissionService,
    admission_types::{AdmissionError, AdmissionResult, CapacityReason, Failpoint},
};
use chrono::Utc;

async fn accounting(database: &TestDatabase) -> Accounting {
    let mut connection = database.primary.acquire().await.unwrap();
    Accounting::current(&mut connection).await.unwrap()
}

#[test]
fn selected_runtime_config_drives_admission_defaults() {
    let config = rustzen_config::AdminConfig::local().unwrap();
    let configured =
        crate::features::notifications::admission_types::AdmissionPolicy::from_config(&config)
            .unwrap();
    assert_eq!(
        (
            configured.message_limit,
            configured.recipient_limit,
            configured.receipt_limit,
            configured.charged_bytes_limit,
            configured.free_space_reserve_bytes,
            configured.wal_pressure_frames,
            configured.wal_pressure_observations,
        ),
        (100_000, 1_000_000, 1_000_000, 512 * 1024 * 1024, 128 * 1024 * 1024, 1024, 3)
    );
}

#[tokio::test]
async fn admission_is_atomic_across_every_write_failpoint() {
    for stage in [
        Failpoint::AfterReceipt,
        Failpoint::AfterMessage,
        Failpoint::DuringRecipients,
        Failpoint::AfterRecipients,
    ] {
        let database = TestDatabase::new().await;
        grant(&database.primary, "monitor:incident:view").await;
        let service =
            AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
                .await
                .unwrap()
                .with_failpoint(stage);
        let error = service
            .admit(&event("failpoint", Utc::now().naive_utc(), vec![2]), Utc::now().naive_utc())
            .await
            .expect_err("failpoint must abort admission");
        assert!(matches!(error, AdmissionError::Database(_)), "{stage:?}");
        assert_eq!(accounting(&database).await, Accounting::default(), "{stage:?}");
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
        assert_eq!(counts, (0, 0, 0, 0), "{stage:?}");
        database.close().await;
    }
}

#[tokio::test]
async fn duplicate_precedes_saturation_and_digest_conflict() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let original = event("same-id", now, vec![2]);
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    assert!(matches!(
        service.admit(&original, now).await.unwrap(),
        AdmissionResult::Stored { recipients: 1, .. }
    ));
    let used = accounting(&database).await;
    let saturated = AdmissionService::start(
        database.primary.clone(),
        database.path.clone(),
        crate::features::notifications::admission_types::AdmissionPolicy {
            message_limit: used.message_count,
            recipient_limit: used.recipient_count,
            receipt_limit: used.receipt_count,
            charged_bytes_limit: used.charged_bytes,
            ..policy()
        },
    )
    .await
    .unwrap();
    assert_eq!(
        saturated.admit(&original, now).await.unwrap(),
        AdmissionResult::Duplicate { result: "stored".into() }
    );
    let mut changed = original.clone();
    changed.payload_sha256 = "b".repeat(64);
    assert!(matches!(saturated.admit(&changed, now).await, Err(AdmissionError::Conflict)));
    assert_eq!(accounting(&database).await, used);
    database.close().await;
}

#[tokio::test]
async fn each_durable_budget_refuses_without_partial_rows() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let seed = AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
        .await
        .unwrap();
    seed.admit(&event("seed", now, vec![2]), now).await.unwrap();
    let used = accounting(&database).await;
    for (reason, candidate) in [
        (
            CapacityReason::Messages,
            crate::features::notifications::admission_types::AdmissionPolicy {
                message_limit: used.message_count,
                ..policy()
            },
        ),
        (
            CapacityReason::Recipients,
            crate::features::notifications::admission_types::AdmissionPolicy {
                recipient_limit: used.recipient_count,
                ..policy()
            },
        ),
        (
            CapacityReason::Receipts,
            crate::features::notifications::admission_types::AdmissionPolicy {
                receipt_limit: used.receipt_count,
                ..policy()
            },
        ),
        (
            CapacityReason::ChargedBytes,
            crate::features::notifications::admission_types::AdmissionPolicy {
                charged_bytes_limit: used.charged_bytes,
                ..policy()
            },
        ),
    ] {
        let service =
            AdmissionService::start(database.primary.clone(), database.path.clone(), candidate)
                .await
                .unwrap();
        let fresh = event(&format!("budget-{reason:?}"), now, vec![2]);
        assert!(matches!(
            service.admit(&fresh, now).await,
            Err(AdmissionError::Capacity { reason: actual, retry_after_seconds: 30 }) if actual == reason
        ));
        assert_eq!(service.status().reason, Some(reason));
        assert_eq!(accounting(&database).await, used);
    }
    database.close().await;
}

#[tokio::test]
async fn eligible_audience_over_one_thousand_is_rejected_atomically() {
    let database = TestDatabase::new().await;
    create_users(&database.primary, 3, 1002).await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    let candidates = (2..=1002).collect();
    assert!(matches!(
        service.admit(&event("large-audience", now, candidates), now).await,
        Err(AdmissionError::AudienceTooLarge)
    ));
    assert_eq!(accounting(&database).await, Accounting::default());
    database.close().await;
}

#[tokio::test]
async fn restart_validates_persisted_accounting_and_utf8_bounds() {
    let database = TestDatabase::new().await;
    grant(&database.primary, "monitor:incident:view").await;
    let now = Utc::now().naive_utc();
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    service.admit(&event("persisted", now, vec![2]), now).await.unwrap();
    assert_eq!(
        accounting(&database).await,
        Accounting {
            message_count: 1,
            recipient_count: 1,
            receipt_count: 1,
            charged_bytes: 1024 + 512 + 256 + 128,
        }
    );
    drop(service);
    let reopened = database.reopen().await;
    AdmissionService::start(reopened.clone(), database.path.clone(), policy())
        .await
        .expect("durable accounting survives a new pool");

    let mut oversized = event("utf8", now, vec![2]);
    oversized.title = "界".repeat(86);
    let service =
        AdmissionService::start(database.primary.clone(), database.path.clone(), policy())
            .await
            .unwrap();
    assert!(matches!(service.admit(&oversized, now).await, Err(AdmissionError::Invalid)));

    sqlx::query(
        "UPDATE notification_accounting SET charged_bytes = charged_bytes + 1 WHERE id = 1",
    )
    .execute(&database.primary)
    .await
    .unwrap();
    assert!(matches!(
        AdmissionService::start(database.peer.clone(), database.path.clone(), policy()).await,
        Err(AdmissionError::Accounting)
    ));
    reopened.close().await;
    database.close().await;
}
