use crate::install_server_activation_process::{systemctl, validate_selected_config};
use crate::{
    install_admission::{PrivateParent, PublishError},
    install_crypto::read_regular,
    install_server_activation_journal::{DatabaseJournal, fault},
    install_server_activation_state::ServerActivationState,
    install_server_config::SourceConfig,
    install_server_database,
    install_server_identity::{self, ServiceIdentity},
    install_server_layout::make_payload_executable,
    install_server_readiness,
    install_server_release::ServerRelease,
};
use serde::Serialize;
use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    process::Command,
};

const ROOT: &str = "/opt/rz";
const ADMIN_CONFIG: &str = "rz-admin.env";

pub(super) struct ActivationInput {
    pub(super) config: PathBuf,
    /// Secondary service family the invoking subcommand names.
    pub(super) secondary: &'static str,
}

#[derive(Serialize)]
pub(super) struct Activated {
    config: [String; 2],
    unit: &'static str,
}

pub(super) fn activate(input: &ActivationInput) -> Result<Activated, String> {
    require_root()?;
    let release = ServerRelease::load()?;
    if release.selection.secondary != input.secondary {
        return Err("installed release preset differs from the activation command".into());
    }
    let source = SourceConfig::read(&input.config, &release.selection)?;
    validate_selected_config(&source, &release)?;
    let secondary_config = release.selection.secondary_config();

    let root = PrivateParent::open(Path::new(ROOT))?;
    let unit_parent = PrivateParent::open(Path::new("/etc/systemd/system"))?;
    reject_unselected_state(&release.selection)?;
    make_payload_executable(&release)?;
    let (admin, secondary) = install_server_identity::ensure_pair(&release.selection)?;
    let state = ServerActivationState::acquire(
        &release.preset,
        &release.build_id,
        &source.admin,
        &source.secondary,
        &release.units,
        &release.schema_fingerprints,
        &release.data_contract_ids,
    )?;
    let tuple_sha256 = state.tuple_sha256();

    if state.already_complete()? {
        require_complete(&root, &unit_parent, &source, &release, &admin, &secondary)?;
        install_server_identity::verify_owner(&source)?;
        systemctl(&["is-enabled", "--quiet", "rz.target"])?;
        require_ready(&source, &release)?;
        DatabaseJournal::remove(&tuple_sha256)?;
        return Ok(result(&release));
    }

    install_server_database::publish(&source, &release, &tuple_sha256, &admin, &secondary)?;
    install_server_database::verify(&source, &release, &admin, &secondary)?;
    install_server_identity::verify_owner(&source)?;
    publish_config(&root, ADMIN_CONFIG, &source.admin, admin.gid)?;
    publish_config(&root, &secondary_config, &source.secondary, secondary.gid)?;
    publish(
        &root.open_child_directory("config")?,
        "rz-release.env",
        &release_binding(&release),
        0,
        0o644,
    )?;
    for name in release.selection.units {
        publish(
            &unit_parent,
            name,
            release.units.get(name).ok_or("selected unit is unavailable")?,
            0,
            0o644,
        )?;
    }
    fault("daemon-reload")?;
    systemctl(&["daemon-reload"])?;
    let activated = (|| {
        fault("start")?;
        systemctl(&["start", "rz.target"])?;
        fault("readiness")?;
        require_ready(&source, &release)?;
        fault("enable")?;
        systemctl(&["enable", "rz.target"])?;
        fault("marker")?;
        state.complete()
    })();
    if let Err(error) = activated {
        let _ = systemctl(&["stop", "rz.target"]);
        let _ = systemctl(&["disable", "rz.target"]);
        return Err(error);
    }
    DatabaseJournal::remove(&tuple_sha256)?;
    Ok(result(&release))
}

fn result(release: &ServerRelease) -> Activated {
    Activated {
        config: [
            "/opt/rz/config/rz-admin.env".to_owned(),
            format!("/opt/rz/config/{}", release.selection.secondary_config()),
        ],
        unit: "rz.target",
    }
}

fn require_root() -> Result<(), String> {
    if unsafe { libc::geteuid() } == 0 {
        Ok(())
    } else {
        Err("Monitor server activation requires root".into())
    }
}

fn publish_config(root: &PrivateParent, name: &str, bytes: &[u8], gid: u32) -> Result<(), String> {
    root.create_child_directory("config", 0o711)?;
    publish(&root.open_child_directory("config")?, name, bytes, gid, 0o640)
}

fn publish(
    parent: &PrivateParent,
    name: &str,
    bytes: &[u8],
    gid: u32,
    mode: u32,
) -> Result<(), String> {
    let path = parent.child_path(name)?;
    if let Ok(meta) = fs::symlink_metadata(&path) {
        if meta.file_type().is_symlink()
            || !meta.is_file()
            || meta.uid() != 0
            || meta.gid() != gid
            || meta.mode() & 0o777 != mode
            || read_regular(&path, 64 * 1024)? != bytes
        {
            return Err("activation destination differs from requested tuple".into());
        }
        return Ok(());
    }
    match parent.publish_regular_noreplace(name, bytes, 0, gid, mode) {
        Ok(()) => Ok(()),
        Err(PublishError::Conflict) => Err("activation destination publication conflict".into()),
        Err(error) => Err(error.to_string()),
    }
}

