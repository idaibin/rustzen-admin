use super::*;

#[tokio::test]
async fn schedule_snapshot_changes_never_create_runs_from_stale_input() {
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
        "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at)
         VALUES('flow-current','system','Current Flow','[]','now','now')",
    )
    .execute(&pool)
    .await
    .expect("current flow");
    for id in ["disabled", "updated", "deleted", "current"] {
        sqlx::query(
            "INSERT INTO automation_schedules
             (id,flow_id,cadence,weekday,due_time,input_json,description,enabled,effective_at,created_at,updated_at)
             VALUES(?, 'flow', 'daily', NULL, '10:00', '{}', '', 1, '2026-08-10T09:00:00+00:00', 'now', 'now')",
        )
        .bind(id)
        .execute(&pool)
        .await
        .expect("schedule");
    }
    sqlx::query(
        "UPDATE automation_schedules SET enabled=0,revision=revision+1,updated_at='disabled-v2'
         WHERE id='disabled'",
    )
    .execute(&pool)
    .await
    .expect("disable schedule");
    assert_eq!(
        enqueue_schedule_occurrence(
            &pool,
            "disabled",
            "2026-08-10T10:00",
            "2026-08-10T10:00",
            "2026-08-10T10:00:00+00:00",
            "2026-08-10T10:00:30+00:00",
            0,
            "now",
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("disabled snapshot"),
        ScheduleEnqueueOutcome::ScheduleChanged
    );

    sqlx::query(
        "UPDATE automation_schedules SET input_json='{\"value\":\"new\"}',revision=revision+1,updated_at='updated-v2'
         WHERE id='updated'",
    )
    .execute(&pool)
    .await
    .expect("update schedule");
    assert_eq!(
        enqueue_schedule_occurrence(
            &pool,
            "updated",
            "2026-08-10T10:00",
            "2026-08-10T10:00",
            "2026-08-10T10:00:00+00:00",
            "2026-08-10T10:00:30+00:00",
            0,
            "now",
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("updated snapshot"),
        ScheduleEnqueueOutcome::ScheduleChanged
    );

    sqlx::query("DELETE FROM automation_schedules WHERE id='deleted'")
        .execute(&pool)
        .await
        .expect("delete schedule");
    assert_eq!(
        enqueue_schedule_occurrence(
            &pool,
            "deleted",
            "2026-08-10T10:00",
            "2026-08-10T10:00",
            "2026-08-10T10:00:00+00:00",
            "2026-08-10T10:00:30+00:00",
            0,
            "now",
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("deleted snapshot"),
        ScheduleEnqueueOutcome::ScheduleChanged
    );

    sqlx::query(
        "UPDATE automation_schedules SET flow_id='flow-current',input_json='{\"value\":\"current\"}',revision=revision+1,updated_at='current-v2'
         WHERE id='current'",
    )
    .execute(&pool)
    .await
    .expect("current schedule");
    let current_run = match enqueue_schedule_occurrence(
        &pool,
        "current",
        "2026-08-10T10:00",
        "2026-08-10T10:00",
        "2026-08-10T10:00:00+00:00",
        "2026-08-10T10:00:30+00:00",
        1,
        "current-v2",
        "2026-08-10T09:00:00+00:00",
    )
    .await
    .expect("current snapshot")
    {
        ScheduleEnqueueOutcome::Enqueued(run_id) => run_id,
        other => panic!("unexpected current outcome: {other:?}"),
    };
    let current_run: (String, String) =
        sqlx::query_as("SELECT flow_id,input_json FROM automation_runs WHERE id=?")
            .bind(current_run)
            .fetch_one(&pool)
            .await
            .expect("current run snapshot");
    assert_eq!(current_run.0, "flow-current");
    assert_eq!(current_run.1, r#"{"value":"current"}"#);

    let runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM automation_runs")
        .fetch_one(&pool)
        .await
        .expect("run count");
    assert_eq!(runs, 1);
}
