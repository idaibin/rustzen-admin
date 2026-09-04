use super::invalid;
use super::{BINARIES, SUPPORTED_ARCHES, SYSTEMD_FILES};
use crate::common::error::ServiceError;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
    path::{Component, Path},
};

pub(super) fn inspect_archive(content: &[u8], version: &str) -> Result<String, ServiceError> {
    let mut archive = tar::Archive::new(Cursor::new(content));
    let mut seen = BTreeSet::new();
    let mut files = BTreeMap::new();
    let mut root = None;

    for entry in archive.entries().map_err(|_| invalid("Release bundle is not a valid tar"))? {
        let mut entry = entry.map_err(|_| invalid("Release bundle contains an invalid entry"))?;
        let path = entry
            .path()
            .map_err(|_| invalid("Release bundle contains an invalid path"))?
            .into_owned();
        validate_path(&path)?;
        let path = path
            .to_str()
            .ok_or_else(|| invalid("Release bundle paths must be UTF-8"))?
            .trim_end_matches('/')
            .to_string();
        if !seen.insert(path.clone()) {
            return Err(invalid("Release bundle contains duplicate paths"));
        }
        let top = path.split('/').next().unwrap_or_default().to_string();
        if root.as_ref().is_some_and(|known| known != &top) {
            return Err(invalid("Release bundle must contain exactly one root directory"));
        }
        root.get_or_insert(top);

        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            continue;
        }
        if !entry_type.is_file() {
            return Err(invalid("Release bundle may contain only directories and regular files"));
        }
        let mode = entry
            .header()
            .mode()
            .map_err(|_| invalid("Release bundle contains an invalid file mode"))?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|_| invalid("Release bundle contains an unreadable file"))?;
        files.insert(path, (mode, bytes));
    }

    let root = root.ok_or_else(|| invalid("Release bundle is empty"))?;
    let arch = SUPPORTED_ARCHES
        .into_iter()
        .find(|arch| root == format!("rz-{version}-{arch}"))
        .ok_or_else(|| invalid("Release bundle root does not match version and architecture"))?;
    validate_file_set(&root, &files)?;
    validate_binaries(&root, version, arch, &files)?;
    validate_systemd(&root, &files)?;
    validate_support_files(&root, &files)?;
    Ok(arch.to_string())
}

pub(super) fn validate_path(path: &Path) -> Result<(), ServiceError> {
    if path.is_absolute()
        || path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(invalid("Release bundle contains an unsafe path"));
    }
    Ok(())
}

fn validate_file_set(
    root: &str,
    files: &BTreeMap<String, (u32, Vec<u8>)>,
) -> Result<(), ServiceError> {
    let expected = BINARIES
        .iter()
        .map(|name| format!("{root}/bin/{name}"))
        .chain(SYSTEMD_FILES.iter().map(|name| format!("{root}/systemd/{name}")))
        .chain([
            format!("{root}/config/rz.env"),
            format!("{root}/config/rz-reports.env"),
            format!("{root}/setup-layout.sh"),
        ])
        .collect::<BTreeSet<_>>();
    let actual = files.keys().cloned().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(invalid("Release bundle file set is incomplete or contains unknown files"));
    }
    Ok(())
}

fn validate_binaries(
    root: &str,
    version: &str,
    arch: &str,
    files: &BTreeMap<String, (u32, Vec<u8>)>,
) -> Result<(), ServiceError> {
    for binary in BINARIES {
        let (mode, data) = file(files, &format!("{root}/bin/{binary}"))?;
        if mode & 0o111 == 0 {
            return Err(invalid("Release bundle binaries must be executable"));
        }
        if detect_elf_arch(data)? != arch {
            return Err(invalid("Release bundle binaries must use one architecture"));
        }
        let marker = format!(
            "RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary={binary}\nversion={version}\n"
        );
        if !data.windows(marker.len()).any(|window| window == marker.as_bytes()) {
            return Err(invalid(
                "Release bundle binary identity marker is missing or inconsistent",
            ));
        }
    }
    Ok(())
}

