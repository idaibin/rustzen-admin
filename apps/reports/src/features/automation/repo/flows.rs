use super::*;

pub async fn flows(
    pool: &SqlitePool,
    system_id: Option<&str>,
) -> Result<Vec<FlowRow>, sqlx::Error> {
    sqlx::query_as("SELECT id,system_id,name,steps_json,created_at,updated_at FROM automation_flows WHERE (? IS NULL OR system_id=?) ORDER BY created_at DESC").bind(system_id).bind(system_id).fetch_all(pool).await
}

pub async fn flow_options(pool: &SqlitePool) -> Result<Vec<FlowOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT flows.id, flows.name, systems.enabled
         FROM automation_flows AS flows
         INNER JOIN automation_systems AS systems ON systems.id = flows.system_id
         ORDER BY flows.created_at DESC, flows.id DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn flow(pool: &SqlitePool, id: &str) -> Result<Option<FlowRow>, sqlx::Error> {
    sqlx::query_as("SELECT id,system_id,name,steps_json,created_at,updated_at FROM automation_flows WHERE id=?").bind(id).fetch_optional(pool).await
}
pub async fn insert_flow(
    pool: &SqlitePool,
    id: &str,
    system_id: &str,
    name: &str,
    steps: &str,
    now: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO automation_flows(id,system_id,name,steps_json,created_at,updated_at) VALUES(?,?,?,?,?,?)").bind(id).bind(system_id).bind(name).bind(steps).bind(now).bind(now).execute(pool).await?;
    Ok(())
}
pub async fn update_flow(
    pool: &SqlitePool,
    id: &str,
    system_id: &str,
    name: &str,
    steps: &str,
    now: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query(
        "UPDATE automation_flows SET system_id=?,name=?,steps_json=?,updated_at=? WHERE id=?",
    )
    .bind(system_id)
    .bind(name)
    .bind(steps)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await
    .map(|r| r.rows_affected() == 1)
}
pub async fn delete_flow(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    sqlx::query("DELETE FROM automation_flows WHERE id=?")
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() == 1)
}
