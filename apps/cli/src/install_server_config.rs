//! Root-only Monitor server configuration parsing and rendering.
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    fs::{File, OpenOptions},
    io::Read,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
    path::{Path, PathBuf},
};

pub(super) struct SourceConfig {
    pub(super) admin: Vec<u8>,
    pub(super) secondary: Vec<u8>,
    pub(super) admin_port: u16,
    pub(super) secondary_port: u16,
    pub(super) owner_password: Vec<u8>,
}
impl SourceConfig {
    pub(super) fn read(
        path: &Path,
        selection: &crate::install_server_selection::ServerSelection,
    ) -> Result<Self, String> {
        let values = parse_source(&read_source(path)?, selection)?;
        let notify = selection.notify;
        let admin_root =
            runtime_root(&values, "RUSTZEN_ADMIN_RUNTIME_ROOT", "/var/lib/rustzen-admin")?;
        let secondary_root = runtime_root(
            &values,
            &format!("RUSTZEN_{}_RUNTIME_ROOT", secondary_service(selection)),
            &selection.secondary_runtime_root(),
        )?;
        let admin_db = PathBuf::from(
            values
                .get("RUSTZEN_ADMIN_SQLITE_PATH")
                .ok_or("Monitor server config source is incomplete")?,
        );
        let secondary_db = PathBuf::from(
            values
                .get(selection.secondary_sqlite_key())
                .ok_or("Monitor server config source is incomplete")?,
        );
        if admin_db != admin_root.join("admin.db")
            || secondary_db != secondary_root.join(selection.secondary_database())
        {
            return Err("selected database path is outside its service runtime root".into());
        }
        let mut admin_values = values.clone();
        admin_values.insert("RUSTZEN_RUNTIME_ROOT".into(), admin_root.display().to_string());
        let mut secondary_values = values.clone();
        secondary_values
            .insert("RUSTZEN_RUNTIME_ROOT".into(), secondary_root.display().to_string());
        let secondary_keys: &[&str] = match selection.secondary {
            "monitor" => MONITOR_KEYS,
            "insights" => INSIGHTS_KEYS,
            _ => return Err("selected server preset is invalid".into()),
        };
        let mut admin = render(&admin_values, admin_keys(selection))?;
        let mut secondary = render(&secondary_values, secondary_keys)?;
        if notify {
            admin.extend(render(&admin_values, ADMIN_NOTIFY_KEYS)?);
            secondary.extend(render(&secondary_values, MONITOR_NOTIFY_KEYS)?);
        }
        let admin_port = port(&values, "RUSTZEN_ADMIN_PORT", 9801)?;
        let secondary_port = port(&values, selection.secondary_port_key(), selection.secondary_port_default())?;
        if admin_port == 0 || secondary_port == 0 || admin_port == secondary_port {
            return Err("Monitor server config ports are invalid".into());
        }
        let owner_password = values
            .get("RUSTZEN_BOOTSTRAP_OWNER_PASSWORD")
            .ok_or("Monitor server owner credential is missing")?
            .as_bytes()
            .to_vec();
        if !(12..=1024).contains(&owner_password.len())
            || owner_password.contains(&b'\n')
            || owner_password.contains(&b'\r')
            || owner_password == b"replace-me"
        {
            return Err("Monitor server owner credential is invalid".into());
        }
        Ok(Self { admin, secondary, admin_port, secondary_port, owner_password })
    }
}

fn secondary_service(selection: &crate::install_server_selection::ServerSelection) -> String {
    selection.secondary.to_ascii_uppercase()
}

fn admin_keys(selection: &crate::install_server_selection::ServerSelection) -> &'static [&'static str] {
    match selection.secondary {
        "monitor" => ADMIN_MONITOR_KEYS,
        "insights" => ADMIN_INSIGHTS_KEYS,
        _ => unreachable!("validated secondary service"),
    }
}

const ADMIN_MONITOR_KEYS: &[&str] = &[
    "RUSTZEN_ADMIN_HOST",
    "RUSTZEN_ADMIN_PORT",
    "RUSTZEN_ADMIN_SQLITE_PATH",
    "RUSTZEN_DB_CONN_TIMEOUT",
    "RUSTZEN_DB_IDLE_TIMEOUT",
    "RUSTZEN_DB_MAX_CONN",
    "RUSTZEN_DB_MIN_CONN",
    "RUSTZEN_ENV",
    "RUSTZEN_INTERNAL_HOST",
    "RUSTZEN_IPC_TOKEN",
    "RUSTZEN_JWT_EXPIRATION",
    "RUSTZEN_JWT_SECRET",
    "RUSTZEN_MONITOR_PORT",
    "RUSTZEN_RUNTIME_ROOT",
    "RUSTZEN_TIMEZONE",
];
const ADMIN_INSIGHTS_KEYS: &[&str] = &[
    "RUSTZEN_ADMIN_HOST",
    "RUSTZEN_ADMIN_PORT",
    "RUSTZEN_ADMIN_SQLITE_PATH",
    "RUSTZEN_DB_CONN_TIMEOUT",
    "RUSTZEN_DB_IDLE_TIMEOUT",
    "RUSTZEN_DB_MAX_CONN",
    "RUSTZEN_DB_MIN_CONN",
    "RUSTZEN_ENV",
    "RUSTZEN_INTERNAL_HOST",
    "RUSTZEN_INSIGHTS_PORT",
    "RUSTZEN_IPC_TOKEN",
    "RUSTZEN_JWT_EXPIRATION",
    "RUSTZEN_JWT_SECRET",
    "RUSTZEN_RUNTIME_ROOT",
    "RUSTZEN_TIMEZONE",
];
use crate::install_server_notification_config::{ADMIN_NOTIFY_KEYS, MONITOR_NOTIFY_KEYS};

