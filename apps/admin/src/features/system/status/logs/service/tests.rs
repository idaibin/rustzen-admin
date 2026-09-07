use super::*;
use sqlx::sqlite::SqlitePoolOptions;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

fn temp_log_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!("rustzen-admin-module-logs-{}", Uuid::new_v4()));
    fs::create_dir(&path).unwrap();
    path
}

fn selector(module: &str, date: &str, _root: &Path) -> ParsedSelector {
    ParsedSelector {
        module: module.into(),
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        file_name: format!("{module}.{date}"),
    }
}

#[test]
fn selector_rejects_unknown_modules_traversal_and_noncanonical_dates() {
    assert!(parse_module("../../etc").is_err());
    assert!(parse_date("2026-1-01").is_err());
    assert!(
        parse_selector(&ModuleLogFileSelector {
            module: "admin/../monitor".into(),
            date: "2026-01-01".into(),
        })
        .is_err()
    );
}

#[test]
fn file_name_parser_compares_the_raw_date_to_the_canonical_form() {
    assert!(parse_file_name("admin.2026-1-01").is_none());
    assert!(parse_file_name("admin.2026-01-1").is_none());
    assert!(parse_file_name("admin.2026-01-01.extra").is_none());
    assert_eq!(
        parse_file_name("admin.2026-01-01").map(|(_, date)| date),
        NaiveDate::from_ymd_opt(2026, 1, 1)
    );
}

#[test]
fn cursor_is_opaque_and_bound_to_file_signature_and_selector() {
    let root = Path::new("/var/lib/rustzen/logs");
    let selected = selector("admin", "2026-01-01", root);
    let signature = FileSignature { size: 128, modified: None, identity: FileIdentity::default() };
    let token = encode_cursor(&selected, signature, 64).unwrap();
    assert!(!token.starts_with("r1:"));
    assert!(!token.contains("0000000000000040"));
    assert_eq!(decode_cursor(&token, &selected, signature).unwrap(), 64);

    let other_file = selector("monitor", "2026-01-01", root);
    assert!(decode_cursor(&token, &other_file, signature).is_err());
    assert!(decode_cursor(&token, &selected, FileSignature { size: 129, ..signature },).is_err());
    assert!(decode_cursor("r1:0000000000000040", &selected, signature).is_err());
}

