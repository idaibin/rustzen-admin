use super::{close, database, run};
use crate::{features::automation::repo, notifications::outbox};
use rustzen_ipc::NotificationEvent;
use std::sync::Arc;
use tokio::sync::Barrier;

const NOW: &str = "2026-09-07T01:00:00Z";

async fn events(pool: &rustzen_storage::SqlitePool) -> Vec<NotificationEvent> {
    sqlx::query_scalar::<_, String>(
        "SELECT payload_json FROM notification_outbox ORDER BY occurred_at,event_id",
    )
    .fetch_all(pool)
    .await
    .unwrap()
    .into_iter()
    .map(|json| serde_json::from_str(&json).unwrap())
    .collect()
}

#[tokio::test]
async fn manual_terminal_events_bind_initiator_while_scheduled_run_is_silent() {
    let (pool, path) = database().await;
    run(&pool, "manual", "running", Some(41)).await;
    run(&pool, "scheduled", "running", None).await;
    assert!(repo::finish_run(&pool, "manual", "succeeded", None, NOW).await.unwrap());
    assert!(repo::finish_run(&pool, "scheduled", "failed", Some("failed"), NOW).await.unwrap());
    let events = events(&pool).await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].topic, "reports.run.completed");
    assert_eq!(events[0].subject.id, "manual");
    assert_eq!(events[0].subject.revision, 1);
    assert_eq!(events[0].audience.initiator_user_id, Some(41));
    let bytes = serde_json::to_string(&events[0]).unwrap();
    assert!(!bytes.contains("input_json"));
    assert!(!bytes.contains("screenshot"));
    close(pool, path).await;
}

#[tokio::test]
async fn queued_and_running_cancellation_emit_once_on_terminal_transition() {
    let (pool, path) = database().await;
    run(&pool, "queued", "queued", Some(7)).await;
    assert!(repo::cancel_run(&pool, "queued", NOW).await.unwrap());
    assert!(!repo::cancel_run(&pool, "queued", NOW).await.unwrap());

    run(&pool, "running", "running", Some(8)).await;
    assert!(repo::cancel_run(&pool, "running", NOW).await.unwrap());
    assert!(!repo::cancel_run(&pool, "running", NOW).await.unwrap());
    assert_eq!(events(&pool).await.len(), 1);
    assert!(repo::finish_cancelled(&pool, "running", NOW).await.unwrap());
    assert!(!repo::finish_cancelled(&pool, "running", NOW).await.unwrap());
    let mut topics = events(&pool)
        .await
        .into_iter()
        .map(|event| (event.subject.id, event.topic))
        .collect::<Vec<_>>();
    topics.sort();
    assert_eq!(
        topics,
        [
            ("queued".into(), "reports.run.cancelled".into()),
            ("running".into(), "reports.run.cancelled".into())
        ]
    );
    close(pool, path).await;
}

#[tokio::test]
async fn recovery_handles_failed_cancelled_and_null_initiator_once() {
    let (pool, path) = database().await;
    run(&pool, "failed", "running", Some(1)).await;
    run(&pool, "cancelled", "running", Some(2)).await;
    sqlx::query("UPDATE automation_runs SET cancel_requested_at=? WHERE id='cancelled'")
        .bind(NOW)
        .execute(&pool)
        .await
        .unwrap();
    run(&pool, "scheduled", "running", None).await;
    assert_eq!(repo::recover_runs(&pool, NOW).await.unwrap(), 3);
    assert_eq!(repo::recover_runs(&pool, NOW).await.unwrap(), 0);
    let topics = events(&pool).await.into_iter().map(|event| event.topic).collect::<Vec<_>>();
    assert_eq!(topics.len(), 2);
    assert!(topics.contains(&"reports.run.failed".into()));
    assert!(topics.contains(&"reports.run.cancelled".into()));
    close(pool, path).await;
}

#[tokio::test]
async fn completion_and_cancellation_race_converges_to_one_terminal_event() {
    let (pool, path) = database().await;
    run(&pool, "race", "running", Some(9)).await;
    let barrier = Arc::new(Barrier::new(3));
    let finish_pool = pool.clone();
    let finish_barrier = Arc::clone(&barrier);
    let finish = tokio::spawn(async move {
        finish_barrier.wait().await;
        repo::finish_run(&finish_pool, "race", "succeeded", None, NOW).await.unwrap()
    });
    let cancel_pool = pool.clone();
    let cancel_barrier = Arc::clone(&barrier);
    let cancel = tokio::spawn(async move {
        cancel_barrier.wait().await;
        repo::cancel_run(&cancel_pool, "race", NOW).await.unwrap()
    });
    barrier.wait().await;
    let (finished, cancelled) = (finish.await.unwrap(), cancel.await.unwrap());
    assert_ne!(finished, cancelled);
    if cancelled {
        assert!(repo::finish_cancelled(&pool, "race", NOW).await.unwrap());
    }
    assert_eq!(events(&pool).await.len(), 1);
    assert!(matches!(
        repo::run(&pool, "race").await.unwrap().unwrap().status.as_str(),
        "succeeded" | "cancelled"
    ));
    close(pool, path).await;
}

#[tokio::test]
async fn outbox_failure_rolls_back_the_terminal_business_update() {
    let (pool, path) = database().await;
    run(&pool, "rollback", "running", Some(9)).await;
    sqlx::query("DROP TABLE notification_delivery_status").execute(&pool).await.unwrap();
    assert!(outbox::finish(&pool, "rollback", "failed", Some("error"), NOW).await.is_err());
    assert_eq!(repo::run(&pool, "rollback").await.unwrap().unwrap().status, "running");
    close(pool, path).await;
}

#[tokio::test]
async fn full_outbox_commits_terminal_run_and_records_omission_atomically() {
    let (pool, path) = database().await;
    run(&pool, "full", "running", Some(9)).await;
    sqlx::query("UPDATE notification_delivery_status SET pending_count=100000 WHERE id=1")
        .execute(&pool)
        .await
        .unwrap();

    assert!(repo::finish_run(&pool, "full", "failed", Some("error"), NOW).await.unwrap());
    assert_eq!(repo::run(&pool, "full").await.unwrap().unwrap().status, "failed");
    assert_eq!(events(&pool).await.len(), 0);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT omitted_count FROM notification_delivery_status WHERE id=1",
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
    close(pool, path).await;
}

#[tokio::test]
async fn relay_startup_rejects_tampered_durable_accounting() {
    let (pool, path) = database().await;
    crate::notifications::runtime::validate_accounting(&pool).await.unwrap();
    sqlx::query("UPDATE notification_delivery_status SET pending_count=1 WHERE id=1")
        .execute(&pool)
        .await
        .unwrap();
    assert!(crate::notifications::runtime::validate_accounting(&pool).await.is_err());
    close(pool, path).await;
}