const MONITOR_KEYS: &[&str] = &[
    "RUSTZEN_DB_CONN_TIMEOUT",
    "RUSTZEN_DB_IDLE_TIMEOUT",
    "RUSTZEN_DB_MAX_CONN",
    "RUSTZEN_DB_MIN_CONN",
    "RUSTZEN_ENV",
    "RUSTZEN_INTERNAL_HOST",
    "RUSTZEN_IPC_TOKEN",
    "RUSTZEN_MONITOR_AGENT_TOKEN",
    "RUSTZEN_MONITOR_PORT",
    "RUSTZEN_MONITOR_SQLITE_PATH",
    "RUSTZEN_RUNTIME_ROOT",
    "RUSTZEN_TIMEZONE",
];
const INSIGHTS_KEYS: &[&str] = &[
    "RUSTZEN_DB_CONN_TIMEOUT",
    "RUSTZEN_DB_IDLE_TIMEOUT",
    "RUSTZEN_DB_MAX_CONN",
    "RUSTZEN_DB_MIN_CONN",
    "RUSTZEN_ENV",
    "RUSTZEN_INSIGHTS_PORT",
    "RUSTZEN_INSIGHTS_SQLITE_PATH",
    "RUSTZEN_INTERNAL_HOST",
    "RUSTZEN_IPC_TOKEN",
    "RUSTZEN_RUNTIME_ROOT",
    "RUSTZEN_TIMEZONE",
];

fn parse_source(
    bytes: &[u8],
    selection: &crate::install_server_selection::ServerSelection,
) -> Result<BTreeMap<String, String>, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "Monitor server config source is invalid")?;
    let notify = selection.notify;
    let allowed = admin_keys(selection)
        .iter()
        .chain(match selection.secondary {
            "monitor" => MONITOR_KEYS,
            "insights" => INSIGHTS_KEYS,
            _ => return Err("selected server preset is invalid".into()),
        })
        .chain(ADMIN_NOTIFY_KEYS)
        .chain(MONITOR_NOTIFY_KEYS)
        .copied()
        .filter(|key| *key != "RUSTZEN_RUNTIME_ROOT")
        .map(str::to_owned)
        .chain([
            "RUSTZEN_BOOTSTRAP_OWNER_PASSWORD".to_owned(),
            "RUSTZEN_ADMIN_RUNTIME_ROOT".to_owned(),
            format!("RUSTZEN_{}_RUNTIME_ROOT", secondary_service(selection)),
        ])
        .collect::<BTreeSet<_>>();
    let mut result: BTreeMap<String, String> = BTreeMap::new();
    for line in text.lines() {
        let (key, value) = line.split_once('=').ok_or("Monitor server config source is invalid")?;
        if !allowed.contains(key)
            || value.is_empty()
            || value.contains(['\n', '\r'])
            || value.chars().next().is_some_and(char::is_whitespace)
            || value.chars().last().is_some_and(char::is_whitespace)
            || value.contains(['\'', '"', '\\'])
            || result.insert(key.into(), value.into()).is_some()
        {
            return Err("Monitor server config source is invalid".into());
        }
    }
    if !notify && result.keys().any(|key| key.starts_with("RUSTZEN_NOTIFICATION_")) {
        return Err("Monitor server config source contains notification keys".into());
    }
    if selection.secondary == "insights"
        && result.keys().any(|key| key.starts_with("RUSTZEN_MONITOR"))
    {
        return Err("Monitor server config source contains Monitor keys".into());
    }
    let required = if notify {
        vec![
            "RUSTZEN_NOTIFICATION_EVENT_KEY",
            "RUSTZEN_NOTIFICATION_EVENT_KEY_ID",
            "RUSTZEN_NOTIFICATION_INGRESS_PORT",
            "RUSTZEN_NOTIFICATION_INGRESS_URL",
        ]
    } else {
        vec![]
    };
    let mut keys: Vec<&str> = vec![
        "RUSTZEN_ENV",
        "RUSTZEN_JWT_SECRET",
        "RUSTZEN_IPC_TOKEN",
        "RUSTZEN_ADMIN_SQLITE_PATH",
        "RUSTZEN_BOOTSTRAP_OWNER_PASSWORD",
        "RUSTZEN_ADMIN_RUNTIME_ROOT",
    ];
    match selection.secondary {
        "monitor" => keys.extend(["RUSTZEN_MONITOR_AGENT_TOKEN", "RUSTZEN_MONITOR_SQLITE_PATH"]),
        "insights" => keys.push("RUSTZEN_INSIGHTS_SQLITE_PATH"),
        _ => return Err("selected server preset is invalid".into()),
    }
    let runtime_key =
        format!("RUSTZEN_{}_RUNTIME_ROOT", secondary_service(selection));
    if result.get(&runtime_key).is_none_or(String::is_empty) {
        return Err("Monitor server config source is incomplete".into());
    }
    for key in required.into_iter().chain(keys) {
        if result.get(key).is_none_or(String::is_empty) {
            return Err("Monitor server config source is incomplete".into());
        }
    }
    if result.get("RUSTZEN_ENV").map(String::as_str) != Some("production")
        || result.values().any(|value| value == "replace-me" || value.contains("placeholder"))
    {
        return Err("Monitor server config source is invalid".into());
    }
    Ok(result)
}
fn runtime_root(
    values: &BTreeMap<String, String>,
    key: &str,
    expected: &str,
) -> Result<PathBuf, String> {
    let root = PathBuf::from(values.get(key).ok_or("Monitor server runtime root is missing")?);
    if root != Path::new(expected) {
        return Err("Monitor server runtime root is invalid".into());
    }
    Ok(root)
}
fn render(values: &BTreeMap<String, String>, keys: &[&str]) -> Result<Vec<u8>, String> {
    let mut out = String::new();
    for key in keys {
        if let Some(value) = values.get(*key) {
            out.push_str(key);
            out.push('=');
            out.push_str(value);
            out.push('\n');
        } else if matches!(
            *key,
            "RUSTZEN_JWT_SECRET"
                | "RUSTZEN_IPC_TOKEN"
                | "RUSTZEN_MONITOR_AGENT_TOKEN"
                | "RUSTZEN_ADMIN_SQLITE_PATH"
                | "RUSTZEN_MONITOR_SQLITE_PATH"
                | "RUSTZEN_INSIGHTS_SQLITE_PATH"
                | "RUSTZEN_ENV"
                | "RUSTZEN_NOTIFICATION_EVENT_KEY"
                | "RUSTZEN_NOTIFICATION_EVENT_KEY_ID"
                | "RUSTZEN_NOTIFICATION_INGRESS_PORT"
                | "RUSTZEN_NOTIFICATION_INGRESS_URL"
        ) {
            return Err("Monitor server config source is incomplete".into());
        }
    }
    Ok(out.into_bytes())
}
fn port(values: &BTreeMap<String, String>, key: &str, default: u16) -> Result<u16, String> {
    values
        .get(key)
        .map(|x| x.parse().map_err(|_| "Monitor server config port is invalid".into()))
        .unwrap_or(Ok(default))
}

