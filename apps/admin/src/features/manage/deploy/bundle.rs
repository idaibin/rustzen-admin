use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read},
    path::{Component, Path, PathBuf},
};

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::common::error::ServiceError;

pub const SIGNED_MARKER_BEGIN: &[u8] = b"\nRUSTZEN_BUNDLE_SIGNED_MARKER_BEGIN\n";
pub const SIGNED_MARKER_END: &[u8] = b"\nRUSTZEN_BUNDLE_SIGNED_MARKER_END\n";
const SIGNATURE_PAYLOAD_VERSION: &str = "rustzen-bundle-v1";
const SUPPORTED_ARCHES: [&str; 2] = ["x86_64", "aarch64"];
const BINARIES: [&str; 5] = ["rz", "rz-admin", "rz-monitor", "rz-insights", "rz-reports"];
const SYSTEMD_FILES: [&str; 6] = [
    "rz.target",
    "rz-recovery.service",
    "rz-admin.service",
    "rz-monitor.service",
    "rz-insights.service",
    "rz-reports.service",
];

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BundleInfo {
    pub arch: String,
    pub content_len: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BundleSignatureMarker {
    schema_version: u8,
    component: String,
    version: String,
    arch: String,
    content_sha256: String,
    signature: String,
}

pub fn validate_bundle(
    data: &[u8],
    version: &str,
    signature_required: bool,
    verify_key: Option<&str>,
) -> Result<BundleInfo, ServiceError> {
    let (content, marker) = split_signed_content(data)?;
    let arch = inspect_archive(content, version)?;
    if signature_required {
        let marker = marker.ok_or_else(|| invalid("Signed release bundle is required"))?;
        verify_signature(content, &marker, version, &arch, verify_key)?;
    }
    Ok(BundleInfo { arch, content_len: content.len() })
}

pub fn install_bundle(
    data: &[u8],
    info: &BundleInfo,
    version: &str,
    release_id: i64,
    runtime_root: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let content = data
        .get(..info.content_len)
        .ok_or_else(|| std::io::Error::other("invalid verified bundle length"))?;
    let releases = runtime_root.join("releases");
    let destination = releases.join(version);
    if destination.exists() {
        return Err(std::io::Error::other(format!(
            "release directory already exists: {}",
            destination.display()
        ))
        .into());
    }
    fs::create_dir_all(&releases)?;
    let staging = releases.join(format!(".{version}.{release_id}.installing"));
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    fs::create_dir(&staging)?;

    let result = extract_archive(content, version, &info.arch, &staging).and_then(|()| {
        sync_release_tree(&staging)?;
        fs::rename(&staging, &destination)?;
        sync_directory(&releases)?;
        Ok(())
    });
    if result.is_err()
        && staging.exists()
        && let Err(error) = fs::remove_dir_all(&staging)
    {
        tracing::warn!(%error, path = %staging.display(), "Failed to remove release staging directory");
    }
    result.map(|()| destination)
}

pub fn verify_installed_bundle(
    data: &[u8],
    info: &BundleInfo,
    version: &str,
    release_dir: &Path,
) -> Result<(), ServiceError> {
    if !fs::symlink_metadata(release_dir).is_ok_and(|metadata| metadata.is_dir()) {
        return Err(invalid("Installed release directory is unavailable"));
    }
    let content = data
        .get(..info.content_len)
        .ok_or_else(|| invalid("Verified release bundle length is invalid"))?;
    let root = format!("rz-{version}-{}", info.arch);
    let mut expected = BTreeSet::new();
    let mut archive = tar::Archive::new(Cursor::new(content));
    for entry in archive.entries().map_err(|_| invalid("Release bundle is not a valid tar"))? {
        let mut entry = entry.map_err(|_| invalid("Release bundle contains an invalid entry"))?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry
            .path()
            .map_err(|_| invalid("Release bundle contains an invalid path"))?
            .into_owned();
        let relative = path
            .strip_prefix(&root)
            .map_err(|_| invalid("Release bundle root is invalid"))?
            .to_path_buf();
        let installed = release_dir.join(&relative);
        let metadata = fs::symlink_metadata(&installed)
            .map_err(|_| invalid("Installed release is incomplete"))?;
        if !metadata.file_type().is_file() {
            return Err(invalid("Installed release contains a non-regular member"));
        }
        let expected_mode = entry
            .header()
            .mode()
            .map_err(|_| invalid("Release bundle contains an invalid file mode"))?;
        if !installed_mode_matches(&metadata, expected_mode) {
            return Err(invalid("Installed release member mode does not match the signed bundle"));
        }
        let mut expected_data = Vec::new();
        entry
            .read_to_end(&mut expected_data)
            .map_err(|_| invalid("Release bundle contains an unreadable file"))?;
        if fs::read(&installed).map_err(|_| invalid("Installed release member is unreadable"))?
            != expected_data
        {
            return Err(invalid("Installed release does not match the signed bundle"));
        }
        expected.insert(relative);
    }

    let mut actual = BTreeSet::new();
    collect_installed_files(release_dir, release_dir, &mut actual)?;
    if actual != expected {
        return Err(invalid("Installed release contains an unexpected file set"));
    }
    Ok(())
}

pub fn installed_release_arch(release_dir: &Path) -> Result<&'static str, ServiceError> {
    let admin = release_dir.join("bin/rz-admin");
    let metadata = fs::symlink_metadata(&admin)
        .map_err(|_| invalid("Installed release Admin binary is unavailable"))?;
    if !metadata.file_type().is_file() {
        return Err(invalid("Installed release Admin binary must be a regular file"));
    }
    detect_elf_arch(
        &fs::read(admin).map_err(|_| invalid("Installed release Admin binary is unreadable"))?,
    )
}

