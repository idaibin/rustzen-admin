use super::*;

#[tokio::test]
async fn skip_rechecks_enabled_revision_updated_at_and_delete_before_writing() {
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
    for id in ["disabled", "updated", "deleted", "ok"] {
        insert_schedule(
            &pool,
            id,
            "flow",
            "daily",
            None,
            "10:00",
            "{}",
            "",
            true,
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("schedule");
    }
    sqlx::query(
        "UPDATE automation_schedules
         SET enabled=0,revision=revision+1,effective_at='2026-08-10T10:01:00+00:00',updated_at='disabled-v2'
         WHERE id='disabled'",
    )
    .execute(&pool)
    .await
    .expect("disable");
    sqlx::query(
        "UPDATE automation_schedules
         SET input_json='{\"value\":\"new\"}',revision=revision+1,effective_at='2026-08-10T10:01:00+00:00',updated_at='updated-v2'
         WHERE id='updated'",
    )
    .execute(&pool)
    .await
    .expect("update");
    sqlx::query("DELETE FROM automation_schedules WHERE id='deleted'")
        .execute(&pool)
        .await
        .expect("delete");

    for id in ["disabled", "updated", "deleted"] {
        let outcome = skip_schedule_occurrence(
            &pool,
            id,
            "2026-08-10T10:00",
            "2026-08-10T10:00",
            Some("2026-08-10T10:00:00+00:00"),
            "2026-08-10T10:00:30+00:00",
            "missed",
            0,
            "2026-08-10T09:00:00+00:00",
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("stale skip outcome");
        assert_eq!(outcome, ScheduleSkipOutcome::ScheduleChanged, "{id}");
    }
    assert_eq!(
        skip_schedule_occurrence(
            &pool,
            "ok",
            "2026-08-10T10:00",
            "2026-08-10T10:00",
            Some("2026-08-10T10:00:00+00:00"),
            "2026-08-10T10:00:30+00:00",
            "missed",
            0,
            "2026-08-10T09:00:00+00:00",
            "2026-08-10T09:00:00+00:00",
        )
        .await
        .expect("valid skip"),
        ScheduleSkipOutcome::Skipped
    );
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM automation_schedule_occurrences WHERE decision='skipped'",
    )
    .fetch_one(&pool)
    .await
    .expect("skip count");
    assert_eq!(count, 1);
}
