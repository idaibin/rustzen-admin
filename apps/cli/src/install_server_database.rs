use crate::{
    install_admission::PrivateParent,
    install_crypto::hash,
    install_server_activation_journal::{DatabaseJournal, fault},
    install_server_config::SourceConfig,
    install_server_identity::{self, ServiceIdentity},
    install_server_release::ServerRelease,
    install_service_parent::ServiceParent,
};
use std::{path::Path, process::Command};

const MAX_DATABASE_BYTES: usize = 256 * 1024 * 1024;

pub(super) fn publish(
    source: &SourceConfig,
    release: &ServerRelease,
    tuple_sha256: &str,
    admin: &ServiceIdentity,
    monitor: &ServiceIdentity,
) -> Result<(), String> {
    let var_lib = PrivateParent::open(Path::new("/var/lib"))?;
    let admin_parent = ServiceParent::create(&var_lib, "rustzen-admin", admin.uid, admin.gid)?;
    let monitor_parent =
        ServiceParent::create(&var_lib, "rustzen-monitor", monitor.uid, monitor.gid)?;
    let suffix = tuple_sha256.get(..16).ok_or("activation tuple identity is invalid")?;
    let admin_stage = format!(".admin.db.rz-staging-{suffix}");
    let monitor_stage = format!(".monitor.db.rz-staging-{suffix}");

    let journal = match DatabaseJournal::load(tuple_sha256)? {
        Some(journal) => journal,
        None => {
            admin_parent.absent("admin.db")?;
            monitor_parent.absent("monitor.db")?;
            admin_parent.remove_regular_owned(&admin_stage, admin.uid, admin.gid)?;
            monitor_parent.remove_regular_owned(&monitor_stage, monitor.uid, monitor.gid)?;
            let admin_path = Path::new("/var/lib/rustzen-admin").join(&admin_stage);
            let monitor_path = Path::new("/var/lib/rustzen-monitor").join(&monitor_stage);
            if let Err(error) = initialize(source, release, &admin_path, &monitor_path) {
                let _ = admin_parent.remove_regular_owned(&admin_stage, admin.uid, admin.gid);
                let _ =
                    monitor_parent.remove_regular_owned(&monitor_stage, monitor.uid, monitor.gid);
                return Err(error);
            }
            let admin_bytes = admin_parent.read_regular_owned(
                &admin_stage,
                MAX_DATABASE_BYTES,
                admin.uid,
                admin.gid,
            )?;
            let monitor_bytes = monitor_parent.read_regular_owned(
                &monitor_stage,
                MAX_DATABASE_BYTES,
                monitor.uid,
                monitor.gid,
            )?;
            let journal = DatabaseJournal::new(
                tuple_sha256.into(),
                admin_stage.clone(),
                monitor_stage.clone(),
                &admin_bytes,
                &monitor_bytes,
            );
            journal.publish()?;
            journal
        }
    };
    if journal.admin_stage != admin_stage || journal.monitor_stage != monitor_stage {
        return Err("activation database journal has invalid staging names".into());
    }
    fault("rename-admin")?;
    publish_one(&admin_parent, &admin_stage, "admin.db", admin, &journal.admin_sha256)?;
    fault("rename-monitor")?;
    publish_one(&monitor_parent, &monitor_stage, "monitor.db", monitor, &journal.monitor_sha256)
}

pub(super) fn verify(
    source: &SourceConfig,
    release: &ServerRelease,
    admin: &ServiceIdentity,
    monitor: &ServiceIdentity,
) -> Result<(), String> {
    let admin_parent =
        ServiceParent::open(Path::new("/var/lib/rustzen-admin"), admin.uid, admin.gid)?;
    let monitor_parent =
        ServiceParent::open(Path::new("/var/lib/rustzen-monitor"), monitor.uid, monitor.gid)?;
    admin_parent.require_regular_owned("admin.db", admin.uid, admin.gid)?;
    monitor_parent.require_regular_owned("monitor.db", monitor.uid, monitor.gid)?;
    run_database_command(source, release, "admin", "validate-database", None)?;
    run_database_command(source, release, "monitor", "validate-database", None)?;
    Ok(())
}