fn require_complete(
    root: &PrivateParent,
    units: &PrivateParent,
    source: &SourceConfig,
    release: &ServerRelease,
    admin: &ServiceIdentity,
    secondary: &ServiceIdentity,
) -> Result<(), String> {
    let config = root.open_child_directory("config")?;
    let config_meta = config.metadata()?;
    if config_meta.uid() != 0 || config_meta.gid() != 0 || config_meta.mode() & 0o777 != 0o711 {
        return Err("activation destination differs from requested tuple".into());
    }
    let binding = release_binding(release);
    for (name, bytes, gid, mode) in [
        (ADMIN_CONFIG, source.admin.as_slice(), admin.gid, 0o640),
        (
            release.selection.secondary_config().as_str(),
            source.secondary.as_slice(),
            secondary.gid,
            0o640,
        ),
        ("rz-release.env", binding.as_slice(), 0, 0o644),
    ] {
        require_file(&config, name, bytes, gid, mode)?;
    }
    for name in release.selection.units {
        require_file(
            units,
            name,
            release.units.get(name).ok_or("selected unit is unavailable")?,
            0,
            0o644,
        )?;
    }
    install_server_database::verify(source, release, admin, secondary)
}

fn require_file(
    parent: &PrivateParent,
    name: &str,
    bytes: &[u8],
    gid: u32,
    mode: u32,
) -> Result<(), String> {
    let path = parent.child_path(name)?;
    let meta = fs::symlink_metadata(&path)
        .map_err(|_| "activated Monitor server destination is unavailable")?;
    if meta.file_type().is_symlink()
        || !meta.is_file()
        || meta.uid() != 0
        || meta.gid() != gid
        || meta.mode() & 0o777 != mode
        || read_regular(&path, 64 * 1024)? != bytes
    {
        return Err("activation destination differs from requested tuple".into());
    }
    Ok(())
}

fn release_binding(release: &ServerRelease) -> Vec<u8> {
    format!(
        "RUSTZEN_BUILD_ID={}\nRUSTZEN_COMPOSITION_ID={}\nRUSTZEN_ADMIN_SCHEMA_FINGERPRINT={}\nRUSTZEN_ADMIN_DATA_CONTRACT_ID={}\nRUSTZEN_{}_SCHEMA_FINGERPRINT={}\nRUSTZEN_{}_DATA_CONTRACT_ID={}\n",
        release.build_id,
        release.composition_id,
        release.schema_fingerprints["admin"],
        release.data_contract_ids["admin"],
        release.selection.secondary.to_ascii_uppercase(),
        release.schema_fingerprints[release.selection.secondary],
        release.selection.secondary.to_ascii_uppercase(),
        release.data_contract_ids[release.selection.secondary],
    )
    .into_bytes()
}

fn reject_unselected_state(
    selection: &crate::install_server_selection::ServerSelection,
) -> Result<(), String> {
    for service in selection.excluded_services() {
        for path in [
            format!("/etc/systemd/system/{service}.service"),
            format!("/usr/lib/systemd/system/{service}.service"),
            format!("/lib/systemd/system/{service}.service"),
            format!("/opt/rz/config/{service}.env"),
            format!("/var/lib/rustzen-{}", service.trim_start_matches("rz-")),
        ] {
            if fs::symlink_metadata(path).is_ok() {
                return Err(
                    "unselected Rustzen service state conflicts with Monitor activation".into()
                );
            }
        }
        require_unit_not_found(&format!("{service}.service"))?;
    }
    Ok(())
}

fn require_unit_not_found(unit: &str) -> Result<(), String> {
    let output = Command::new("/usr/bin/systemctl")
        .args([
            "show",
            "--property=LoadState",
            "--property=FragmentPath",
            "--property=UnitFileState",
            unit,
        ])
        .output()
        .map_err(|_| "unselected service admission check failed")?;
    if !output.status.success() {
        return Err("unselected service admission check failed".into());
    }
    let text = std::str::from_utf8(&output.stdout)
        .map_err(|_| "unselected service admission check failed")?;
    let fields = text
        .lines()
        .filter_map(|line| line.split_once('='))
        .collect::<std::collections::BTreeMap<_, _>>();
    if fields.get("LoadState") == Some(&"not-found")
        && fields.get("FragmentPath") == Some(&"")
        && fields.get("UnitFileState") == Some(&"")
        && fields.len() == 3
    {
        Ok(())
    } else {
        Err("unselected Rustzen service unit conflicts with Monitor activation".into())
    }
}

fn require_ready(source: &SourceConfig, release: &ServerRelease) -> Result<(), String> {
    install_server_readiness::verify_service_process("rz-admin.service", "rz-admin")?;
    install_server_readiness::verify_service_process(
        release.selection.units[1],
        release.selection.secondary_binary(),
    )?;
    install_server_readiness::wait(
        source.admin_port,
        source.secondary_port,
        &release.build_id,
        &release.composition_id,
    )
}
