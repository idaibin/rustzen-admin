use super::*;

#[tokio::test]
async fn enqueue_rejects_a_due_slot_before_effective_at_without_side_effects() {
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
    insert_schedule(
        &pool,
        "schedule",
        "flow",
        "daily",
        None,
        "10:00",
        "{}",
        "",
        true,
        "2026-08-10T10:00:30+00:00",
    )
    .await
    .expect("schedule");

    assert_eq!(
        enqueue_schedule_occurrence(
            &pool,
            "schedule",
            "2026-08-10T10:00",
            "2026-08-10T10:00",
            "2026-08-10T10:00:00+00:00",
            "2026-08-10T10:00:30+00:00",
            0,
            "2026-08-10T09:00:00+00:00",
            "2026-08-10T10:00:30+00:00",
        )
        .await
        .expect("stale due decision"),
        ScheduleEnqueueOutcome::ScheduleChanged
    );
    let runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM automation_runs")
        .fetch_one(&pool)
        .await
        .expect("run count");
    let occurrences: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM automation_schedule_occurrences WHERE schedule_id='schedule'",
    )
    .fetch_one(&pool)
    .await
    .expect("occurrence count");
    assert_eq!((runs, occurrences), (0, 0));
}