#[test]
fn list_marks_symlink_unreadable_without_following_target() {
    let dir = temp_log_dir();
    fs::write(dir.join("admin.2026-01-01"), b"safe").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink("admin.2026-01-01", dir.join("monitor.2026-01-01")).unwrap();
    let items = list_in(&dir, None, None).unwrap();
    assert!(items.iter().any(|item| item.module == "admin" && item.readable));
    #[cfg(unix)]
    assert!(items.iter().any(|item| item.module == "monitor" && !item.readable));
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn reports_uses_only_the_fixed_service_account_log_directory() {
    let dir = temp_log_dir();
    let reports_dir = dir.join("reports");
    fs::create_dir(&reports_dir).unwrap();
    fs::write(dir.join("reports.2026-01-01"), b"root-decoy").unwrap();
    fs::write(reports_dir.join("reports.2026-01-01"), b"nested-report-log\n").unwrap();

    let items = list_in(&dir, Some("reports"), None).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].file_name, "reports.2026-01-01");
    let tail = tail_in(&dir, selector("reports", "2026-01-01", &dir), None).unwrap();
    assert_eq!(tail.content, "nested-report-log");

    let request = ModuleLogBackupRequest {
        files: vec![ModuleLogFileSelector { module: "reports".into(), date: "2026-01-01".into() }],
    };
    let archive = build_archive(&dir, request).unwrap();
    let mut tar_archive = tar::Archive::new(archive.bytes.as_slice());
    let mut entries = tar_archive.entries().unwrap();
    assert_eq!(entries.next().unwrap().unwrap().path().unwrap(), Path::new("reports.2026-01-01"));
    assert_eq!(entries.next().unwrap().unwrap().path().unwrap(), Path::new("manifest.json"));
    assert!(entries.next().is_none());

    let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let cutoff = today - Days::new(RETENTION_DAYS);
    let (candidates, failures) = collect_cleanup_candidates(&dir, today, cutoff).unwrap();
    assert!(failures.is_empty());
    assert_eq!(candidates.len(), 1);
    let preview = CleanupPreviewState {
        preview_id: "reports-preview".into(),
        expires_at_instant: Instant::now() + CLEANUP_PREVIEW_TTL,
        cutoff_date: cutoff,
        candidates,
    };
    let result = execute_cleanup(&dir, &preview, today).unwrap();
    assert_eq!(result.removed.len(), 1);
    assert!(!reports_dir.join("reports.2026-01-01").exists());
    assert_eq!(fs::read(dir.join("reports.2026-01-01")).unwrap(), b"root-decoy");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_reports_directory_is_empty_and_non_directory_fails_closed() {
    let dir = temp_log_dir();
    assert!(list_in(&dir, Some("reports"), None).unwrap().is_empty());
    let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let cutoff = today - Days::new(RETENTION_DAYS);
    assert!(collect_cleanup_candidates(&dir, today, cutoff).unwrap().0.is_empty());
    fs::write(dir.join("reports"), b"not-a-directory").unwrap();
    assert!(list_in(&dir, Some("reports"), None).is_err());
    assert!(tail_in(&dir, selector("reports", "2026-01-01", &dir), None).is_err());
    assert!(
        build_archive(
            &dir,
            ModuleLogBackupRequest {
                files: vec![ModuleLogFileSelector {
                    module: "reports".into(),
                    date: "2026-01-01".into(),
                }],
            },
        )
        .is_err()
    );
    assert!(collect_cleanup_candidates(&dir, today, cutoff).is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn reports_directory_symlink_fails_closed() {
    let dir = temp_log_dir();
    let outside = temp_log_dir();
    fs::write(outside.join("reports.2026-01-01"), b"outside").unwrap();
    std::os::unix::fs::symlink(&outside, dir.join("reports")).unwrap();
    assert!(list_in(&dir, Some("reports"), None).is_err());
    assert!(tail_in(&dir, selector("reports", "2026-01-01", &dir), None).is_err());
    assert!(
        build_archive(
            &dir,
            ModuleLogBackupRequest {
                files: vec![ModuleLogFileSelector {
                    module: "reports".into(),
                    date: "2026-01-01".into(),
                }],
            },
        )
        .is_err()
    );
    let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let cutoff = today - Days::new(RETENTION_DAYS);
    assert!(collect_cleanup_candidates(&dir, today, cutoff).is_err());
    fs::remove_dir_all(dir).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[cfg(unix)]
#[test]
fn tail_is_bounded_by_bytes_lines_and_individual_line_size() {
    let dir = temp_log_dir();
    let mut file = fs::File::create(dir.join("admin.2026-01-01")).unwrap();
    for index in 0..(MAX_TAIL_LINES + 20) {
        writeln!(file, "{index}:{}", "x".repeat(MAX_LINE_BYTES + 100)).unwrap();
    }
    let result = tail_in(&dir, selector("admin", "2026-01-01", &dir), None).unwrap();
    assert!(result.truncated);
    assert!(result.byte_count <= MAX_TAIL_BYTES);
    assert!(result.line_count <= MAX_TAIL_LINES);
    assert!(result.content.lines().all(|line| line.len() <= MAX_LINE_BYTES));
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn tail_paginates_10k_short_lines_without_gaps_or_duplicates() {
    let dir = temp_log_dir();
    let mut file = fs::File::create(dir.join("admin.2026-01-01")).unwrap();
    let expected = (0..10_000).map(|index| format!("line-{index:05}")).collect::<Vec<_>>();
    for line in &expected {
        writeln!(file, "{line}").unwrap();
    }

    let mut cursor = None;
    let mut pages = Vec::new();
    loop {
        let page = tail_in(&dir, selector("admin", "2026-01-01", &dir), cursor.as_deref()).unwrap();
        assert!(!page.content.is_empty());
        pages.extend(page.content.lines().map(str::to_owned));
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }
    assert_eq!(pages.len(), expected.len());
    pages.sort();
    assert_eq!(pages, expected);
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn tail_paginates_a_single_unterminated_line_larger_than_the_read_window() {
    let dir = temp_log_dir();
    let expected = "x".repeat(MAX_TAIL_BYTES + MAX_LINE_BYTES + 123);
    fs::write(dir.join("admin.2026-01-01"), &expected).unwrap();

    let mut cursor = None;
    let mut chunks = Vec::new();
    loop {
        let page = tail_in(&dir, selector("admin", "2026-01-01", &dir), cursor.as_deref()).unwrap();
        assert_eq!(page.line_count, 1);
        assert!(!page.content.is_empty());
        assert!(page.byte_count <= MAX_LINE_BYTES);
        assert!(page.truncated);
        chunks.push(page.content);
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => break,
        }
    }
    let reconstructed = chunks.into_iter().rev().collect::<String>();
    assert_eq!(reconstructed, expected);
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn backup_contains_manifest_hash_and_respects_archive_cap() {
    let dir = temp_log_dir();
    fs::write(dir.join("admin.2026-01-01"), b"hello").unwrap();
    let request = ModuleLogBackupRequest {
        files: vec![ModuleLogFileSelector { module: "admin".into(), date: "2026-01-01".into() }],
    };
    let archive = build_archive(&dir, request).unwrap();
    assert!(archive.bytes.windows(b"manifest.json".len()).any(|window| window == b"manifest.json"));
    assert_eq!(archive.archive_sha256, digest_bytes(&archive.bytes));
    assert!(archive.bytes.len() as u64 <= MAX_ARCHIVE_BYTES);
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn cleanup_preview_excludes_today_and_confirmation_is_single_use() {
    let dir = temp_log_dir();
    let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let cutoff = today - Days::new(RETENTION_DAYS);
    fs::write(dir.join("admin.2026-06-01"), b"old").unwrap();
    fs::write(dir.join("admin.2026-08-10"), b"today").unwrap();
    fs::write(dir.join("admin.2026-07-15"), b"inside").unwrap();
    let (candidates, failures) = collect_cleanup_candidates(&dir, today, cutoff).unwrap();
    assert!(failures.is_empty());
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].candidate.date, "2026-06-01");

    let token = "test-token".to_string();
    store_preview(
        token.clone(),
        CleanupPreviewState {
            preview_id: "preview".into(),
            expires_at_instant: Instant::now() + CLEANUP_PREVIEW_TTL,
            cutoff_date: cutoff,
            candidates,
        },
    )
    .unwrap();
    let state = take_preview(&token).unwrap();
    assert!(take_preview(&token).is_err());
    let result = execute_cleanup(&dir, &state, today).unwrap();
    assert_eq!(result.removed.len(), 1);
    assert!(!dir.join("admin.2026-06-01").exists());
    assert!(dir.join("admin.2026-08-10").exists());
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn cleanup_refuses_changed_file_between_preview_and_confirm() {
    let dir = temp_log_dir();
    let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let cutoff = today - Days::new(RETENTION_DAYS);
    let path = dir.join("admin.2026-06-01");
    fs::write(&path, b"old").unwrap();
    let (mut candidates, _) = collect_cleanup_candidates(&dir, today, cutoff).unwrap();
    OpenOptions::new().append(true).open(&path).unwrap().write_all(b"changed").unwrap();
    let preview = CleanupPreviewState {
        preview_id: "preview".into(),
        expires_at_instant: Instant::now() + CLEANUP_PREVIEW_TTL,
        cutoff_date: cutoff,
        candidates: std::mem::take(&mut candidates),
    };
    let result = execute_cleanup(&dir, &preview, today).unwrap();
    assert!(result.removed.is_empty());
    assert_eq!(result.failures.len(), 1);
    assert!(path.exists());
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn cleanup_refuses_same_size_replacement_and_symlink_barriers() {
    let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let cutoff = today - Days::new(RETENTION_DAYS);

    let dir = temp_log_dir();
    let path = dir.join("admin.2026-06-01");
    fs::write(&path, b"old").unwrap();
    let (candidates, _) = collect_cleanup_candidates(&dir, today, cutoff).unwrap();
    fs::rename(&path, dir.join("admin.2026-06-01.replaced")).unwrap();
    fs::write(&path, b"new").unwrap();
    let preview = CleanupPreviewState {
        preview_id: "replacement".into(),
        expires_at_instant: Instant::now() + CLEANUP_PREVIEW_TTL,
        cutoff_date: cutoff,
        candidates,
    };
    let result = execute_cleanup(&dir, &preview, today).unwrap();
    assert!(result.removed.is_empty());
    assert_eq!(result.failures.len(), 1);
    assert!(path.exists());
    fs::remove_dir_all(&dir).unwrap();

    let dir = temp_log_dir();
    let path = dir.join("admin.2026-06-01");
    fs::write(&path, b"old").unwrap();
    let (candidates, _) = collect_cleanup_candidates(&dir, today, cutoff).unwrap();
    fs::remove_file(&path).unwrap();
    fs::write(dir.join("decoy"), b"decoy").unwrap();
    std::os::unix::fs::symlink("decoy", &path).unwrap();
    let preview = CleanupPreviewState {
        preview_id: "symlink".into(),
        expires_at_instant: Instant::now() + CLEANUP_PREVIEW_TTL,
        cutoff_date: cutoff,
        candidates,
    };
    let result = execute_cleanup(&dir, &preview, today).unwrap();
    assert!(result.removed.is_empty());
    assert_eq!(result.failures.len(), 1);
    assert!(path.is_symlink());
    assert!(dir.join("decoy").exists());
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn cleanup_transaction_rejects_same_name_replacement_after_final_check() {
    let dir = temp_log_dir();
    let path = dir.join("admin.2026-06-01");
    let moved_original = dir.join("admin.2026-06-01.original");
    fs::write(&path, b"old").unwrap();
    let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
    let cutoff = today - Days::new(RETENTION_DAYS);
    let (candidates, _) = collect_cleanup_candidates(&dir, today, cutoff).unwrap();
    let candidate = &candidates[0];
    let directory = secure_fs::open_directory(&dir).unwrap().unwrap();
    let result = directory.unlink_if_unchanged_with_barrier(
        &candidate.selector.file_name,
        candidate.signature,
        &candidate.digest,
        || {
            fs::rename(&path, &moved_original).unwrap();
            fs::write(&path, b"new").unwrap();
        },
    );
    assert!(result.is_err());
    assert_eq!(fs::read(&path).unwrap(), b"new");
    assert_eq!(fs::read(&moved_original).unwrap(), b"old");
    fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn audit_records_action_metadata_without_log_content() {
    let pool =
        SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    crate::infra::db::run_migrations(&pool).await.unwrap();
    let user = CurrentUser::new(1, "owner", ["*".to_owned()], true);
    let mut data = operation_data(
        "success",
        json!({
            "files": [{ "module": "admin", "date": "2026-01-01" }]
        }),
        json!({ "fileCount": 1 }),
        None,
    );
    data["archiveSha256"] = json!("abc");
    audit_with_status(
        &pool,
        &user,
        "MODULE_LOG_BACKUP",
        "Backed up one module log file".into(),
        "SUCCESS",
        data,
    )
    .await
    .unwrap();
    let (user_id, username, action, status, data): (i64, String, String, String, Option<String>) =
        sqlx::query_as("SELECT user_id, username, action, status, data FROM operation_logs ORDER BY id DESC LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(user_id, 1);
    assert_eq!(username, "owner");
    assert_eq!(action, "MODULE_LOG_BACKUP");
    assert_eq!(status, "SUCCESS");
    let data = data.unwrap();
    assert!(data.contains("archiveSha256"));
    assert!(data.contains("resultCounts"));
    assert!(data.contains("admin"));
    assert!(!data.contains("log line"));
}

#[tokio::test]
async fn cleanup_intent_and_result_are_durable_and_content_free() {
    let pool =
        SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    crate::infra::db::run_migrations(&pool).await.unwrap();
    let user = CurrentUser::new(7, "owner", ["*".to_owned()], true);
    let selection = json!({
        "previewId": "preview-1",
        "files": [{ "module": "admin", "fileName": "admin.2026-01-01", "sizeBytes": 12 }]
    });
    let intent_id = audit_with_status(
        &pool,
        &user,
        "MODULE_LOG_CLEANUP_CONFIRM",
        "Persisted module log cleanup intent".into(),
        "PENDING",
        operation_data(
            "intent",
            selection.clone(),
            json!({ "removedCount": 0, "retainedCount": 0, "failureCount": 0 }),
            None,
        ),
    )
    .await
    .unwrap();
    let result_id = audit_with_status(
        &pool,
        &user,
        "MODULE_LOG_CLEANUP_CONFIRM",
        "Persisted module log cleanup result".into(),
        "PARTIAL",
        operation_data(
            "result",
            json!({ "intentId": intent_id, "selection": selection }),
            json!({ "removedCount": 0, "retainedCount": 1, "failureCount": 1 }),
            Some("item_failed"),
        ),
    )
    .await
    .unwrap();
    let rows: Vec<(i64, String, String, Option<String>)> = sqlx::query_as(
        "SELECT user_id, status, action, data FROM operation_logs WHERE id IN (?, ?) ORDER BY id",
    )
    .bind(intent_id)
    .bind(result_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, 7);
    assert_eq!(rows[0].1, "PENDING");
    assert_eq!(rows[1].1, "PARTIAL");
    assert!(rows.iter().all(|row| row.2 == "MODULE_LOG_CLEANUP_CONFIRM"));
    assert!(rows.iter().all(|row| !row.3.as_deref().unwrap().contains("log line")));
}