#[cfg(unix)]
fn installed_mode_matches(metadata: &fs::Metadata, expected: u32) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o777 == expected & 0o777
}

#[cfg(not(unix))]
fn installed_mode_matches(_metadata: &fs::Metadata, _expected: u32) -> bool {
    true
}

fn collect_installed_files(
    root: &Path,
    directory: &Path,
    files: &mut BTreeSet<PathBuf>,
) -> Result<(), ServiceError> {
    for entry in fs::read_dir(directory).map_err(|_| invalid("Installed release is unreadable"))? {
        let entry = entry.map_err(|_| invalid("Installed release is unreadable"))?;
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|_| invalid("Installed release is unreadable"))?;
        if metadata.file_type().is_symlink() {
            return Err(invalid("Installed release must not contain symlinks"));
        }
        if metadata.is_dir() {
            collect_installed_files(root, &entry.path(), files)?;
        } else if metadata.is_file() {
            files.insert(
                entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|_| invalid("Installed release path is invalid"))?
                    .to_path_buf(),
            );
        } else {
            return Err(invalid("Installed release contains a non-regular member"));
        }
    }
    Ok(())
}

fn split_signed_content(
    data: &[u8],
) -> Result<(&[u8], Option<BundleSignatureMarker>), ServiceError> {
    let Some(begin) = find_last(data, SIGNED_MARKER_BEGIN) else {
        return Ok((data, None));
    };
    let marker_start = begin + SIGNED_MARKER_BEGIN.len();
    let marker_end = data[marker_start..]
        .windows(SIGNED_MARKER_END.len())
        .position(|window| window == SIGNED_MARKER_END)
        .map(|offset| marker_start + offset)
        .ok_or_else(|| invalid("Signed release bundle marker is invalid"))?;
    if marker_end + SIGNED_MARKER_END.len() != data.len() {
        return Err(invalid("Signed release bundle marker must terminate the artifact"));
    }
    let marker = serde_json::from_slice::<BundleSignatureMarker>(&data[marker_start..marker_end])
        .map_err(|_| invalid("Signed release bundle marker is invalid"))?;
    verify_marker_metadata_without_signature(&marker, &data[..begin])?;
    Ok((&data[..begin], Some(marker)))
}

fn verify_marker_metadata_without_signature(
    marker: &BundleSignatureMarker,
    content: &[u8],
) -> Result<(), ServiceError> {
    if marker.schema_version != 1
        || marker.component != "bundle"
        || marker.content_sha256 != sha256_hex(content)
    {
        return Err(invalid("Signed release bundle metadata does not match the artifact"));
    }
    Ok(())
}

fn verify_signature(
    content: &[u8],
    marker: &BundleSignatureMarker,
    version: &str,
    arch: &str,
    verify_key: Option<&str>,
) -> Result<(), ServiceError> {
    let content_hash = sha256_hex(content);
    if marker.schema_version != 1
        || marker.component != "bundle"
        || marker.version != version
        || marker.arch != arch
        || marker.content_sha256 != content_hash
    {
        return Err(invalid("Signed release bundle metadata does not match the upload"));
    }
    let key_hex =
        verify_key.ok_or_else(|| invalid("Release bundle verify key is not configured"))?;
    let key_bytes: [u8; 32] = hex::decode(key_hex)
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| invalid("Release bundle verify key is invalid"))?;
    let signature = Signature::from_slice(
        &hex::decode(&marker.signature)
            .map_err(|_| invalid("Release bundle signature is invalid"))?,
    )
    .map_err(|_| invalid("Release bundle signature is invalid"))?;
    let payload = signature_payload(version, arch, &content_hash);
    VerifyingKey::from_bytes(&key_bytes)
        .map_err(|_| invalid("Release bundle verify key is invalid"))?
        .verify(payload.as_bytes(), &signature)
        .map_err(|_| invalid("Release bundle signature verification failed"))
}

