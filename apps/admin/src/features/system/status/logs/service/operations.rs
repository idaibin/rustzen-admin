use super::*;

impl ModuleLogService {
    pub async fn list(
        _pool: &SqlitePool,
        _user: &CurrentUser,
        query: ModuleLogListQuery,
    ) -> Result<Vec<ModuleLogFileResp>, ServiceError> {
        let module = query.module.map(|value| parse_module(&value)).transpose()?;
        let date = query.date.map(|value| parse_date(&value)).transpose()?;
        let log_dir = CONFIG.runtime.log_dir();
        task::spawn_blocking(move || list_in(&log_dir, module, date)).await.map_err(join_error)?
    }

    pub async fn tail(
        _pool: &SqlitePool,
        _user: &CurrentUser,
        query: ModuleLogTailQuery,
    ) -> Result<ModuleLogTailResp, ServiceError> {
        let selector =
            parse_selector(&ModuleLogFileSelector { module: query.module, date: query.date })?;
        let log_dir = CONFIG.runtime.log_dir();
        task::spawn_blocking(move || tail_in(&log_dir, selector, query.cursor.as_deref()))
            .await
            .map_err(join_error)?
    }

    pub async fn backup(
        pool: &SqlitePool,
        user: &CurrentUser,
        request: ModuleLogBackupRequest,
    ) -> Result<BackupArchive, ServiceError> {
        let selection = backup_selection(&request);
        let log_dir = CONFIG.runtime.log_dir();
        let archive_result =
            match task::spawn_blocking(move || build_archive(&log_dir, request)).await {
                Ok(result) => result,
                Err(error) => Err(join_error(error)),
            };
        match archive_result {
            Ok(archive) => {
                let file_count = archive.file_count;
                let digest = archive.archive_sha256.clone();
                let mut data =
                    operation_data("success", selection, json!({ "fileCount": file_count }), None);
                if let Some(object) = data.as_object_mut() {
                    object.insert("archiveSha256".into(), json!(digest));
                }
                audit_with_status(
                    pool,
                    user,
                    "MODULE_LOG_BACKUP",
                    "Backed up selected module log files".into(),
                    "SUCCESS",
                    data,
                )
                .await
                .map_err(|_| {
                    ServiceError::InvalidOperation(
                        "Module log backup completed but its audit record could not be persisted"
                            .into(),
                    )
                })?;
                tracing::debug!(archive_sha256 = %digest, file_count, "Module log archive audit persisted");
                Ok(archive)
            }
            Err(error) => {
                let data = operation_data(
                    "failure",
                    selection,
                    json!({ "fileCount": 0 }),
                    Some("backup_failed"),
                );
                audit_failure(
                    pool,
                    user,
                    "MODULE_LOG_BACKUP",
                    "Module log backup failed".into(),
                    data,
                )
                .await;
                Err(error)
            }
        }
    }

    pub async fn preview_cleanup(
        pool: &SqlitePool,
        user: &CurrentUser,
    ) -> Result<ModuleLogCleanupPreviewResp, ServiceError> {
        let today = Utc::now().date_naive();
        let cutoff_date = today - Days::new(RETENTION_DAYS);
        let log_dir = CONFIG.runtime.log_dir();
        let collection = match task::spawn_blocking(move || {
            collect_cleanup_candidates(&log_dir, today, cutoff_date)
        })
        .await
        {
            Ok(result) => result,
            Err(error) => Err(join_error(error)),
        };
        let (candidates, failures) = match collection {
            Ok(result) => result,
            Err(error) => {
                audit_failure(
                    pool,
                    user,
                    "MODULE_LOG_CLEANUP_PREVIEW",
                    "Module log cleanup preview failed".into(),
                    operation_data(
                        "failure",
                        json!({ "cutoffDate": cutoff_date.to_string() }),
                        json!({ "candidateCount": 0, "failureCount": 1 }),
                        Some("preview_failed"),
                    ),
                )
                .await;
                return Err(error);
            }
        };

        let preview_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let expires_at =
            Utc::now() + chrono::Duration::seconds(CLEANUP_PREVIEW_TTL.as_secs() as i64);
        let state = CleanupPreviewState {
            preview_id: preview_id.clone(),
            expires_at_instant: Instant::now() + CLEANUP_PREVIEW_TTL,
            cutoff_date,
            candidates: candidates.clone(),
        };
        if let Err(error) = store_preview(token.clone(), state) {
            audit_failure(
                pool,
                user,
                "MODULE_LOG_CLEANUP_PREVIEW",
                "Module log cleanup preview could not be stored".into(),
                operation_data(
                    "failure",
                    json!({ "previewId": preview_id, "cutoffDate": cutoff_date.to_string() }),
                    json!({ "candidateCount": candidates.len(), "failureCount": failures.len() }),
                    Some("preview_state_unavailable"),
                ),
            )
            .await;
            return Err(error);
        }

        let status = if failures.is_empty() { "SUCCESS" } else { "PARTIAL" };
        if audit_with_status(
            pool,
            user,
            "MODULE_LOG_CLEANUP_PREVIEW",
            "Previewed module log cleanup candidates".into(),
            status,
            operation_data(
                "success",
                json!({
                    "previewId": preview_id,
                    "cutoffDate": cutoff_date.to_string(),
                    "files": candidates.iter().map(|state| &state.candidate).collect::<Vec<_>>()
                }),
                json!({
                    "candidateCount": candidates.len(),
                    "failureCount": failures.len()
                }),
                (!failures.is_empty()).then_some("candidate_unavailable"),
            ),
        )
        .await
        .is_err()
        {
            remove_preview(&token);
            return Err(ServiceError::InvalidOperation(
                "Module log cleanup preview could not be audited".into(),
            ));
        }

        Ok(ModuleLogCleanupPreviewResp {
            preview_id,
            token,
            expires_at,
            cutoff_date: cutoff_date.to_string(),
            candidates: candidates.into_iter().map(|state| state.candidate).collect(),
            failures,
        })
    }

