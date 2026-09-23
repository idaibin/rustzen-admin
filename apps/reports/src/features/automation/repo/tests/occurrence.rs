use super::*;

#[tokio::test]
async fn schedule_occurrence_decision_and_run_are_atomic_and_idempotent() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect");
    crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
    sqlx::query(
        "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at)
         VALUES('system','System','https://example.com',1,'','now','now')",
    )
    .execute(&pool)
    .await
    .expect("system");
    sqlx::query(
        "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at)
         VALUES('flow','system','Flow','[]','now','now')",
    )
    .execute(&pool)
    .await
    .expect("flow");
    sqlx::query(
        "INSERT INTO automation_schedules
         (id,flow_id,cadence,weekday,due_time,input_json,description,enabled,effective_at,created_at,updated_at)
         VALUES('schedule','flow','daily',NULL,'10:00','{}','',1,'2026-08-10T09:00:00+00:00','now','now')",
    )
    .execute(&pool)
    .await
    .expect("schedule");

    let first = enqueue_schedule_occurrence(
        &pool,
        "schedule",
        "2026-08-10T10:00",
        "2026-08-10T10:00",
        "2026-08-10T10:00:00+00:00",
        "2026-08-10T10:00:30+00:00",
        0,
        "now",
        "2026-08-10T09:00:00+00:00",
    )
    .await
    .expect("enqueue");
    assert!(matches!(first, ScheduleEnqueueOutcome::Enqueued(_)));
    let second = enqueue_schedule_occurrence(
        &pool,
        "schedule",
        "2026-08-10T10:00",
        "2026-08-10T10:00",
        "2026-08-10T10:00:00+00:00",
        "2026-08-10T10:00:45+00:00",
        0,
        "now",
        "2026-08-10T09:00:00+00:00",
    )
    .await
    .expect("idempotent enqueue");
    assert_eq!(second, ScheduleEnqueueOutcome::AlreadyDecided);

    let occurrences: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM automation_schedule_occurrences WHERE schedule_id='schedule'",
    )
    .fetch_one(&pool)
    .await
    .expect("occurrence count");
    let runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM automation_runs")
        .fetch_one(&pool)
        .await
        .expect("run count");
    assert_eq!(occurrences, 1);
    assert_eq!(runs, 1);
    let initiator: Option<i64> =
        sqlx::query_scalar("SELECT initiator_user_id FROM automation_runs LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("scheduled run initiator");
    assert_eq!(initiator, None);
    assert_eq!(
        skip_schedule_occurrence(
            &pool,
            "schedule",
            "2026-08-10T10:00",
            "2026-08-10T10:00",
            None,
            "2026-08-10T10:01:00+00:00",
            "missed",
            0,
            "now",
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("idempotent skip"),
        ScheduleSkipOutcome::AlreadyDecided
    );
}
