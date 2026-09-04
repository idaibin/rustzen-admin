use crate::{
    install_activation_state::ActivationState,
    install_admission::{PrivateParent, PublishError},
    install_crypto::{hash, read_regular},
    install_pairing::{
        agent_manifest, service_gid, validate_agent_traversal, validate_current_agent_binary,
    },
};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    io::Read,
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    process::Command,
};

const ROOT: &str = "/opt/rz";
const CONFIG_NAME: &str = "rz-monitor-agent.env";
const UNIT_NAME: &str = "rz-monitor-agent.service";

pub(super) struct ActivationInput {
    pub(super) config: PathBuf,
}

#[derive(Serialize)]
pub(super) struct Activated {
    config: &'static str,
    unit: &'static str,
}

pub(super) fn activate(input: &ActivationInput) -> Result<Activated, String> {
    let context = Context::load()?;
    let config = SourceConfig::read(&input.config, &context.endpoint)?;
    let parents = DestinationParents::preflight(&config.bytes, context.gid, &context.unit)?;
    let state =
        ActivationState::acquire(&context.build_id, &hash(&config.bytes), &hash(&context.unit))?;
    if state.already_complete()? {
        parents.require_complete(&config.bytes, context.gid, &context.unit)?;
        return Ok(Activated { config: "/opt/rz/config/rz-monitor-agent.env", unit: UNIT_NAME });
    }
    publish_config(&parents.root, &config.bytes, context.gid)?;
    publish(&parents.unit, UNIT_NAME, &context.unit, 0, 0o644)?;
    for args in [
        ["daemon-reload"].as_slice(),
        ["enable", UNIT_NAME].as_slice(),
        ["--no-block", "start", UNIT_NAME].as_slice(),
    ] {
        systemctl(args)?;
    }
    state.complete()?;
    Ok(Activated { config: "/opt/rz/config/rz-monitor-agent.env", unit: UNIT_NAME })
}

struct DestinationParents {
    root: PrivateParent,
    unit: PrivateParent,
}

impl DestinationParents {
    fn preflight(config: &[u8], gid: u32, unit: &[u8]) -> Result<Self, String> {
        let root = PrivateParent::open(Path::new(ROOT))?;
        let unit_parent = PrivateParent::open(Path::new("/etc/systemd/system"))?;
        let directory = root.child_path("config")?;
        if let Ok(meta) = fs::symlink_metadata(&directory) {
            if meta.file_type().is_symlink()
                || !meta.is_dir()
                || meta.uid() != 0
                || meta.mode() & 0o022 != 0
            {
                return Err("Agent config directory is unsafe".into());
            }
            existing(&directory.join(CONFIG_NAME), config, gid, 0o640)?;
            let config_parent = root.open_child_directory("config")?;
            let metadata = config_parent.metadata()?;
            if metadata.uid() != 0 || metadata.gid() != gid || metadata.mode() & 0o777 != 0o750 {
                return Err("Agent config directory ownership or mode is invalid".into());
            }
        }
        existing(&unit_parent.child_path(UNIT_NAME)?, unit, 0, 0o644)?;
        Ok(Self { root, unit: unit_parent })
    }

    fn require_complete(&self, config: &[u8], gid: u32, unit: &[u8]) -> Result<(), String> {
        let config_parent = self
            .root
            .open_child_directory("config")
            .map_err(|_| "activated Agent config directory is unavailable")?;
        let metadata = config_parent.metadata()?;
        if metadata.uid() != 0 || metadata.gid() != gid || metadata.mode() & 0o777 != 0o750 {
            return Err("Agent config directory ownership or mode is invalid".into());
        }
        required(&config_parent.child_path(CONFIG_NAME)?, config, gid, 0o640)?;
        required(&self.unit.child_path(UNIT_NAME)?, unit, 0, 0o644)
    }
}

fn required(path: &Path, bytes: &[u8], gid: u32, mode: u32) -> Result<(), String> {
    if fs::symlink_metadata(path).is_err() {
        return Err("activated Agent destination is unavailable".into());
    }
    existing(path, bytes, gid, mode)
}

fn existing(path: &Path, bytes: &[u8], gid: u32, mode: u32) -> Result<(), String> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink()
            || !meta.is_file()
            || meta.uid() != 0
            || meta.gid() != gid
            || meta.mode() & 0o777 != mode
        {
            return Err("activation destination is unsafe".into());
        }
        if read_regular(path, 64 * 1024)? != bytes {
            return Err("activation destination differs from requested tuple".into());
        }
    }
    Ok(())
}

struct Context {
    build_id: String,
    gid: u32,
    endpoint: String,
    unit: Vec<u8>,
}

