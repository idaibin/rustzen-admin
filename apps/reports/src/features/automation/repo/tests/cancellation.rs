use super::*;

#[tokio::test]
async fn cancelling_a_running_run_keeps_it_non_terminal_until_execution_stops() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect");
    crate::infra::db::MIGRATOR.run(&pool).await.expect("migrate");
    sqlx::query(
        "INSERT INTO automation_systems(id,name,base_url,enabled,notes,created_at,updated_at) VALUES('system','System','https://example.com',1,'','now','now')",
    )
    .execute(&pool)
    .await
    .expect("system");
    sqlx::query(
        "INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at) VALUES('flow','system','Flow','[]','now','now')",
    )
    .execute(&pool)
    .await
    .expect("flow");
    sqlx::query(
        "INSERT INTO automation_runs(id,flow_id,status,input_json,created_at,started_at) VALUES('run','flow','running','{}','now','now')",
    )
    .execute(&pool)
    .await
    .expect("run");

    assert!(cancel_run(&pool, "run", "later").await.expect("request cancellation"));
    let status: String = sqlx::query_scalar("SELECT status FROM automation_runs WHERE id='run'")
        .fetch_one(&pool)
        .await
        .expect("status");
    assert_eq!(status, "running");
    assert_eq!(run(&pool, "run").await.expect("load run").expect("run").status, "cancelling");
    assert!(
        !finish_run(&pool, "run", "succeeded", None, "finished")
            .await
            .expect("reject normal finish")
    );

    finish_cancelled(&pool, "run", "finished").await.expect("finish cancellation");
    let status: String = sqlx::query_scalar("SELECT status FROM automation_runs WHERE id='run'")
        .fetch_one(&pool)
        .await
        .expect("status");
    assert_eq!(status, "cancelled");
}
