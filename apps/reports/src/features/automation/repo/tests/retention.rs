use super::*;

#[tokio::test]
async fn retention_clears_live_run_fk_but_preserves_occurrence_decision_and_snapshot() {
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
    let outcome = enqueue_schedule_occurrence(
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
    let run_id = match outcome {
        ScheduleEnqueueOutcome::Enqueued(run_id) => run_id,
        other => panic!("unexpected outcome: {other:?}"),
    };
    assert_eq!(
        skip_schedule_occurrence(
            &pool,
            "schedule",
            "2026-08-11T10:00",
            "2026-08-11T10:00",
            Some("2026-08-11T10:00:00+00:00"),
            "2026-08-11T10:00:30+00:00",
            "missed",
            0,
            "now",
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("skip latest occurrence"),
        ScheduleSkipOutcome::Skipped
    );
    assert!(last_schedule_run(&pool, "schedule").await.expect("last run").is_none());

    sqlx::query(
        "UPDATE automation_runs
         SET status='succeeded',created_at='2000-01-01T00:00:00+00:00',finished_at='2000-01-01T00:01:00+00:00'
         WHERE id=?",
    )
    .bind(&run_id)
    .execute(&pool)
    .await
    .expect("age run");
    sqlx::query(
        "INSERT INTO automation_artifacts(id,run_id,kind,file_name,created_at)
         VALUES('artifact',?,'screenshot','artifact.png','2000-01-01T00:00:00+00:00')",
    )
    .bind(&run_id)
    .execute(&pool)
    .await
    .expect("artifact");
    let (artifacts, runs) =
        cleanup_retention(&pool, "2020-01-01T00:00:00+00:00", "2020-01-01T00:00:00+00:00")
            .await
            .expect("retention cleanup");
    assert_eq!((artifacts, runs), (1, 1));

    let occurrence: (Option<String>, Option<String>, String) = sqlx::query_as(
        "SELECT run_id,run_id_snapshot,decision FROM automation_schedule_occurrences
         WHERE schedule_id='schedule' AND occurrence_key='2026-08-10T10:00'",
    )
    .fetch_one(&pool)
    .await
    .expect("retained decision");
    assert_eq!(occurrence.0, None);
    assert_eq!(occurrence.1, Some(run_id));
    assert_eq!(occurrence.2, "enqueued");
    let remaining_runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM automation_runs")
        .fetch_one(&pool)
        .await
        .expect("remaining runs");
    let remaining_artifacts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM automation_artifacts")
        .fetch_one(&pool)
        .await
        .expect("remaining artifacts");
    assert_eq!(remaining_runs, 0);
    assert_eq!(remaining_artifacts, 0);
}

#[tokio::test]
async fn retention_deletes_an_old_retry_source_and_keeps_a_newer_child() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect");
    crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
    for statement in [
        "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at) VALUES('system','System','https://example.com',1,'','now','now')",
        "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at) VALUES('flow','system','Flow','[]','now','now')",
        "INSERT INTO automation_runs(id,flow_id,status,input_json,created_at,finished_at) VALUES('source','flow','failed','{}','2000-01-01T00:00:00+00:00','2000-01-01T00:01:00+00:00')",
        "INSERT INTO automation_runs(id,flow_id,retry_source_run_id,status,input_json,created_at,finished_at) VALUES('child','flow','source','succeeded','{}','2026-01-01T00:00:00+00:00','2026-01-01T00:01:00+00:00')",
    ] {
        sqlx::query(statement).execute(&pool).await.expect("fixture");
    }

    let (_, deleted_runs) =
        cleanup_retention(&pool, "2020-01-01T00:00:00+00:00", "2020-01-01T00:00:00+00:00")
            .await
            .expect("retention cleanup");
    assert_eq!(deleted_runs, 1);
    let child_source: Option<String> =
        sqlx::query_scalar("SELECT retry_source_run_id FROM automation_runs WHERE id='child'")
            .fetch_one(&pool)
            .await
            .expect("retained child");
    assert_eq!(child_source, None);
}