impl Context {
    fn load() -> Result<Self, String> {
        let root = Path::new(ROOT);
        let (manifest, manifest_bytes) = agent_manifest(root)?;
        validate_current_agent_binary(root, &manifest)?;
        let gid = service_gid()?;
        validate_agent_traversal(root, gid)?;
        let profile = root.join("controller-profile.json");
        let metadata =
            fs::symlink_metadata(&profile).map_err(|_| "Agent profile is unavailable")?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.uid() != 0
            || metadata.gid() != gid
            || metadata.mode() & 0o777 != 0o640
        {
            return Err("Agent profile is unsafe".into());
        }
        let profile_value = rustzen_config::read_controller_profile(&profile, gid)?;
        let endpoint = rustzen_config::canonical_monitor_endpoint(&profile_value.endpoint)
            .map_err(|_| "Agent profile endpoint is invalid")?;
        if profile_value.agent_build_id != manifest.build_id
            || profile_value.agent_manifest_sha256 != hash(&manifest_bytes)
            || profile_value.protocol_id != manifest.agent_protocol_contract_id
        {
            return Err("Agent profile differs from retained Agent binding".into());
        }
        let unit =
            root.join("releases").join(&manifest.build_id).join("payload/systemd").join(UNIT_NAME);
        let metadata =
            fs::symlink_metadata(&unit).map_err(|_| "Agent native unit is unavailable")?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.uid() != 0
            || metadata.mode() & 0o777 != 0o644
        {
            return Err("Agent native unit is unsafe".into());
        }
        let bytes = read_regular(&unit, 64 * 1024)?;
        let expected = manifest
            .files
            .iter()
            .find(|entry| entry.path == "systemd/rz-monitor-agent.service")
            .ok_or("Agent native unit is missing from manifest")?;
        if expected.mode != "0644" || expected.sha256 != hash(&bytes) {
            return Err("Agent native unit differs from manifest".into());
        }
        Ok(Self { build_id: manifest.build_id, gid, endpoint, unit: bytes })
    }
}

struct SourceConfig {
    bytes: Vec<u8>,
}

impl SourceConfig {
    fn read(path: &Path, profile_endpoint: &str) -> Result<Self, String> {
        let bytes = read_source(path)?;
        let values = rustzen_config::parse_agent_environment_bytes(&bytes)
            .map_err(|_| "Agent config source is invalid")?;
        if values.endpoint != profile_endpoint {
            return Err("Agent config endpoint differs from Controller profile".into());
        }
        let bytes = format!("RUSTZEN_ENV={}\nRUSTZEN_MONITOR_AGENT_TOKEN={}\nRUSTZEN_MONITOR_CONTROLLER_URL={}\nRUSTZEN_MONITOR_NODE_ID={}\n", values.environment, values.token, values.endpoint, values.node_id).into_bytes();
        Ok(Self { bytes })
    }
}

fn read_source(path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err("Agent config source must be absolute".into());
    }
    let mut current = PathBuf::from("/");
    for component in path.parent().ok_or("Agent config source has no parent")?.components() {
        let std::path::Component::Normal(component) = component else { continue };
        current.push(component);
        let meta = fs::symlink_metadata(&current)
            .map_err(|_| "Agent config source parent is unavailable")?;
        if meta.file_type().is_symlink()
            || !meta.is_dir()
            || meta.uid() != 0
            || meta.mode() & 0o022 != 0
        {
            return Err("Agent config source parent is unsafe".into());
        }
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "Agent config source is unavailable")?;
    let before = file.metadata().map_err(|_| "Agent config source is unavailable")?;
    if !before.is_file()
        || before.uid() != 0
        || !matches!(before.mode() & 0o777, 0o400 | 0o600)
        || before.len() > 16 * 1024
    {
        return Err("Agent config source is unsafe".into());
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    file.read_to_end(&mut bytes).map_err(|_| "Agent config source is unreadable")?;
    let after = file.metadata().map_err(|_| "Agent config source is unreadable")?;
    if before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
        || bytes.len() as u64 != before.len()
    {
        return Err("Agent config source changed while read".into());
    }
    Ok(bytes)
}

fn publish_config(root_parent: &PrivateParent, bytes: &[u8], gid: u32) -> Result<(), String> {
    let directory = root_parent.child_path("config")?;
    match fs::create_dir(&directory) {
        Ok(()) => {
            let path = std::ffi::CString::new(directory.as_os_str().as_bytes())
                .map_err(|_| "Agent config path is invalid")?;
            if unsafe { libc::chown(path.as_ptr(), 0, gid) } != 0
                || unsafe { libc::chmod(path.as_ptr(), 0o750) } != 0
            {
                return Err("Agent config directory is unsafe".into());
            }
            root_parent.sync()?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err("Agent config directory could not be created".into()),
    }
    let path = std::ffi::CString::new(directory.as_os_str().as_bytes())
        .map_err(|_| "Agent config path is invalid")?;
    if unsafe { libc::chown(path.as_ptr(), 0, gid) } != 0
        || unsafe { libc::chmod(path.as_ptr(), 0o750) } != 0
    {
        return Err("Agent config directory is unsafe".into());
    }
    let config_parent = root_parent.open_child_directory("config")?;
    publish(&config_parent, CONFIG_NAME, bytes, gid, 0o640)
}

fn publish(
    parent: &PrivateParent,
    name: &str,
    bytes: &[u8],
    gid: u32,
    mode: u32,
) -> Result<(), String> {
    let path = parent.child_path(name)?;
    if fs::symlink_metadata(&path).is_ok() {
        existing(&path, bytes, gid, mode)?;
        return parent.sync();
    }
    match parent.publish_regular_noreplace(name, bytes, 0, gid, mode) {
        Ok(()) => Ok(()),
        Err(PublishError::Conflict) => {
            existing(&path, bytes, gid, mode)?;
            parent.sync()
        }
        Err(error) => Err(error.to_string()),
    }
}

fn systemctl(args: &[&str]) -> Result<(), String> {
    #[cfg(debug_assertions)]
    let binary = std::env::var_os("RUSTZEN_SYSTEMCTL_RECORDER")
        .unwrap_or_else(|| "/usr/bin/systemctl".into());
    #[cfg(not(debug_assertions))]
    let binary = std::ffi::OsString::from("/usr/bin/systemctl");
    let status =
        Command::new(binary).args(args).status().map_err(|_| "systemctl invocation failed")?;
    if status.success() { Ok(()) } else { Err("systemctl activation failed".into()) }
}
