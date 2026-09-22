#![allow(unused_imports)]
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use super::super::{
    DeployService, SYSTEMD_UNITS, UpdateJournal, backup_database_online, backup_database_paths,
    claim_update_request, cleanup_failed_release, create_dir_all_durable, load_installed_bundle,
    read_update_journal_at, read_update_request, restore_database_paths,
    restore_release_state_with_paths, roll_services_with, run_boot_recovery, swap_symlink,
    update_request_path, validate_upload_size, validate_version, write_update_journal_at,
};
use crate::features::manage::deploy::types::{
    DeployComponent, DeploymentPayload, ExpireVersionRequest,
};

#[cfg(unix)]
#[test]
fn update_request_reader_rejects_links_and_insecure_files() {
    use std::os::unix::fs::PermissionsExt;

    let root = std::env::temp_dir().join(format!("rz-update-request-{}", uuid::Uuid::new_v4()));
    let directory = root.join("data/update-requests");
    fs::create_dir_all(&directory).expect("request directory");
    let request = update_request_path(&root);
    let outside = root.join("outside.json");
    fs::write(&outside, br#"{"releaseId":7,"deployedBy":"owner"}"#).expect("outside request");
    std::os::unix::fs::symlink(&outside, &request).expect("request link");
    assert!(read_update_request(&request).is_err());
    fs::remove_file(&request).expect("remove link");

    fs::write(&request, br#"{"releaseId":7,"deployedBy":"owner"}"#).expect("request");
    let mut permissions = fs::metadata(&request).expect("request metadata").permissions();
    permissions.set_mode(0o644);
    fs::set_permissions(&request, permissions).expect("insecure mode");
    assert!(read_update_request(&request).is_err());

    let mut permissions = fs::metadata(&request).expect("request metadata").permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(&request, permissions).expect("private mode");
    let parsed = read_update_request(&request).expect("regular request");
    assert_eq!(parsed.release_id, 7);
    assert_eq!(parsed.deployed_by, "owner");
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn update_request_claim_moves_the_exact_pending_file_before_reading() {
    use std::os::unix::fs::PermissionsExt;

    let root = std::env::temp_dir().join(format!("rz-update-claim-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(root.join("data/update-requests")).expect("request directory");
    let pending = update_request_path(&root);
    fs::write(&pending, br#"{"releaseId":8,"deployedBy":"owner"}"#).expect("pending request");
    let mut permissions = fs::metadata(&pending).expect("pending metadata").permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(&pending, permissions).expect("private mode");
    let claimed = claim_update_request(&root).expect("claim pending request");
    assert!(!pending.exists());
    let parsed = read_update_request(&claimed).expect("read claimed request");
    assert_eq!(parsed.release_id, 8);
    fs::remove_file(claimed).expect("remove claimed request");
    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn boot_recovery_requeues_services_after_a_failed_first_attempt() {
    let root = std::env::temp_dir().join(format!("rz-boot-recovery-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).expect("recovery root");
    let sentinel = root.join("recovery-blocked");
    let journal = root.join("update-state.json");
    fs::write(&journal, b"pending").expect("journal marker");

    let first = run_boot_recovery(
        &sentinel,
        &journal,
        || async { Err(std::io::Error::other("injected recovery failure").into()) },
        || async { panic!("failed recovery must not requeue services") },
    )
    .await;
    assert!(first.is_err());
    assert!(sentinel.is_file());

    let requeued = Arc::new(Mutex::new(false));
    let observed = Arc::clone(&requeued);
    let sentinel_during_requeue = sentinel.clone();
    run_boot_recovery(
        &sentinel,
        &journal,
        || async { Ok(()) },
        || async move {
            assert!(sentinel_during_requeue.is_file());
            *observed.lock().expect("requeue state") = true;
            Ok(())
        },
    )
    .await
    .expect("second recovery attempt");
    assert!(*requeued.lock().expect("requeue result"));
    assert!(!sentinel.exists());
    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn boot_recovery_keeps_services_blocked_when_requeue_fails() {
    let root = std::env::temp_dir().join(format!("rz-boot-requeue-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).expect("recovery root");
    let sentinel = root.join("recovery-blocked");
    let journal = root.join("update-state.json");
    fs::write(&journal, b"pending").expect("journal marker");

    let result = run_boot_recovery(
        &sentinel,
        &journal,
        || async { Ok(()) },
        || async { Err(std::io::Error::other("injected requeue failure").into()) },
    )
    .await;
    assert!(result.is_err());
    assert!(sentinel.is_file());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn durable_directory_creation_persists_the_full_parent_chain() {
    let root = std::env::temp_dir().join(format!("rz-durable-dir-{}", uuid::Uuid::new_v4()));
    let leaf = root.join("data/backups/release");
    create_dir_all_durable(&leaf).expect("durable directory chain");
    assert!(leaf.is_dir());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn validates_release_upload_size_and_version() {
    assert!(validate_upload_size(b"bundle").is_ok());
    assert_eq!(validate_version("1.2.3".to_string()).expect("version"), "1.2.3");
}

#[test]
fn rejects_invalid_release_inputs() {
    assert!(validate_upload_size(&[]).is_err());
    assert!(validate_version("../module".to_string()).is_err());
    assert!(validate_version(".".to_string()).is_err());
    assert!(validate_version("..".to_string()).is_err());
}

#[tokio::test]
async fn release_record_is_retained_when_file_removal_fails() {
    let root = std::env::temp_dir().join(format!("rz-delete-release-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).expect("test root");
    let database = root.join("admin.db");
    let artifact = root.join("artifact-is-a-directory");
    fs::create_dir(&artifact).expect("invalid artifact fixture");
    let pool = crate::infra::db::create_pool_for_path(&database).await.expect("test pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    let service = DeployService::new(pool.clone());
    let item = service
        .repo
        .insert(&DeploymentPayload {
            component: DeployComponent::Release,
            version: "1.2.3".to_string(),
            arch: "x86_64".to_string(),
            file_path: artifact.to_string_lossy().to_string(),
            file_size: 1,
            file_hash: "fixture".to_string(),
            frontend_hash: "a".repeat(64),
            backend_hash: "b".repeat(64),
            notes: None,
        })
        .await
        .expect("insert release");
    assert_eq!(item.frontend_hash, "a".repeat(64));
    assert_eq!(item.backend_hash, "b".repeat(64));

    assert!(service.delete(item.id).await.is_err());
    service.expire(item.id, ExpireVersionRequest { notes: None }).await.unwrap();
    assert!(service.cleanup_expired(None).await.is_err());
    let visible: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM deploy_versions WHERE id=? AND deleted_at IS NULL",
    )
    .bind(item.id)
    .fetch_one(&pool)
    .await
    .expect("visible release row");
    assert_eq!(visible, 1);

    pool.close().await;
    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn release_file_is_removed_before_its_record_is_hidden() {
    let root = std::env::temp_dir().join(format!("rz-delete-release-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).expect("test root");
    let database = root.join("admin.db");
    let artifact = root.join("release.tar");
    fs::write(&artifact, b"fixture").expect("artifact fixture");
    let pool = crate::infra::db::create_pool_for_path(&database).await.expect("test pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    let service = DeployService::new(pool.clone());
    let item = service
        .repo
        .insert(&DeploymentPayload {
            component: DeployComponent::Release,
            version: "1.2.4".to_string(),
            arch: "x86_64".to_string(),
            file_path: artifact.to_string_lossy().to_string(),
            file_size: 7,
            file_hash: "fixture".to_string(),
            frontend_hash: "a".repeat(64),
            backend_hash: "b".repeat(64),
            notes: None,
        })
        .await
        .expect("insert release");

    service.delete(item.id).await.expect("delete release");
    assert!(!artifact.exists());
    let visible: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM deploy_versions WHERE id=? AND deleted_at IS NULL",
    )
    .bind(item.id)
    .fetch_one(&pool)
    .await
    .expect("hidden release row");
    assert_eq!(visible, 0);

    pool.close().await;
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn installed_bundle_selection_uses_the_running_architecture() {
    let root = std::env::temp_dir().join(format!("rz-installed-arch-{}", uuid::Uuid::new_v4()));
    let releases = root.join("data/releases");
    fs::create_dir_all(&releases).expect("release uploads");
    for arch in ["x86_64", "aarch64"] {
        fs::write(
            releases.join(format!("rz-1.2.3-{arch}.tar")),
            crate::features::manage::deploy::bundle::tests::fixture("1.2.3", arch),
        )
        .expect("bundle");
    }
    let (_, selected) = load_installed_bundle(&root, "1.2.3", "x86_64").expect("selected bundle");
    assert_eq!(selected.arch, "x86_64");
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[tokio::test]
async fn fresh_install_bootstraps_exactly_one_current_release_row() {
    let version = "1.2.3";
    let arch = "x86_64";
    let root = std::env::temp_dir().join(format!("rz-bootstrap-current-{}", uuid::Uuid::new_v4()));
    let data = crate::features::manage::deploy::bundle::tests::fixture(version, arch);
    let info =
        crate::features::manage::deploy::bundle::validate_bundle(&data, version, false, None)
            .expect("bundle");
    fs::create_dir_all(root.join("data/db/admin")).expect("Admin ownership source");
    crate::features::manage::deploy::bundle::install_bundle(&data, &info, version, 1, &root)
        .expect("installed release");
    fs::create_dir_all(root.join("data/releases")).expect("release store");
    fs::write(root.join(format!("data/releases/rz-{version}-{arch}.tar")), &data)
        .expect("stored bundle");
    std::os::unix::fs::symlink(format!("releases/{version}"), root.join("current"))
        .expect("current link");

    let pool = crate::infra::db::create_pool_for_path(&root.join("admin-test.db"))
        .await
        .expect("test pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    let service = DeployService::new(pool.clone());
    service.bootstrap_installed_current_at(&root, true).await.expect("bootstrap current");
    service.bootstrap_installed_current_at(&root, true).await.expect("idempotent bootstrap");
    let current: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM deploy_versions WHERE is_current = 1 AND version = ? AND arch = ?",
    )
    .bind(version)
    .bind(arch)
    .fetch_one(&pool)
    .await
    .expect("current count");
    assert_eq!(current, 1);

    let candidate_version = "2.0.0";
    let candidate =
        crate::features::manage::deploy::bundle::tests::fixture(candidate_version, arch);
    let candidate_info = crate::features::manage::deploy::bundle::validate_bundle(
        &candidate,
        candidate_version,
        false,
        None,
    )
    .expect("candidate bundle");
    crate::features::manage::deploy::bundle::install_bundle(
        &candidate,
        &candidate_info,
        candidate_version,
        2,
        &root,
    )
    .expect("candidate install");
    fs::write(root.join(format!("data/releases/rz-{candidate_version}-{arch}.tar")), candidate)
        .expect("candidate stored bundle");
    swap_symlink(&root.join("current"), Path::new("releases/2.0.0"))
        .expect("candidate current link");
    fs::write(root.join("data/update-state.json"), b"rollout in progress")
        .expect("update journal sentinel");
    service.bootstrap_installed_current_at(&root, true).await.expect("rollout startup validation");
    let still_current: String =
        sqlx::query_scalar("SELECT version FROM deploy_versions WHERE is_current = 1")
            .fetch_one(&pool)
            .await
            .expect("current version");
    assert_eq!(still_current, version);
    pool.close().await;
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn update_journal_round_trips_interrupted_state() {
    let root = std::env::temp_dir().join(format!("rz-update-{}", uuid::Uuid::new_v4()));
    let path = root.join("data/update-state.json");
    let journal = UpdateJournal {
        release_id: 7,
        backup_dir: root.join("backup"),
        link: root.join("current"),
        old_target: PathBuf::from("releases/old"),
        new_release_dir: root.join("releases/new"),
        install_staging_dir: root.join("releases/.new.7.installing"),
        installed_by_update: true,
        stage: "switched".to_string(),
        restarted_units: Vec::new(),
    };

    write_update_journal_at(&path, &journal).expect("write journal");
    let restored = read_update_journal_at(&path).expect("read journal").expect("journal");
    assert_eq!(restored.release_id, journal.release_id);
    assert_eq!(restored.backup_dir, journal.backup_dir);
    assert_eq!(restored.old_target, journal.old_target);
    assert_eq!(restored.stage, "switched");
    assert!(restored.restarted_units.is_empty());
    fs::remove_dir_all(root).expect("remove journal root");
}
