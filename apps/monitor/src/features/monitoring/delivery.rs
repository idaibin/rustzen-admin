use axum::extract::State;
use serde::Serialize;

use crate::{
    app::AppState,
    common::api::{ApiResponse, AppResult},
};

#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub(super) struct DeliveryStatus {
    pending_count: i64,
    pending_bytes: i64,
    quarantine_count: i64,
    quarantine_bytes: i64,
    omitted_count: i64,
    expired_count: i64,
    unconfirmed_count: i64,
    quarantined_count: i64,
    quarantine_evicted_count: i64,
    first_gap_at: Option<String>,
    last_gap_at: Option<String>,
    last_success_at: Option<String>,
}

pub(super) async fn status(State(state): State<AppState>) -> AppResult<DeliveryStatus> {
    let status = sqlx::query_as::<_, DeliveryStatus>(
        "SELECT pending_count,pending_bytes,quarantine_count,quarantine_bytes,
                omitted_count,expired_count,unconfirmed_count,quarantined_count,
                quarantine_evicted_count,first_gap_at,last_gap_at,last_success_at
         FROM notification_delivery_status WHERE id=1",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(ApiResponse::success(status))
}