fn validate_systemd(
    root: &str,
    files: &BTreeMap<String, (u32, Vec<u8>)>,
) -> Result<(), ServiceError> {
    let target = text_file(files, &format!("{root}/systemd/rz.target"))?;
    reject_requires(target)?;
    let wants = directive_values(target, "Wants");
    if SYSTEMD_FILES[1..].iter().any(|unit| !wants.contains(*unit)) {
        return Err(invalid("rz.target must Want recovery and all four services"));
    }

    let recovery = text_file(files, &format!("{root}/systemd/rz-recovery.service"))?;
    reject_requires(recovery)?;
    if !has_directive(recovery, "PartOf", "rz.target")
        || !has_directive(recovery, "Restart", "on-failure")
        || !has_directive(recovery, "ExecStart", "/opt/rz/current/bin/rz-admin update recover")
        || directive_values(recovery, "StartLimitIntervalSec").is_empty()
        || directive_values(recovery, "StartLimitBurst").is_empty()
        || SYSTEMD_FILES[2..].iter().any(|unit| {
            !directive_values(recovery, "Before").split_whitespace().any(|value| value == *unit)
        })
    {
        return Err(invalid("Release bundle contains an invalid recovery unit"));
    }

    let specs = [
        ("rz-admin.service", "/opt/rz/current/bin/rz-admin serve"),
        ("rz-monitor.service", "/opt/rz/current/bin/rz-monitor controller"),
        ("rz-insights.service", "/opt/rz/current/bin/rz-insights serve"),
        ("rz-reports.service", "/opt/rz/current/bin/rz-reports serve"),
    ];
    for (unit, command) in specs {
        let text = text_file(files, &format!("{root}/systemd/{unit}"))?;
        reject_requires(text)?;
        if !has_directive(text, "PartOf", "rz.target")
            || !has_directive(text, "Restart", "on-failure")
            || !has_directive(text, "ExecStart", command)
            || !directive_values(text, "After")
                .split_whitespace()
                .any(|value| value == "rz-recovery.service")
            || !has_directive(
                text,
                "ExecCondition",
                "/usr/bin/test ! -e /opt/rz/data/recovery-blocked",
            )
            || directive_values(text, "StartLimitIntervalSec").is_empty()
            || directive_values(text, "StartLimitBurst").is_empty()
        {
            return Err(invalid("Release bundle contains an invalid service topology"));
        }
    }
    Ok(())
}

fn validate_support_files(
    root: &str,
    files: &BTreeMap<String, (u32, Vec<u8>)>,
) -> Result<(), ServiceError> {
    let (config_mode, config) = file(files, &format!("{root}/config/rz.env"))?;
    if config_mode & 0o111 != 0 || config.is_empty() {
        return Err(invalid("Release bundle config/rz.env is invalid"));
    }
    let (reports_config_mode, reports_config) =
        file(files, &format!("{root}/config/rz-reports.env"))?;
    if reports_config_mode & 0o111 != 0 || reports_config.is_empty() {
        return Err(invalid("Release bundle config/rz-reports.env is invalid"));
    }
    let (script_mode, script) = file(files, &format!("{root}/setup-layout.sh"))?;
    if script_mode & 0o111 == 0 || !script.starts_with(b"#!/") {
        return Err(invalid("Release bundle setup-layout.sh is invalid"));
    }
    Ok(())
}

fn file<'a>(
    files: &'a BTreeMap<String, (u32, Vec<u8>)>,
    path: &str,
) -> Result<&'a (u32, Vec<u8>), ServiceError> {
    files.get(path).ok_or_else(|| invalid("Release bundle is incomplete"))
}

fn text_file<'a>(
    files: &'a BTreeMap<String, (u32, Vec<u8>)>,
    path: &str,
) -> Result<&'a str, ServiceError> {
    std::str::from_utf8(&file(files, path)?.1)
        .map_err(|_| invalid("Release bundle deployment files must be UTF-8"))
}

fn has_directive(text: &str, name: &str, expected: &str) -> bool {
    directive_values(text, name).split_whitespace().any(|value| value == expected)
        || directive_values(text, name) == expected
}

fn directive_values<'a>(text: &'a str, name: &str) -> &'a str {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .find_map(|line| line.strip_prefix(name).and_then(|line| line.strip_prefix('=')))
        .map(str::trim)
        .unwrap_or_default()
}

fn reject_requires(text: &str) -> Result<(), ServiceError> {
    if !directive_values(text, "Requires").is_empty() {
        return Err(invalid("Release services must not use Requires coupling"));
    }
    Ok(())
}

pub(super) fn detect_elf_arch(data: &[u8]) -> Result<&'static str, ServiceError> {
    if data.len() < 20 || &data[..4] != b"\x7fELF" {
        return Err(invalid("Release bundle binaries must be ELF executables"));
    }
    match u16::from_le_bytes([data[18], data[19]]) {
        62 => Ok("x86_64"),
        183 => Ok("aarch64"),
        _ => Err(invalid("Unsupported release bundle architecture")),
    }
}
