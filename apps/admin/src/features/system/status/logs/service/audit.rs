use super::*;

pub(super) fn backup_selection(request: &ModuleLogBackupRequest) -> serde_json::Value {
    json!({
        "files": request.files.iter().map(|file| json!({
            "module": file.module,
            "date": file.date
        })).collect::<Vec<_>>()
    })
}

pub(super) fn cleanup_selection(preview: &CleanupPreviewState) -> serde_json::Value {
    json!({
        "previewId": preview.preview_id,
        "cutoffDate": preview.cutoff_date.to_string(),
        "files": preview.candidates.iter().map(|state| &state.candidate).collect::<Vec<_>>()
    })
}

pub(super) fn operation_data(
    phase: &str,
    selection: serde_json::Value,
    result_counts: serde_json::Value,
    failure_category: Option<&str>,
) -> serde_json::Value {
    json!({
        "phase": phase,
        "selection": selection,
        "resultCounts": result_counts,
        "failureCategory": failure_category
    })
}

pub(super) async fn audit_with_status(
    pool: &SqlitePool,
    user: &CurrentUser,
    action: &'static str,
    description: String,
    status: &'static str,
    data: serde_json::Value,
) -> Result<i64, ServiceError> {
    let command = LogWriteCommand {
        user_id: user.user_id,
        username: user.username.clone(),
        action: action.to_owned(),
        description,
        data: Some(data),
        status: status.into(),
        duration_ms: 0,
        ip_address: "module-log".into(),
        user_agent: "Admin module-log diagnostics".into(),
    };
    LogService::record_operation(pool, command).await.map_err(|error| {
        tracing::error!(%error, action, status, "Failed to audit module log operation");
        error
    })
}

pub(super) async fn audit_failure(
    pool: &SqlitePool,
    user: &CurrentUser,
    action: &'static str,
    description: String,
    data: serde_json::Value,
) {
    if let Err(error) = audit_with_status(pool, user, action, description, "FAILED", data).await {
        tracing::error!(%error, action, "Failed to persist module log failure audit");
    }
}

pub(super) fn io_error(action: &'static str) -> impl FnOnce(io::Error) -> ServiceError {
    move |error| {
        tracing::warn!(%error, action, "Module log filesystem operation failed");
        ServiceError::InvalidOperation(format!("Module log operation failed: {action}"))
    }
}

pub(super) fn join_error(error: task::JoinError) -> ServiceError {
    ServiceError::InvalidOperation(format!("Module log operation failed: {error}"))
}
