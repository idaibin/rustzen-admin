#![allow(unused_imports)]
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use super::super::{
    DeployService, SYSTEMD_UNITS, UpdateJournal, backup_database_online, backup_database_paths,
    cleanup_failed_release, complete_update_failure, create_dir_all_durable, load_installed_bundle,
    read_update_journal_at, restore_database_paths, restore_release_state_with_paths,
    roll_services_with, run_boot_recovery, swap_symlink, validate_upload_size, validate_version,
    write_update_journal_at,
};

#[test]
fn successful_rollback_still_propagates_the_original_update_failure() {
    let journal_removed = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&journal_removed);
    let error = complete_update_failure(
        std::io::Error::other("injected upgrade failure").into(),
        Ok(()),
        move || {
            observed.store(true, Ordering::SeqCst);
            Ok(())
        },
    )
    .expect_err("successful rollback must not report a successful update");
    assert!(journal_removed.load(Ordering::SeqCst));
    assert!(error.to_string().contains("injected upgrade failure"));
}
use crate::features::manage::deploy::types::{
    DeployComponent, DeploymentPayload, ExpireVersionRequest,
};

#[tokio::test]
async fn every_service_health_failure_stops_the_roll_sequence() {
    for (failed_index, failed_unit) in SYSTEMD_UNITS.iter().enumerate() {
        let root = std::env::temp_dir().join(format!("rz-health-gate-{}", uuid::Uuid::new_v4()));
        let journal_path = root.join("update-state.json");
        let mut journal = UpdateJournal {
            release_id: 9,
            backup_dir: root.join("backup"),
            link: root.join("current"),
            old_target: PathBuf::from("releases/old"),
            new_release_dir: root.join("releases/new"),
            install_staging_dir: root.join("releases/.new.9.installing"),
            installed_by_update: true,
            stage: "switched".to_string(),
            restarted_units: Vec::new(),
        };
        let attempts = Arc::new(Mutex::new(Vec::new()));
        let recorded_attempts = Arc::clone(&attempts);
        let failed_unit = failed_unit.to_string();

        let result = roll_services_with(&mut journal, &journal_path, move |unit, _url| {
            let attempts = Arc::clone(&recorded_attempts);
            let failed_unit = failed_unit.clone();
            async move {
                attempts.lock().expect("attempt lock").push(unit.clone());
                if unit == failed_unit {
                    Err(std::io::Error::other("injected health failure").into())
                } else {
                    Ok(())
                }
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(
            *attempts.lock().expect("attempt lock"),
            SYSTEMD_UNITS[..=failed_index].iter().map(|unit| unit.to_string()).collect::<Vec<_>>()
        );
        assert_eq!(journal.restarted_units.len(), failed_index + 1);
        let saved =
            read_update_journal_at(&journal_path).expect("read journal").expect("saved journal");
        assert_eq!(saved.restarted_units, journal.restarted_units);
        fs::remove_dir_all(root).expect("remove health gate root");
    }
}

#[cfg(unix)]
#[test]
fn apply_and_each_health_gate_rollback_restore_one_release_boundary() {
    for failed_index in 0..SYSTEMD_UNITS.len() {
        let root = std::env::temp_dir()
            .join(format!("rz-apply-rollback-{failed_index}-{}", uuid::Uuid::new_v4()));
        let releases = root.join("releases");
        let data = root.join("data");
        let backup = root.join("backup");
        fs::create_dir_all(releases.join("old")).expect("old release");
        fs::create_dir_all(releases.join("new")).expect("new release");
        fs::create_dir_all(&data).expect("data");
        fs::create_dir_all(&backup).expect("backup");
        let paths =
            ["monitor", "insights", "reports", "admin"].map(|name| data.join(format!("{name}.db")));
        for path in &paths {
            fs::write(path, "old-database").expect("old database");
        }
        backup_database_paths(&paths, &backup).expect("backup");
        for path in &paths {
            fs::write(path, "new-database").expect("new database");
        }

        let current = root.join("current");
        std::os::unix::fs::symlink("releases/old", &current).expect("initial current");
        swap_symlink(&current, Path::new("releases/new")).expect("apply release");
        assert_eq!(
            fs::read_link(&current).expect("applied current"),
            PathBuf::from("releases/new")
        );

        let staging = releases.join(format!(".new.{failed_index}.installing"));
        fs::create_dir(&staging).expect("staging");
        let journal = UpdateJournal {
            release_id: failed_index as i64 + 1,
            backup_dir: backup,
            link: current.clone(),
            old_target: PathBuf::from("releases/old"),
            new_release_dir: releases.join("new"),
            install_staging_dir: staging,
            installed_by_update: true,
            stage: format!("restarting:{}", SYSTEMD_UNITS[failed_index]),
            restarted_units: SYSTEMD_UNITS[..=failed_index]
                .iter()
                .map(|unit| unit.to_string())
                .collect(),
        };
        let units = journal.restarted_units.iter().map(String::as_str).collect::<Vec<_>>();
        restore_release_state_with_paths(&journal, &units, &paths).expect("restore state");
        cleanup_failed_release(&journal).expect("cleanup failed release");

        assert_eq!(
            fs::read_link(&current).expect("rolled back current"),
            PathBuf::from("releases/old")
        );
        assert!(!journal.new_release_dir.exists());
        assert!(!journal.install_staging_dir.exists());
        for (index, path) in paths.iter().enumerate() {
            let expected = if index <= failed_index { "old-database" } else { "new-database" };
            assert_eq!(fs::read_to_string(path).expect("database boundary"), expected);
        }
        fs::remove_dir_all(root).expect("cleanup");
    }
}

#[cfg(unix)]
#[test]
fn interruption_cleanup_preserves_reused_release_directory() {
    let root = std::env::temp_dir().join(format!("rz-reused-release-{}", uuid::Uuid::new_v4()));
    let releases = root.join("releases");
    fs::create_dir_all(releases.join("old")).expect("old release");
    fs::create_dir_all(releases.join("reused")).expect("reused release");
    let current = root.join("current");
    std::os::unix::fs::symlink("releases/old", &current).expect("current");
    let staging = releases.join(".reused.8.installing");
    fs::create_dir(&staging).expect("staging");
    let journal = UpdateJournal {
        release_id: 8,
        backup_dir: root.join("backup"),
        link: current,
        old_target: PathBuf::from("releases/old"),
        new_release_dir: releases.join("reused"),
        install_staging_dir: staging,
        installed_by_update: false,
        stage: "installing".to_string(),
        restarted_units: Vec::new(),
    };
    cleanup_failed_release(&journal).expect("cleanup interruption");
    assert!(journal.new_release_dir.is_dir());
    assert!(!journal.install_staging_dir.exists());
    fs::remove_dir_all(root).expect("cleanup");
}