    pub async fn confirm_cleanup(
        pool: &SqlitePool,
        user: &CurrentUser,
        request: ModuleLogCleanupConfirmRequest,
    ) -> Result<ModuleLogCleanupResultResp, ServiceError> {
        let preview = match take_preview(&request.token) {
            Ok(preview) => preview,
            Err(error) => {
                audit_failure(
                    pool,
                    user,
                    "MODULE_LOG_CLEANUP_CONFIRM",
                    "Module log cleanup confirmation was rejected".into(),
                    operation_data(
                        "failure",
                        json!({ "previewId": serde_json::Value::Null }),
                        json!({ "removedCount": 0, "retainedCount": 0, "failureCount": 1 }),
                        Some("invalid_confirmation"),
                    ),
                )
                .await;
                return Err(error);
            }
        };
        let intent_data = operation_data(
            "intent",
            cleanup_selection(&preview),
            json!({ "removedCount": 0, "retainedCount": 0, "failureCount": 0 }),
            None,
        );
        let intent_id = audit_with_status(
            pool,
            user,
            "MODULE_LOG_CLEANUP_CONFIRM",
            "Persisted module log cleanup intent".into(),
            "PENDING",
            intent_data,
        )
        .await
        .map_err(|_| {
            ServiceError::InvalidOperation(
                "Module log cleanup intent could not be persisted; no files were removed".into(),
            )
        })?;
        let log_dir = CONFIG.runtime.log_dir();
        let preview_id = preview.preview_id.clone();
        let selection = cleanup_selection(&preview);
        let today = Utc::now().date_naive();
        let execution_preview = preview.clone();
        let execution = match task::spawn_blocking(move || {
            execute_cleanup(&log_dir, &execution_preview, today)
        })
        .await
        {
            Ok(result) => result,
            Err(error) => Err(join_error(error)),
        };
        match execution {
            Ok(result) => {
                let status = if result.partial { "PARTIAL" } else { "SUCCESS" };
                let failure_category = (!result.failures.is_empty()).then_some("item_failed");
                let result_data = operation_data(
                    "result",
                    json!({
                        "previewId": preview_id,
                        "intentId": intent_id,
                        "files": selection["files"].clone()
                    }),
                    json!({
                        "removedCount": result.removed.len(),
                        "retainedCount": result.retained.len(),
                        "failureCount": result.failures.len(),
                        "partial": result.partial
                    }),
                    failure_category,
                );
                if audit_with_status(
                    pool,
                    user,
                    "MODULE_LOG_CLEANUP_CONFIRM",
                    "Persisted module log cleanup result".into(),
                    status,
                    result_data,
                )
                .await
                .is_err()
                {
                    return Err(ServiceError::InvalidOperation(format!(
                        "Module log cleanup result could not be persisted after intent {intent_id}; outcome is partial or unknown and the intent remains durable"
                    )));
                }
                Ok(result)
            }
            Err(error) => {
                let result_audit = audit_with_status(
                    pool,
                    user,
                    "MODULE_LOG_CLEANUP_CONFIRM",
                    "Module log cleanup execution failed".into(),
                    "FAILED",
                    operation_data(
                        "result",
                        json!({ "previewId": preview_id, "intentId": intent_id }),
                        json!({ "removedCount": 0, "retainedCount": 0, "failureCount": 1 }),
                        Some("cleanup_execution_failed"),
                    ),
                )
                .await;
                if result_audit.is_err() {
                    return Err(ServiceError::InvalidOperation(format!(
                        "Module log cleanup failed and its result could not be persisted; intent {intent_id} remains durable"
                    )));
                }
                Err(error)
            }
        }
    }
}