fn run_database_command(
    source: &SourceConfig,
    release: &ServerRelease,
    owner: &str,
    mode: &str,
    database: Option<&Path>,
) -> Result<(), String> {
    let (user, binary, config, database_key, schema_key, data_key) = match owner {
        "admin" => (
            "rz-admin",
            "rz-admin",
            source.admin.as_slice(),
            "RUSTZEN_ADMIN_SQLITE_PATH",
            "RUSTZEN_ADMIN_SCHEMA_FINGERPRINT",
            "RUSTZEN_ADMIN_DATA_CONTRACT_ID",
        ),
        "monitor" => (
            "rz-monitor",
            "rz-monitor",
            source.monitor.as_slice(),
            "RUSTZEN_MONITOR_SQLITE_PATH",
            "RUSTZEN_MONITOR_SCHEMA_FINGERPRINT",
            "RUSTZEN_MONITOR_DATA_CONTRACT_ID",
        ),
        _ => return Err("selected database owner is invalid".into()),
    };
    let mut command = Command::new("/usr/sbin/runuser");
    command.args(["-u", user, "--", &format!("/opt/rz/current/bin/{binary}"), mode]);
    for line in
        std::str::from_utf8(config).map_err(|_| "selected database config is invalid")?.lines()
    {
        let (key, value) = line.split_once('=').ok_or("selected database config is invalid")?;
        command.env(
            key,
            if key == database_key {
                database.and_then(Path::to_str).unwrap_or(value)
            } else {
                value
            },
        );
    }
    command
        .env("RUSTZEN_BUILD_ID", &release.build_id)
        .env("RUSTZEN_COMPOSITION_ID", &release.composition_id)
        .env(
            schema_key,
            release
                .schema_fingerprints
                .get(owner)
                .ok_or("selected schema identity is unavailable")?,
        )
        .env(
            data_key,
            release.data_contract_ids.get(owner).ok_or("selected data identity is unavailable")?,
        );
    if command.status().map_err(|_| "database command could not start")?.success() {
        Ok(())
    } else {
        Err(format!("selected {owner} database {mode} failed"))
    }
}

fn publish_one(
    parent: &ServiceParent,
    stage: &str,
    destination: &str,
    identity: &ServiceIdentity,
    expected_hash: &str,
) -> Result<(), String> {
    match (parent.exists(stage)?, parent.exists(destination)?) {
        (true, false) => {
            require_hash(parent, stage, identity, expected_hash)?;
            parent.rename_noreplace(stage, destination)
        }
        // A valid journal plus an absent stage proves this destination was
        // published by the no-replace rename. SQLite may legitimately change
        // the live file during credential verification or a failed start.
        (false, true) => Ok(()),
        _ => Err("activation database publication state is invalid".into()),
    }
}

fn require_hash(
    parent: &ServiceParent,
    name: &str,
    identity: &ServiceIdentity,
    expected_hash: &str,
) -> Result<(), String> {
    let bytes = parent.read_regular_owned(name, MAX_DATABASE_BYTES, identity.uid, identity.gid)?;
    if hash(&bytes) == expected_hash {
        Ok(())
    } else {
        Err("activation database differs from journal".into())
    }
}

fn initialize(
    source: &SourceConfig,
    release: &ServerRelease,
    admin: &Path,
    monitor: &Path,
) -> Result<(), String> {
    install_server_identity::bootstrap_at(source, admin)?;
    run_database_command(source, release, "admin", "bind-database", Some(admin))?;
    run_database_command(source, release, "monitor", "init-db", Some(monitor))?;
    run_database_command(source, release, "monitor", "bind-database", Some(monitor))
}
