//! Production single-poll integration with fresh SQLite; no worker is spawned.
use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::{calendar::ScheduleZone, initialize, process_schedules_once};
use crate::{
    app::AppState,
    features::automation::{repo, service},
};

struct Fixture {
    state: AppState,
    root: std::path::PathBuf,
    options: SqliteConnectOptions,
    due: DateTime<Utc>,
}

impl Fixture {
    async fn new() -> Self {
        let root = std::env::temp_dir().join(format!("reports-poll-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let options = SqliteConnectOptions::new()
            .filename(root.join("test.db"))
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options.clone())
            .await
            .unwrap();
        crate::infra::db::run_migrations(&pool).await.unwrap();
        sqlx::query("INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at) VALUES('system','Owned fixture','http://127.0.0.1:9',0,'','now','now')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at) VALUES('flow','system','Fixture','[]','now','now')")
            .execute(&pool).await.unwrap();
        let local =
            chrono::NaiveDate::from_ymd_opt(2026, 8, 10).unwrap().and_hms_opt(10, 0, 0).unwrap();
        let zone = super::parse_schedule_timezone(crate::config::CONFIG.timezone()).unwrap();
        let due = match zone {
            ScheduleZone::Iana(zone) => {
                zone.from_local_datetime(&local).single().unwrap().with_timezone(&Utc)
            }
            ScheduleZone::Fixed(zone) => {
                zone.from_local_datetime(&local).single().unwrap().with_timezone(&Utc)
            }
        };
        let state = AppState {
            pool,
            output_dir: root.join("unused-output"),
            browser_path: Some("/nonexistent-test-browser".into()),
            headless: true,
            max_concurrency: 1,
        };
        Self { state, root, options, due }
    }

    async fn schedule(&self, id: &str, cadence: &str, enabled: bool, effective: DateTime<Utc>) {
        repo::insert_schedule(
            &self.state.pool,
            id,
            "flow",
            cadence,
            (cadence == "weekly").then_some(0),
            "10:00",
            r#"{"fixture":"original"}"#,
            "",
            enabled,
            &effective.to_rfc3339(),
        )
        .await
        .unwrap();
    }

    async fn reopen(&mut self) {
        self.state.pool.close().await;
        self.state.pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(self.options.clone())
            .await
            .unwrap();
        initialize(&self.state).await.unwrap();
    }

    async fn counts(&self, occurrences: i64, runs: i64) {
        for (table, expected) in [
            ("SELECT COUNT(*) FROM automation_schedule_occurrences", occurrences),
            ("SELECT COUNT(*) FROM automation_runs", runs),
            ("SELECT COUNT(*) FROM automation_artifacts", 0),
        ] {
            let count: i64 = sqlx::query_scalar(table).fetch_one(&self.state.pool).await.unwrap();
            assert_eq!(count, expected, "{table}");
        }
        #[cfg(feature = "notifications")]
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM notification_outbox")
                .fetch_one(&self.state.pool)
                .await
                .unwrap(),
            0
        );
        assert!(!self.state.output_dir.exists());
    }

    async fn close(self) {
        assert_eq!(
            sqlx::query_scalar::<_, String>("PRAGMA quick_check")
                .fetch_one(&self.state.pool)
                .await
                .unwrap(),
            "ok"
        );
        self.state.pool.close().await;
        std::fs::remove_dir_all(self.root).unwrap();
    }
}

#[tokio::test]
async fn due_daily_and_weekly_survive_reopen_without_duplicate_and_keep_cancelled_link() {
    let mut fixture = Fixture::new().await;
    for (id, cadence) in [("daily", "daily"), ("weekly", "weekly")] {
        fixture.schedule(id, cadence, true, fixture.due - TimeDelta::hours(1)).await;
    }
    process_schedules_once(&fixture.state, fixture.due - TimeDelta::seconds(1)).await.unwrap();
    fixture.counts(0, 0).await;
    process_schedules_once(&fixture.state, fixture.due + TimeDelta::seconds(60)).await.unwrap();
    fixture.counts(2, 2).await;
    let daily = service::schedule(&fixture.state.pool, "daily").await.unwrap();
    let occurrence = daily.last_occurrence.unwrap();
    assert_eq!(occurrence.decision, "enqueued");
    assert_eq!(occurrence.due_local, "2026-08-10T10:00");
    assert_eq!(occurrence.due_at, Some(fixture.due.to_rfc3339()));
    let run_id = occurrence.run_id.unwrap();
    assert_eq!(daily.last_run.unwrap().id, run_id);
    assert_eq!(service::run(&fixture.state.pool, &run_id).await.unwrap().status, "queued");
    let initiators: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM automation_runs WHERE initiator_user_id IS NOT NULL",
    )
    .fetch_one(&fixture.state.pool)
    .await
    .unwrap();
    assert_eq!(initiators, 0);
    assert_eq!(
        service::cancel_run(&fixture.state.pool, &run_id).await.unwrap().status,
        "cancelled"
    );
    assert!(service::cancel_run(&fixture.state.pool, &run_id).await.is_err());
    fixture.reopen().await;
    process_schedules_once(&fixture.state, fixture.due + TimeDelta::seconds(60)).await.unwrap();
    fixture.counts(2, 2).await;
    process_schedules_once(&fixture.state, fixture.due + TimeDelta::seconds(61)).await.unwrap();
    fixture.counts(2, 2).await;
    let daily = service::schedule(&fixture.state.pool, "daily").await.unwrap();
    assert_eq!(daily.last_occurrence.unwrap().run_id, Some(run_id.clone()));
    assert_eq!(daily.last_run.unwrap().status, "cancelled");
    assert_eq!(
        service::run(&fixture.state.pool, &run_id).await.unwrap().input_json,
        r#"{"fixture":"original"}"#
    );
    fixture.close().await;
}

#[tokio::test]
async fn missed_poll_is_persisted_once_without_run_and_disabled_or_new_slots_are_ignored() {
    let mut fixture = Fixture::new().await;
    fixture.schedule("missed", "daily", true, fixture.due - TimeDelta::hours(1)).await;
    fixture.schedule("disabled", "daily", false, fixture.due - TimeDelta::hours(1)).await;
    fixture.schedule("new", "daily", true, fixture.due + TimeDelta::seconds(1)).await;
    process_schedules_once(&fixture.state, fixture.due + TimeDelta::seconds(61)).await.unwrap();
    fixture.counts(1, 0).await;
    fixture.reopen().await;
    process_schedules_once(&fixture.state, fixture.due + TimeDelta::minutes(5)).await.unwrap();
    fixture.counts(1, 0).await;
    let missed = service::schedule(&fixture.state.pool, "missed").await.unwrap();
    let occurrence = missed.last_occurrence.unwrap();
    assert_eq!(occurrence.decision, "skipped");
    assert_eq!(occurrence.reason.as_deref(), Some("missed"));
    assert_eq!(occurrence.due_at, Some(fixture.due.to_rfc3339()));
    assert!(occurrence.run_id.is_none());
    assert!(missed.last_run.is_none());
    for id in ["disabled", "new"] {
        assert!(
            service::schedule(&fixture.state.pool, id).await.unwrap().last_occurrence.is_none()
        );
    }
    fixture.close().await;
}