fn signature_payload(version: &str, arch: &str, content_hash: &str) -> String {
    format!(
        "{SIGNATURE_PAYLOAD_VERSION}\ncomponent=bundle\nversion={version}\narch={arch}\ncontent_sha256={content_hash}\n"
    )
}

fn inspect_archive(content: &[u8], version: &str) -> Result<String, ServiceError> {
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

fn validate_path(path: &Path) -> Result<(), ServiceError> {
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
        .chain([format!("{root}/config/rz.env"), format!("{root}/setup-layout.sh")])
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

fn detect_elf_arch(data: &[u8]) -> Result<&'static str, ServiceError> {
    if data.len() < 20 || &data[..4] != b"\x7fELF" {
        return Err(invalid("Release bundle binaries must be ELF executables"));
    }
    match u16::from_le_bytes([data[18], data[19]]) {
        62 => Ok("x86_64"),
        183 => Ok("aarch64"),
        _ => Err(invalid("Unsupported release bundle architecture")),
    }
}

fn extract_archive(
    content: &[u8],
    version: &str,
    arch: &str,
    staging: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = format!("rz-{version}-{arch}");
    let mut archive = tar::Archive::new(Cursor::new(content));
    for entry in archive.entries()? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path()?.into_owned();
        validate_path(&path).map_err(|error| std::io::Error::other(error.to_string()))?;
        let relative = path.strip_prefix(&root)?;
        let destination = staging.join(relative);
        let parent = destination
            .parent()
            .ok_or_else(|| std::io::Error::other("invalid bundle destination"))?;
        fs::create_dir_all(parent)?;
        let mut output = fs::OpenOptions::new().write(true).create_new(true).open(&destination)?;
        std::io::copy(&mut entry, &mut output)?;
        set_mode(&destination, entry.header().mode()?)?;
        output.sync_all()?;
    }
    Ok(())
}

fn sync_release_tree(root: &Path) -> Result<(), std::io::Error> {
    for directory in [root.join("bin"), root.join("systemd"), root.join("config")] {
        sync_directory(&directory)?;
    }
    sync_directory(root)
}

fn sync_directory(path: &Path) -> Result<(), std::io::Error> {
    fs::File::open(path)?.sync_all()
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode & 0o777))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<(), std::io::Error> {
    Ok(())
}

fn find_last(data: &[u8], needle: &[u8]) -> Option<usize> {
    data.windows(needle.len()).rposition(|window| window == needle)
}

fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

