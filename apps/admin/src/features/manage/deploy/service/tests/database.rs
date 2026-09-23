#![allow(unused_imports)]
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use super::super::{
    DeployService, SYSTEMD_UNITS, UpdateJournal, backup_database_online, backup_database_paths,
    cleanup_failed_release, create_dir_all_durable, load_installed_bundle, read_update_journal_at,
    restore_database_paths, restore_release_state_with_paths, roll_services_with,
    run_boot_recovery, swap_symlink, validate_upload_size, validate_version,
    write_update_journal_at,
};
use crate::features::manage::deploy::types::{
    DeployComponent, DeploymentPayload, ExpireVersionRequest,
};

#[test]
fn four_database_backup_and_restore_are_independent() {
    let root = std::env::temp_dir().join(format!("rz-backup-{}", uuid::Uuid::new_v4()));
    let data = root.join("data");
    let backup = root.join("backup");
    fs::create_dir_all(&data).expect("data dir");
    fs::create_dir_all(&backup).expect("backup dir");
    let paths = ["admin", "monitor", "insights", "reports"]
        .map(|name| data.join(format!("{name}.db")))
        .to_vec();
    for path in &paths {
        let name = path.file_stem().and_then(|value| value.to_str()).expect("name");
        fs::write(path, format!("{name}-database")).expect("database");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o640)).expect("database mode");
        }
        fs::write(PathBuf::from(format!("{}-wal", path.display())), format!("{name}-wal"))
            .expect("wal");
    }

    backup_database_paths(&paths, &backup).expect("backup");
    #[cfg(unix)]
    for path in &paths {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            backup.join(path.file_name().expect("name")),
            fs::Permissions::from_mode(0o600),
        )
        .expect("private backup mode");
    }
    for path in &paths {
        fs::write(path, "corrupt").expect("corrupt");
        let wal = PathBuf::from(format!("{}-wal", path.display()));
        if wal.exists() {
            fs::remove_file(wal).expect("remove wal");
        }
    }
    restore_database_paths(&paths, &backup).expect("restore");

    for path in &paths {
        let name = path.file_stem().and_then(|value| value.to_str()).expect("name");
        assert_eq!(fs::read_to_string(path).expect("database"), format!("{name}-database"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(path).expect("database metadata").permissions().mode() & 0o777,
                0o640
            );
        }
        assert!(!PathBuf::from(format!("{}-wal", path.display())).exists());
    }
    fs::remove_dir_all(root).expect("remove backup root");
}

#[test]
fn incomplete_database_backup_never_modifies_live_databases() {
    let root = std::env::temp_dir().join(format!("rz-backup-preflight-{}", uuid::Uuid::new_v4()));
    let data = root.join("data");
    let backup = root.join("backup");
    fs::create_dir_all(&data).expect("data dir");
    fs::create_dir_all(&backup).expect("backup dir");
    let paths = ["admin", "monitor", "insights", "reports"]
        .map(|name| data.join(format!("{name}.db")))
        .to_vec();
    for path in &paths {
        fs::write(path, "original").expect("original database");
    }
    backup_database_paths(&paths, &backup).expect("backup");
    fs::remove_file(backup.join("insights.db")).expect("remove one backup");
    for path in &paths {
        fs::write(path, "live-after-update").expect("live database");
    }

    assert!(restore_database_paths(&paths, &backup).is_err());
    for path in &paths {
        assert_eq!(fs::read_to_string(path).expect("unchanged live database"), "live-after-update");
    }
    fs::remove_dir_all(root).expect("remove backup root");
}

#[tokio::test]
async fn online_sqlite_backup_is_a_readable_consistent_database() {
    let root = std::env::temp_dir().join(format!("rz-online-backup-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).expect("backup root");
    let source = root.join("source.db");
    let destination = root.join("backup.db");
    let pool = crate::infra::db::create_pool_for_path(&source).await.expect("source pool");
    sqlx::query("CREATE TABLE values_table (value TEXT NOT NULL)")
        .execute(&pool)
        .await
        .expect("create table");
    sqlx::query("INSERT INTO values_table (value) VALUES ('before')")
        .execute(&pool)
        .await
        .expect("insert value");

    backup_database_online(&source, &destination).await.expect("online backup");
    let backup = crate::infra::db::create_pool_for_path(&destination).await.expect("backup pool");
    let value: String = sqlx::query_scalar("SELECT value FROM values_table")
        .fetch_one(&backup)
        .await
        .expect("backup value");
    assert_eq!(value, "before");
    backup.close().await;
    pool.close().await;
    fs::remove_dir_all(root).expect("remove backup root");
}
