use crate::{
    install::{Entry, Loaded},
    install_admission::PrivateParent,
    install_crypto::{hash, read_regular},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::fs::MetadataExt,
    path::Path,
};

pub(super) fn validate_terminal_layout(
    parent: &PrivateParent,
    name: &str,
    loaded: &Loaded,
    allow_work_marker: bool,
) -> Result<(), String> {
    let root = parent.child_path(name)?;
    require_directory(&root, 0o700)?;
    exact_children(&root, &["current", "releases", "state", "trust"])?;
    let current =
        fs::read_link(root.join("current")).map_err(|_| "fresh-root current link is invalid")?;
    if current != format!("releases/{}/payload", loaded.verified.build_id) {
        return Err("fresh-root current link differs from retained manifest".into());
    }
    let releases = root.join("releases");
    require_directory(&releases, 0o750)?;
    exact_children(&releases, &[&loaded.verified.build_id])?;
    let release = releases.join(&loaded.verified.build_id);
    require_directory(&release, 0o750)?;
    exact_children(&release, &["payload", "release-manifest.json", "signature-envelope.json"])?;
    verify_file(&release.join("release-manifest.json"), &loaded.manifest_bytes, 0o640)?;
    verify_file(&release.join("signature-envelope.json"), &loaded.envelope_bytes, 0o640)?;
    validate_payload(&release.join("payload"), &loaded.manifest.files)?;
    let trust = root.join("trust");
    require_directory(&trust, 0o750)?;
    let key_name = format!("{}.pem", loaded.key_id);
    exact_children(&trust, &[&key_name])?;
    verify_file(&trust.join(key_name), &loaded.trusted_key, 0o640)?;
    let state = root.join("state");
    require_directory(&state, 0o750)?;
    let mut expected = vec!["publication-marker.json"];
    let continuation = state.join("continuation.json");
    if allow_work_marker && continuation.exists() {
        verify_mode(&continuation, 0o600)?;
        expected.push("continuation.json");
    }
    exact_children(&state, &expected)?;
    Ok(())
}

fn validate_payload(root: &Path, entries: &[Entry]) -> Result<(), String> {
    require_directory(root, 0o750)?;
    let expected =
        entries.iter().map(|entry| (entry.path.as_str(), entry)).collect::<BTreeMap<_, _>>();
    let mut directories = BTreeSet::from([String::new()]);
    for entry in entries {
        let mut parent = Path::new(&entry.path).parent();
        while let Some(path) = parent {
            if path.as_os_str().is_empty() {
                break;
            }
            directories.insert(path.to_string_lossy().into_owned());
            parent = path.parent();
        }
    }
    let mut seen = BTreeSet::new();
    validate_payload_directory(root, "", &expected, &directories, &mut seen)?;
    if seen.len() != expected.len()
        || seen.into_iter().any(|path| !expected.contains_key(path.as_str()))
    {
        return Err("fresh-root payload differs from manifest".into());
    }
    Ok(())
}

fn validate_payload_directory(
    root: &Path,
    relative: &str,
    files: &BTreeMap<&str, &Entry>,
    directories: &BTreeSet<String>,
    seen: &mut BTreeSet<String>,
) -> Result<(), String> {
    let directory = if relative.is_empty() { root.to_path_buf() } else { root.join(relative) };
    require_directory(&directory, 0o750)?;
    for item in
        fs::read_dir(&directory).map_err(|_| "fresh-root payload directory is unavailable")?
    {
        let item = item.map_err(|_| "fresh-root payload directory is unavailable")?;
        let name = item.file_name().to_string_lossy().into_owned();
        let child = if relative.is_empty() { name.clone() } else { format!("{relative}/{name}") };
        let metadata = fs::symlink_metadata(item.path())
            .map_err(|_| "fresh-root payload entry is unavailable")?;
        if metadata.file_type().is_symlink() {
            return Err("fresh-root payload contains a link".into());
        }
        if metadata.is_dir() {
            if !directories.contains(&child) {
                return Err("fresh-root payload contains an extra directory".into());
            }
            validate_payload_directory(root, &child, files, directories, seen)?;
        } else {
            let entry =
                files.get(child.as_str()).ok_or("fresh-root payload contains an extra file")?;
            if !metadata.is_file()
                || metadata.uid() != 0
                || metadata.gid() != 0
                || metadata.mode() & 0o777 != entry_mode(entry)
            {
                return Err("fresh-root payload file differs from manifest".into());
            }
            let bytes = read_regular(&item.path(), entry.size)?;
            if bytes.len() as u64 != entry.size
                || hash(&bytes) != entry.sha256
                || !seen.insert(child)
            {
                return Err("fresh-root payload file differs from manifest".into());
            }
        }
    }
    Ok(())
}

fn entry_mode(entry: &Entry) -> u32 {
    if entry.mode == "0755" { 0o755 } else { 0o644 }
}

fn exact_children(path: &Path, expected: &[&str]) -> Result<(), String> {
    let actual = fs::read_dir(path)
        .map_err(|_| "fresh-root terminal layout is unavailable")?
        .map(|item| item.map(|item| item.file_name().to_string_lossy().into_owned()))
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(|_| "fresh-root terminal layout is unavailable")?;
    let expected = expected.iter().map(|name| (*name).to_owned()).collect::<BTreeSet<_>>();
    if actual != expected {
        return Err("fresh-root terminal layout differs from manifest".into());
    }
    Ok(())
}

fn require_directory(path: &Path, mode: u32) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "fresh-root terminal directory is unavailable")?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != 0
        || metadata.gid() != 0
        || metadata.mode() & 0o777 != mode
    {
        return Err("fresh-root terminal directory is unsafe".into());
    }
    Ok(())
}

fn verify_mode(path: &Path, mode: u32) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "fresh-root terminal file is unavailable")?;
    if !metadata.is_file()
        || metadata.uid() != 0
        || metadata.gid() != 0
        || metadata.mode() & 0o777 != mode
    {
        return Err("fresh-root terminal file is unsafe".into());
    }
    Ok(())
}

fn verify_file(path: &Path, expected: &[u8], mode: u32) -> Result<(), String> {
    verify_mode(path, mode)?;
    if read_regular(path, expected.len() as u64)? != expected {
        return Err("fresh-root retained file differs from requested tuple".into());
    }
    Ok(())
}
