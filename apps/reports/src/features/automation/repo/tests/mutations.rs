use super::*;

#[tokio::test]
async fn schedule_mutations_refresh_effective_at_and_revision() {
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
        "2026-08-10T09:00:00+00:00",
    )
    .await
    .expect("schedule");

    let initial =
        schedule(&pool, "schedule").await.expect("load initial").expect("initial schedule");
    assert_eq!(initial.revision, 0);
    assert_eq!(initial.effective_at, "2026-08-10T09:00:00+00:00");

    assert!(
        update_schedule(
            &pool,
            "schedule",
            "flow",
            "daily",
            None,
            "10:00",
            "{}",
            "",
            false,
            "2026-08-10T10:01:00+00:00",
        )
        .await
        .expect("disable")
    );
    let disabled =
        schedule(&pool, "schedule").await.expect("load disabled").expect("disabled schedule");
    assert!(!disabled.enabled);
    assert_eq!(disabled.revision, 1);
    assert_eq!(disabled.effective_at, "2026-08-10T10:01:00+00:00");

    assert!(
        update_schedule(
            &pool,
            "schedule",
            "flow",
            "weekly",
            Some(1),
            "11:30",
            r#"{"value":"new"}"#,
            "updated",
            true,
            "2026-08-10T10:02:00+00:00",
        )
        .await
        .expect("re-enable and update")
    );
    let reenabled =
        schedule(&pool, "schedule").await.expect("load reenabled").expect("reenabled schedule");
    assert!(reenabled.enabled);
    assert_eq!(reenabled.cadence, "weekly");
    assert_eq!(reenabled.weekday, Some(1));
    assert_eq!(reenabled.due_time, "11:30");
    assert_eq!(reenabled.input_json, r#"{"value":"new"}"#);
    assert_eq!(reenabled.revision, 2);
    assert_eq!(reenabled.effective_at, "2026-08-10T10:02:00+00:00");
}