fn read_source(path: &Path) -> Result<Vec<u8>, String> {
    if !path.is_absolute() {
        return Err("Monitor server config source must be absolute".into());
    }
    let mut parent = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open("/")
        .map_err(|_| "Monitor server config source parent is unavailable")?;
    for part in path.parent().ok_or("Monitor server config source has no parent")?.components() {
        let part = match part {
            std::path::Component::RootDir => continue,
            std::path::Component::Normal(part) => part,
            _ => return Err("Monitor server config source parent is unsafe".into()),
        };
        let part = CString::new(part.as_bytes())
            .map_err(|_| "Monitor server config source parent is invalid")?;
        let raw = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                part.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err("Monitor server config source parent is unavailable".into());
        }
        parent = unsafe { File::from_raw_fd(raw) };
        let meta =
            parent.metadata().map_err(|_| "Monitor server config source parent is unavailable")?;
        if !meta.is_dir() || meta.uid() != 0 || meta.mode() & 0o022 != 0 {
            return Err("Monitor server config source parent is unsafe".into());
        }
    }
    let name = path.file_name().ok_or("Monitor server config source has no name")?;
    let name =
        CString::new(name.as_bytes()).map_err(|_| "Monitor server config source is invalid")?;
    let raw = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if raw < 0 {
        return Err("Monitor server config source is unavailable".into());
    }
    let mut file = unsafe { File::from_raw_fd(raw) };
    let meta = file.metadata().map_err(|_| "Monitor server config source is unavailable")?;
    if !meta.is_file()
        || meta.uid() != 0
        || !matches!(meta.mode() & 0o777, 0o400 | 0o600)
        || meta.len() > 16 * 1024
    {
        return Err("Monitor server config source is unsafe".into());
    }
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    file.read_to_end(&mut bytes).map_err(|_| "Monitor server config source is unreadable")?;
    let after = file.metadata().map_err(|_| "Monitor server config source is unreadable")?;
    if bytes.len() as u64 != meta.len()
        || meta.dev() != after.dev()
        || meta.ino() != after.ino()
        || meta.mtime() != after.mtime()
        || meta.mtime_nsec() != after.mtime_nsec()
    {
        return Err("Monitor server config source changed while read".into());
    }
    Ok(bytes)
}

#[cfg(test)]
#[path = "install_server_config_tests.rs"]
mod tests;