fn invalid(message: &str) -> ServiceError {
    ServiceError::InvalidOperation(message.to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use std::{fs, io::Write, path::Path};

    use ed25519_dalek::{Signer, SigningKey};

    use super::{
        BINARIES, BundleInfo, SIGNED_MARKER_BEGIN, SIGNED_MARKER_END, install_bundle, sha256_hex,
        signature_payload, validate_bundle, validate_path, verify_installed_bundle,
    };

    pub(crate) fn fixture(version: &str, arch: &str) -> Vec<u8> {
        let root = format!("rz-{version}-{arch}");
        let mut output = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut output);
            for binary in BINARIES {
                let mut elf = vec![0_u8; 64];
                elf[..4].copy_from_slice(b"\x7fELF");
                let machine = if arch == "x86_64" { 62_u16 } else { 183_u16 };
                elf[18..20].copy_from_slice(&machine.to_le_bytes());
                write!(
                    elf,
                    "RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary={binary}\nversion={version}\n"
                )
                .expect("marker");
                append(&mut builder, &format!("{root}/bin/{binary}"), 0o755, &elf);
            }
            append(
                &mut builder,
                &format!("{root}/systemd/rz.target"),
                0o644,
                b"[Unit]\nWants=rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service\n[Install]\nWantedBy=multi-user.target\n",
            );
            append(
                &mut builder,
                &format!("{root}/systemd/rz-recovery.service"),
                0o644,
                b"[Unit]\nPartOf=rz.target\nBefore=rz-admin.service rz-monitor.service rz-insights.service rz-reports.service\nStartLimitIntervalSec=60\nStartLimitBurst=3\n[Service]\nType=oneshot\nExecStart=/opt/rz/current/bin/rz-admin update recover\nRestart=on-failure\n",
            );
            for (unit, command) in [
                ("rz-admin.service", "/opt/rz/current/bin/rz-admin serve"),
                ("rz-monitor.service", "/opt/rz/current/bin/rz-monitor controller"),
                ("rz-insights.service", "/opt/rz/current/bin/rz-insights serve"),
                ("rz-reports.service", "/opt/rz/current/bin/rz-reports serve"),
            ] {
                append(
                    &mut builder,
                    &format!("{root}/systemd/{unit}"),
                    0o644,
                    format!(
                        "[Unit]\nAfter=network.target rz-recovery.service\nPartOf=rz.target\nStartLimitIntervalSec=60\nStartLimitBurst=3\n[Service]\nExecCondition=/usr/bin/test ! -e /opt/rz/data/recovery-blocked\nExecStart={command}\nRestart=on-failure\n"
                    )
                    .as_bytes(),
                );
            }
            append(
                &mut builder,
                &format!("{root}/config/rz.env"),
                0o600,
                b"RUSTZEN_ENV=production\n",
            );
            append(&mut builder, &format!("{root}/setup-layout.sh"), 0o755, b"#!/bin/sh\n");
            builder.finish().expect("finish tar");
        }
        output
    }

    fn append(builder: &mut tar::Builder<&mut Vec<u8>>, path: &str, mode: u32, data: &[u8]) {
        let mut header = tar::Header::new_gnu();
        header.set_path(path).expect("path");
        header.set_size(data.len() as u64);
        header.set_mode(mode);
        header.set_cksum();
        builder.append(&header, data).expect("append");
    }

    #[test]
    fn validates_and_installs_exact_release_binary_bundle() {
        let version = "1.2.3";
        let data = fixture(version, "x86_64");
        let info = validate_bundle(&data, version, false, None).expect("valid bundle");
        assert_eq!(info.arch, "x86_64");
        let root = std::env::temp_dir().join(format!("rz-bundle-{}", uuid::Uuid::new_v4()));
        let release = install_bundle(&data, &info, version, 9, &root).expect("install");
        for binary in BINARIES {
            assert!(release.join("bin").join(binary).is_file());
        }
        verify_installed_bundle(&data, &info, version, &release).expect("verified install");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let reports = release.join("bin/rz-reports");
            fs::set_permissions(&reports, fs::Permissions::from_mode(0o644)).expect("chmod");
            assert!(verify_installed_bundle(&data, &info, version, &release).is_err());
            fs::set_permissions(&reports, fs::Permissions::from_mode(0o755)).expect("restore mode");
        }
        fs::write(release.join("bin/rz-reports"), b"tampered").expect("tamper install");
        assert!(verify_installed_bundle(&data, &info, version, &release).is_err());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn rejects_missing_duplicate_unsafe_and_mismatched_members() {
        let version = "1.2.3";
        let valid = fixture(version, "x86_64");
        assert!(validate_bundle(&valid, "9.9.9", false, None).is_err());

        let mut truncated = valid.clone();
        truncated.truncate(valid.len() / 2);
        assert!(validate_bundle(&truncated, version, false, None).is_err());

        assert!(validate_path(Path::new("../escape")).is_err());
        assert!(validate_path(Path::new("/absolute")).is_err());
    }

    #[test]
    fn install_rejects_an_existing_release_directory() {
        let version = "1.2.3";
        let data = fixture(version, "aarch64");
        let info = validate_bundle(&data, version, false, None).expect("valid bundle");
        let root = std::env::temp_dir().join(format!("rz-existing-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("releases").join(version)).expect("existing release");
        assert!(
            install_bundle(
                &data,
                &BundleInfo { arch: info.arch, content_len: info.content_len },
                version,
                4,
                &root
            )
            .is_err()
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn verifies_one_signature_over_the_complete_bundle_and_rejects_tampering() {
        let version = "1.2.3";
        let arch = "x86_64";
        let content = fixture(version, arch);
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let content_hash = sha256_hex(&content);
        let signature =
            signing_key.sign(signature_payload(version, arch, &content_hash).as_bytes());
        let marker = serde_json::json!({
            "schemaVersion": 1,
            "component": "bundle",
            "version": version,
            "arch": arch,
            "contentSha256": content_hash,
            "signature": hex::encode(signature.to_bytes()),
        });
        let mut signed = content;
        signed.extend_from_slice(SIGNED_MARKER_BEGIN);
        signed.extend_from_slice(serde_json::to_string(&marker).expect("marker").as_bytes());
        signed.extend_from_slice(SIGNED_MARKER_END);
        let verify_key = hex::encode(signing_key.verifying_key().to_bytes());
        validate_bundle(&signed, version, true, Some(&verify_key)).expect("signed bundle");

        signed[512] ^= 1;
        assert!(validate_bundle(&signed, version, true, Some(&verify_key)).is_err());
    }
}
