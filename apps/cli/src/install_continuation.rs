use crate::{
    install::Loaded,
    install_admission::PrivateParent,
    install_archive::extract,
    install_crypto::{hash, io},
    install_fs::{mkdir_private, write_relative, write_secure},
};
use serde::{Deserialize, Serialize};
use std::{os::unix::fs::MetadataExt, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ContinuationJournal {
    archive_sha256: String,
    artifact_class: String,
    build_id: String,
    composition_id: String,
    envelope_sha256: String,
    key_id: String,
    manifest_sha256: String,
    phase: String,
    target: String,
    trusted_key_sha256: String,
    version: u8,
    pub(super) work_root_nonce: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PublicationMarker {
    journal: ContinuationJournal,
    state: String,
    version: u8,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkRootMarker {
    nonce: String,
    version: u8,
}

impl ContinuationJournal {
    pub(super) fn from_loaded(loaded: &Loaded, work_root_nonce: String) -> Self {
        Self {
            archive_sha256: hash(&loaded.archive),
            artifact_class: loaded.verified.artifact_class.clone(),
            build_id: loaded.verified.build_id.clone(),
            composition_id: loaded.manifest.composition_id.clone(),
            envelope_sha256: hash(&loaded.envelope_bytes),
            key_id: loaded.key_id.clone(),
            manifest_sha256: hash(&loaded.manifest_bytes),
            phase: "payload-publishing".into(),
            target: loaded.verified.target.clone(),
            trusted_key_sha256: hash(&loaded.trusted_key),
            version: 1,
            work_root_nonce,
        }
    }

    fn matches_loaded(&self, loaded: &Loaded) -> bool {
        let expected = Self::from_loaded(loaded, self.work_root_nonce.clone());
        self == &expected && valid_nonce(&self.work_root_nonce)
    }
}

pub(super) fn validate_continuation(
    parent: &PrivateParent,
    name: &str,
    loaded: &Loaded,
) -> Result<ContinuationJournal, String> {
    let bytes = parent.read_regular_owned(name, 8 * 1024, 0o600)?;
    let actual: ContinuationJournal =
        serde_json::from_slice(&bytes).map_err(|_| "fresh-root journal is invalid")?;
    if serde_json::to_vec(&actual).map_err(|_| "fresh-root journal encoding failed")? != bytes {
        return Err("fresh-root journal is not canonical".into());
    }
    if !actual.matches_loaded(loaded) {
        return Err("fresh-root journal differs from requested tuple".into());
    }
    Ok(actual)
}

pub(super) fn validate_work_root(
    parent: &PrivateParent,
    name: &str,
    nonce: &str,
) -> Result<(), String> {
    let root = parent.open_child_directory(name)?;
    let metadata = root.metadata()?;
    if metadata.uid() != 0 || metadata.gid() != 0 || metadata.mode() & 0o777 != 0o700 {
        return Err("fresh-root work root is unsafe".into());
    }
    let state =
        root.open_child_directory("state").map_err(|_| "fresh-root work marker state is unsafe")?;
    let metadata = state.metadata()?;
    if metadata.uid() != 0
        || metadata.gid() != 0
        || !matches!(metadata.mode() & 0o777, 0o700 | 0o750)
    {
        return Err("fresh-root work marker state is unsafe".into());
    }
    let bytes = state.read_regular_owned("continuation.json", 8 * 1024, 0o600)?;
    let marker: WorkRootMarker =
        serde_json::from_slice(&bytes).map_err(|_| "fresh-root work marker is invalid")?;
    if marker.version != 1 || marker.nonce != nonce || !valid_nonce(&marker.nonce) {
        return Err("fresh-root work marker differs from journal".into());
    }
    Ok(())
}

pub(super) fn validate_published_destination(
    parent: &PrivateParent,
    name: &str,
    expected: &ContinuationJournal,
    loaded: &Loaded,
) -> Result<(), String> {
    let marker = read_publication_marker(parent, name)?;
    if marker.version != 1 || marker.state != "payload-published" || marker.journal != *expected {
        return Err("fresh-root destination marker differs from requested tuple".into());
    }
    crate::install_terminal::validate_terminal_layout(parent, name, loaded, true)?;
    Ok(())
}

pub(super) fn validate_completed_destination(
    parent: &PrivateParent,
    name: &str,
    loaded: &Loaded,
) -> Result<(), String> {
    let marker = read_publication_marker(parent, name)?;
    if marker.version != 1
        || marker.state != "payload-published"
        || !marker.journal.matches_loaded(loaded)
    {
        return Err("destination must be a fresh absent root".into());
    }
    crate::install_terminal::validate_terminal_layout(parent, name, loaded, false)?;
    Ok(())
}

fn read_publication_marker(
    parent: &PrivateParent,
    name: &str,
) -> Result<PublicationMarker, String> {
    let root = parent.open_child_directory(name)?;
    let metadata = root.metadata()?;
    if metadata.uid() != 0 || metadata.gid() != 0 || metadata.mode() & 0o777 != 0o700 {
        return Err("fresh-root destination is unsafe".into());
    }
    let state =
        root.open_child_directory("state").map_err(|_| "fresh-root destination state is unsafe")?;
    let metadata = state.metadata()?;
    if metadata.uid() != 0 || metadata.gid() != 0 || metadata.mode() & 0o777 != 0o750 {
        return Err("fresh-root destination state is unsafe".into());
    }
    let bytes = state.read_regular_owned("publication-marker.json", 8 * 1024, 0o640)?;
    let marker: PublicationMarker =
        serde_json::from_slice(&bytes).map_err(|_| "fresh-root destination marker is invalid")?;
    Ok(marker)
}
pub(super) fn fault_after_publish() -> Result<(), String> {
    #[cfg(debug_assertions)]
    if std::env::var_os("RUSTZEN_INSTALLER_FAULT_AFTER_PUBLISH").is_some() {
        return Err("debug installer fault after payload publication".into());
    }
    Ok(())
}
pub(super) fn fault_after_rename() -> Result<(), String> {
    #[cfg(debug_assertions)]
    if std::env::var_os("RUSTZEN_INSTALLER_FAULT_AFTER_RENAME").is_some() {
        return Err("debug installer fault after payload rename".into());
    }
    Ok(())
}

pub(super) fn fault_after_work_marker_cleanup() -> Result<(), String> {
    #[cfg(debug_assertions)]
    if std::env::var_os("RUSTZEN_INSTALLER_FAULT_AFTER_WORK_MARKER_CLEANUP").is_some() {
        return Err("debug installer fault after work marker cleanup".into());
    }
    Ok(())
}

pub(super) fn fault_after_journal_cleanup() -> Result<(), String> {
    #[cfg(debug_assertions)]
    if std::env::var_os("RUSTZEN_INSTALLER_FAULT_AFTER_JOURNAL_CLEANUP").is_some() {
        return Err("debug installer fault after journal cleanup".into());
    }
    Ok(())
}

pub(super) fn write_work_marker(
    parent: &PrivateParent,
    name: &str,
    nonce: &str,
) -> Result<(), String> {
    let root = parent.open_child_directory(name)?;
    root.create_temp("state")?;
    let state = root.open_child_directory("state")?;
    let bytes = serde_json::to_vec(&WorkRootMarker { nonce: nonce.into(), version: 1 })
        .map_err(|_| "fresh-root work marker encoding failed")?;
    state
        .publish_regular_noreplace("continuation.json", &bytes, 0, 0, 0o600)
        .map_err(|error| error.to_string())?;
    state.sync()?;
    root.sync()
}

pub(super) fn clear_work_marker_if_present(
    parent: &PrivateParent,
    name: &str,
) -> Result<(), String> {
    let root = parent.open_child_directory(name)?;
    let state = root.open_child_directory("state")?;
    if state.exists("continuation.json")? {
        state.remove_regular_owned("continuation.json", 0o600)?;
    }
    root.sync()
}

pub(super) fn random_nonce() -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        let mut bytes = [0u8; 16];
        if unsafe { libc::getrandom(bytes.as_mut_ptr().cast(), bytes.len(), 0) }
            != bytes.len() as isize
        {
            return Err("fresh-root nonce generation failed".into());
        }
        Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
    }
    #[cfg(not(target_os = "linux"))]
    Err("apply supports Linux only".into())
}

fn valid_nonce(value: &str) -> bool {
    value.len() == 32
        && value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

pub(super) fn publish(
    input: &Loaded,
    root: &Path,
    journal: &ContinuationJournal,
) -> Result<(), String> {
    for path in ["releases", "state", "trust"] {
        mkdir_private(root.join(path), 0o750)?;
    }
    let release = root.join("releases").join(&input.verified.build_id);
    mkdir_private(&release, 0o750)?;
    extract(&input.archive, &input.manifest, &release.join("payload"))?;
    write_secure(&release.join("release-manifest.json"), &input.manifest_bytes)?;
    write_secure(&release.join("signature-envelope.json"), &input.envelope_bytes)?;
    write_relative(
        &root.join("trust"),
        &format!("{}.pem", input.key_id),
        &input.trusted_key,
        0o640,
    )?;
    let marker = PublicationMarker {
        journal: journal.clone(),
        state: "payload-published".into(),
        version: 1,
    };
    write_secure(
        &root.join("state/publication-marker.json"),
        &serde_json::to_vec(&marker).map_err(|_| "marker encoding failed")?,
    )?;
    std::os::unix::fs::symlink(
        format!("releases/{}/payload", input.verified.build_id),
        root.join("current"),
    )
    .map_err(io)?;
    Ok(())
}
