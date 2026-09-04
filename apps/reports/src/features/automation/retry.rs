use chrono::Utc;
use rustzen_storage::SqlitePool;
use uuid::Uuid;

use crate::common::error::AppError;

use super::{repo, repo::RetryRunOutcome, types::Run};

/// Creates a new manual queued run from a terminal source run's persisted snapshot.
/// Schedule occurrences remain attached to the source run only.
pub async fn retry_run(pool: &SqlitePool, source_id: &str) -> Result<Run, AppError> {
    let id = Uuid::new_v4().to_string();
    match repo::retry_run(pool, &id, source_id, &Utc::now().to_rfc3339()).await? {
        RetryRunOutcome::Retry(run) => Ok(run),
        RetryRunOutcome::SourceNotFound => Err(AppError::NotFound("run not found".into())),
        RetryRunOutcome::SourceNotRetryable => {
            Err(AppError::Conflict("only failed or cancelled runs can be retried".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::future::join_all;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::{str::FromStr, time::Duration};
    use uuid::Uuid;

    use super::retry_run;

    #[tokio::test]
    async fn concurrent_retry_requests_share_one_child_and_preserve_source_evidence() {
        let database_path =
            std::env::temp_dir().join(format!("reports-retry-{}.db", Uuid::new_v4()));
        let options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", database_path.display()))
                .expect("sqlite options")
                .create_if_missing(true)
                .foreign_keys(true)
                .busy_timeout(Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .max_connections(16)
            .connect_with(options)
            .await
            .expect("connect");
        crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
        for statement in [
            "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at) VALUES('system','System','https://example.com',1,'','now','now')",
            "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at) VALUES('flow','system','Flow','[]','now','now')",
            "INSERT INTO automation_runs(id,flow_id,status,input_json,error,created_at,finished_at) VALUES('source','flow','failed','{\"value\":\"saved\"}','failure','now','now')",
            "INSERT INTO automation_run_steps(run_id,step_index,action,status,duration_ms,message,created_at) VALUES('source',0,'goto','failed',1,'failure','now')",
            "INSERT INTO automation_artifacts(id,run_id,kind,file_name,created_at) VALUES('artifact','source','screenshot','failure.png','now')",
            "INSERT INTO automation_schedules(id,flow_id,cadence,weekday,due_time,input_json,description,enabled,effective_at,revision,created_at,updated_at) VALUES('schedule','flow','daily',NULL,'10:00','{}','',1,'now',0,'now','now')",
            "INSERT INTO automation_schedule_occurrences(schedule_id,occurrence_key,due_local,due_at,decided_at,decision,reason,run_id,run_id_snapshot) VALUES('schedule','slot','2026-01-01T10:00:00','2026-01-01T10:00:00Z','now','enqueued',NULL,'source','source')",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("fixture");
        }

        let retried = join_all((0..16).map(|_| {
            let pool = pool.clone();
            tokio::spawn(
                async move { retry_run(&pool, "source").await.expect("retry terminal run") },
            )
        }))
        .await
        .into_iter()
        .map(|result| result.expect("retry task"))
        .collect::<Vec<_>>();
        let child_id = retried[0].id.clone();
        assert!(retried.iter().all(|run| run.id == child_id));
        assert!(retried.iter().all(|run| run.status == "queued"));
        let retried = &retried[0];
        assert_eq!(retried.status, "queued");
        assert_eq!(retried.flow_id, "flow");
        let source = sqlx::query_as::<_, (String, String, String, String)>(
            "SELECT flow_id,status,input_json,error FROM automation_runs WHERE id='source'",
        )
        .fetch_one(&pool)
        .await
        .expect("source");
        assert_eq!(
            source,
            ("flow".into(), "failed".into(), "{\"value\":\"saved\"}".into(), "failure".into())
        );
        let retried_input: String =
            sqlx::query_scalar("SELECT input_json FROM automation_runs WHERE id=?")
                .bind(&retried.id)
                .fetch_one(&pool)
                .await
                .expect("retried input");
        assert_eq!(retried_input, "{\"value\":\"saved\"}");
        let occurrence_run: String = sqlx::query_scalar(
            "SELECT run_id FROM automation_schedule_occurrences WHERE schedule_id='schedule'",
        )
        .fetch_one(&pool)
        .await
        .expect("occurrence run");
        assert_eq!(occurrence_run, "source");
        let retry_occurrences: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM automation_schedule_occurrences WHERE run_id=?",
        )
        .bind(&retried.id)
        .fetch_one(&pool)
        .await
        .expect("retry occurrences");
        assert_eq!(retry_occurrences, 0);
        let source_steps: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM automation_run_steps WHERE run_id='source'")
                .fetch_one(&pool)
                .await
                .expect("source steps");
        let source_artifacts: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM automation_artifacts WHERE run_id='source'")
                .fetch_one(&pool)
                .await
                .expect("source artifacts");
        assert_eq!((source_steps, source_artifacts), (1, 1));
        let child_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM automation_runs WHERE retry_source_run_id='source'",
        )
        .fetch_one(&pool)
        .await
        .expect("retry child count");
        assert_eq!(child_count, 1);

        sqlx::query("UPDATE automation_runs SET status='failed',finished_at='later' WHERE id=?")
            .bind(&child_id)
            .execute(&pool)
            .await
            .expect("finish child");
        let original_retry = retry_run(&pool, "source").await.expect("retry source again");
        assert_eq!(original_retry.id, child_id);
        assert_eq!(original_retry.status, "failed");
        let grandchild = retry_run(&pool, &child_id).await.expect("retry terminal child");
        assert_ne!(grandchild.id, child_id);
        let grandchild_source: String =
            sqlx::query_scalar("SELECT retry_source_run_id FROM automation_runs WHERE id=?")
                .bind(&grandchild.id)
                .fetch_one(&pool)
                .await
                .expect("grandchild source");
        assert_eq!(grandchild_source, child_id);
        pool.close().await;
        tokio::fs::remove_file(database_path).await.expect("remove temporary database");
    }

    #[tokio::test]
    async fn retry_rejects_non_terminal_runs() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
        for statement in [
            "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at) VALUES('system','System','https://example.com',1,'','now','now')",
            "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at) VALUES('flow','system','Flow','[]','now','now')",
            "INSERT INTO automation_runs(id,flow_id,status,input_json,created_at) VALUES('queued','flow','queued','{}','now')",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("fixture");
        }
        assert!(retry_run(&pool, "queued").await.is_err());
    }
}
