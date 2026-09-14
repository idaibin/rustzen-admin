use crate::{
    install::{Manifest, Payload},
    install_admission::{
        PrivateParent, PublishError, hash_id, valid_key_id, validate_root_owned_path,
    },
    install_crypto::{canonical_json, hash, read_regular, verify_signature_bytes},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub(super) struct PinInputs {
    pub(super) manifest: PathBuf,
    pub(super) envelope: PathBuf,
    pub(super) trusted_key: PathBuf,
    pub(super) key_id: String,
    pub(super) endpoint: String,
}

#[derive(Debug, Serialize)]
pub(super) struct Pinned {
    controller_build_id: String,
    controller_composition_id: String,
    protocol_id: String,
    profile: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Profile<'a> {
    version: u8,
    endpoint: &'a str,
    controller_build_id: &'a str,
    controller_composition_id: &'a str,
    agent_build_id: &'a str,
    protocol_id: &'a str,
    key_id: &'a str,
    key_fingerprint: String,
    manifest_sha256: String,
    agent_manifest_sha256: String,
}

pub(super) fn pin(input: &PinInputs) -> Result<Pinned, String> {
    let agent_root = Path::new("/opt/rz");
    let controller = controller_manifest(input)?;
    if controller.manifest.artifact_class != "server"
        || controller.manifest.preset != "monitor"
        || controller.manifest.capabilities != ["access", "monitor"]
        || controller.manifest.services != ["admin", "monitor"]
    {
        return Err("Controller release is not the production Monitor server selection".into());
    }
    let endpoint = rustzen_config::canonical_monitor_endpoint(&input.endpoint)
        .map_err(|_| "Controller endpoint is invalid")?;
    let agent_gid = service_gid()?;
    let (agent, agent_bytes) = agent_manifest(agent_root)?;
    if agent.artifact_class != "node-agent"
        || agent.preset != "node-agent"
        || agent.agent_protocol_contract_id != controller.manifest.agent_protocol_contract_id
    {
        return Err("Agent manifest does not exactly match Controller protocol".into());
    }
    let profile = Profile {
        version: 1,
        endpoint: &endpoint,
        controller_build_id: &controller.manifest.build_id,
        controller_composition_id: &controller.manifest.composition_id,
        agent_build_id: &agent.build_id,
        protocol_id: &controller.manifest.agent_protocol_contract_id,
        key_id: &input.key_id,
        key_fingerprint: hash(&controller.trusted_key),
        manifest_sha256: hash(&controller.manifest_bytes),
        agent_manifest_sha256: hash(&agent_bytes),
    };
    let bytes = canonical_json(&serde_json::to_value(profile).map_err(|_| "profile encoding")?)?;
    preflight_profile(agent_root, &bytes, agent_gid)?;
    validate_agent_traversal(agent_root, agent_gid)?;
    let profile_path = agent_root.join("controller-profile.json");
    write_profile(agent_root, &bytes, agent_gid)?;
    Ok(Pinned {
        controller_build_id: controller.manifest.build_id,
        controller_composition_id: controller.manifest.composition_id,
        protocol_id: controller.manifest.agent_protocol_contract_id,
        profile: profile_path,
    })
}

struct Controller {
    manifest: Manifest,
    manifest_bytes: Vec<u8>,
    trusted_key: Vec<u8>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    payload: Payload,
    signature: String,
}

fn controller_manifest(input: &PinInputs) -> Result<Controller, String> {
    if !valid_key_id(&input.key_id) {
        return Err("trusted key ID is invalid".into());
    }
    let manifest_bytes = read_regular(&input.manifest, 4 * 1024 * 1024)?;
    let envelope_bytes = read_regular(&input.envelope, 64 * 1024)?;
    let trusted_key = read_regular(&input.trusted_key, 64 * 1024)?;
    let manifest = crate::install_manifest::parse_manifest(&manifest_bytes)?;
    let envelope: Envelope =
        serde_json::from_slice(&envelope_bytes).map_err(|_| "envelope is invalid JSON")?;
    if canonical_json(&serde_json::to_value(&envelope).map_err(|_| "envelope encoding")?)?
        != envelope_bytes
    {
        return Err("envelope bytes are not canonical".into());
    }
    let payload = &envelope.payload;
    if payload.domain != "rustzen-selected-release-v1"
        || payload.envelope_version != 1
        || payload.algorithm != "Ed25519"
        || payload.key_id != input.key_id
        || payload.release_class != "production"
        || !hash_id(&payload.archive_sha256)
        || payload.manifest_sha256 != hash(&manifest_bytes)
        || (
            payload.release_version.as_str(),
            payload.target.as_str(),
            payload.artifact_class.as_str(),
            payload.composition_id.as_str(),
            payload.build_id.as_str(),
            payload.agent_protocol_contract_id.as_str(),
        ) != (
            manifest.release_version.as_str(),
            manifest.target.as_str(),
            manifest.artifact_class.as_str(),
            manifest.composition_id.as_str(),
            manifest.build_id.as_str(),
            manifest.agent_protocol_contract_id.as_str(),
        )
    {
        return Err("Controller manifest and envelope tuple differs".into());
    }
    verify_signature_bytes(&trusted_key, payload, &envelope.signature)?;
    Ok(Controller { manifest, manifest_bytes, trusted_key })
}

pub(super) fn agent_manifest(root: &Path) -> Result<(Manifest, Vec<u8>), String> {
    validate_root_owned_path(root)?;
    let meta = fs::symlink_metadata(root).map_err(|_| "Agent root is unavailable")?;
    if meta.file_type().is_symlink()
        || !meta.is_dir()
        || meta.uid() != 0
        || meta.mode() & 0o022 != 0
    {
        return Err("Agent root ownership or mode is invalid".into());
    }
    let current = root.join("current");
    let target = fs::read_link(&current).map_err(|_| "Agent root current link is missing")?;
    let target = target.to_str().ok_or("Agent current link is invalid")?;
    let Some(build) = target.strip_prefix("releases/").and_then(|x| x.strip_suffix("/payload"))
    else {
        return Err("Agent current link is invalid".into());
    };
    if !hash_id(build) || target.contains("..") {
        return Err("Agent current link is invalid".into());
    }
    for path in [root.join("releases"), root.join("releases").join(build), root.join("state")] {
        let meta = fs::symlink_metadata(path).map_err(|_| "Agent release path is unavailable")?;
        if meta.file_type().is_symlink()
            || !meta.is_dir()
            || meta.uid() != 0
            || meta.mode() & 0o022 != 0
        {
            return Err("Agent release path is unsafe".into());
        }
    }
    let marker = read_regular(&root.join("state/publication-marker.json"), 16 * 1024)?;
    let marker: serde_json::Value =
        serde_json::from_slice(&marker).map_err(|_| "Agent publication marker is invalid")?;
    // The publication marker nests the identity tuple inside its journal; the
    // top level carries only version and phase state.
    if marker["version"] != 1
        || marker["state"] != "payload-published"
        || marker["journal"]["buildId"] != build
        || marker["journal"]["artifactClass"] != "node-agent"
    {
        return Err("Agent publication marker differs from current release".into());
    }
    let bytes = read_regular(
        &root.join("releases").join(build).join("release-manifest.json"),
        4 * 1024 * 1024,
    )?;
    let manifest = crate::install_manifest::parse_manifest(&bytes)?;
    if manifest.build_id != build {
        return Err("Agent manifest differs from installed release".into());
    }
    Ok((manifest, bytes))
}

pub(super) fn validate_current_agent_binary(
    root: &Path,
    manifest: &Manifest,
) -> Result<(), String> {
    let target =
        fs::read_link(root.join("current")).map_err(|_| "Agent current link is missing")?;
    let target = target.to_str().ok_or("Agent current link is invalid")?;
    if target != format!("releases/{}/payload", manifest.build_id) {
        return Err("Agent current link differs from retained manifest".into());
    }
    let binary = root.join(target).join("bin/rz-monitor-agent");
    let metadata = fs::symlink_metadata(&binary).map_err(|_| "Agent binary is unavailable")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.uid() != 0
        || metadata.mode() & 0o777 != 0o755
    {
        return Err("Agent binary is unsafe".into());
    }
    let expected = manifest
        .binary_digests
        .iter()
        .find(|entry| entry.path == "bin/rz-monitor-agent")
        .ok_or("Agent binary digest is unavailable")?;
    let bytes = read_regular(&binary, 256 * 1024 * 1024)?;
    if expected.sha256 != hash(&bytes) {
        return Err("Agent binary digest differs from retained manifest".into());
    }
    Ok(())
}

pub(super) fn service_gid() -> Result<u32, String> {
    let mut group = std::mem::MaybeUninit::<libc::group>::uninit();
    let mut buffer = vec![0u8; 16 * 1024];
    let mut found = std::ptr::null_mut();
    let result = unsafe {
        libc::getgrnam_r(
            c"rz-monitor-agent".as_ptr(),
            group.as_mut_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut found,
        )
    };
    if result != 0 || found.is_null() {
        return Err("rz-monitor-agent group does not exist".into());
    }
    let gid = unsafe { (*found).gr_gid };
    if gid == 0 {
        return Err("rz-monitor-agent group must not be root".into());
    }
    let mut passwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
    let mut user_buffer = vec![0u8; 16 * 1024];
    let mut user = std::ptr::null_mut();
    let result = unsafe {
        libc::getpwnam_r(
            c"rz-monitor-agent".as_ptr(),
            passwd.as_mut_ptr(),
            user_buffer.as_mut_ptr().cast(),
            user_buffer.len(),
            &mut user,
        )
    };
    if result != 0
        || user.is_null()
        || unsafe { (*user).pw_uid } == 0
        || unsafe { (*user).pw_gid } != gid
    {
        return Err("rz-monitor-agent user/group identity is invalid".into());
    }
    Ok(gid)
}

pub(super) fn prepare_access() -> Result<(), String> {
    let root = Path::new("/opt/rz");
    let (manifest, _) = agent_manifest(root)?;
    let build = manifest.build_id;
    let gid = service_gid()?;
    configure_agent_group_access(root, &build, gid)?;
    validate_agent_traversal(root, gid)
}

fn configure_agent_group_access(root: &Path, build: &str, gid: u32) -> Result<(), String> {
    let payload = root.join("releases").join(build).join("payload");
    for directory in [
        root.to_path_buf(),
        root.join("releases"),
        root.join("releases").join(build),
        payload.clone(),
        payload.join("bin"),
    ] {
        let meta =
            fs::symlink_metadata(&directory).map_err(|_| "Agent payload path is unavailable")?;
        if meta.file_type().is_symlink()
            || !meta.is_dir()
            || meta.uid() != 0
            || meta.mode() & 0o022 != 0
        {
            return Err("Agent payload path is unsafe".into());
        }
        chown_mode(&directory, gid, 0o750)?;
    }
    let binary = payload.join("bin/rz-monitor-agent");
    let meta = fs::symlink_metadata(&binary).map_err(|_| "Agent binary is unavailable")?;
    if meta.file_type().is_symlink()
        || !meta.is_file()
        || meta.uid() != 0
        || meta.mode() & 0o777 != 0o755
    {
        return Err("signed Agent binary mode differs from manifest".into());
    }
    let manifest = root.join("releases").join(build).join("release-manifest.json");
    let meta = fs::symlink_metadata(&manifest).map_err(|_| "Agent manifest is unavailable")?;
    if meta.file_type().is_symlink()
        || !meta.is_file()
        || meta.uid() != 0
        || meta.mode() & 0o022 != 0
    {
        return Err("Agent manifest is unsafe".into());
    }
    chown_mode(&manifest, gid, 0o640)
}

/// Verify the service account can traverse every fixed component before its
/// profile is written. The pairing command never repairs an ancestor it does
/// not own.
pub(super) fn validate_agent_traversal(root: &Path, gid: u32) -> Result<(), String> {
    let mut current = PathBuf::from("/");
    for component in root.components() {
        let std::path::Component::Normal(component) = component else {
            continue;
        };
        current.push(component);
        let metadata =
            fs::symlink_metadata(&current).map_err(|_| "Agent profile ancestor is unavailable")?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err("Agent profile ancestor is unsafe".into());
        }
        let mode = metadata.mode();
        let executable = if metadata.gid() == gid { mode & 0o010 != 0 } else { mode & 0o001 != 0 };
        if !executable {
            return Err("Agent service group cannot traverse the profile path".into());
        }
    }
    Ok(())
}

fn chown_mode(path: &Path, gid: u32, mode: u32) -> Result<(), String> {
    let path =
        std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(|_| "Agent path is invalid")?;
    if unsafe { libc::chown(path.as_ptr(), 0, gid) } != 0
        || unsafe { libc::chmod(path.as_ptr(), mode as libc::mode_t) } != 0
    {
        return Err("Agent group access could not be configured".into());
    }
    Ok(())
}

fn write_profile(root: &Path, bytes: &[u8], gid: u32) -> Result<(), String> {
    let path = root.join("controller-profile.json");
    let parent = PrivateParent::open(root)?;
    if let Ok(existing) = fs::symlink_metadata(&path) {
        if existing.file_type().is_symlink()
            || !existing.is_file()
            || existing.uid() != 0
            || existing.gid() != gid
            || existing.mode() & 0o777 != 0o640
        {
            return Err("profile destination is unsafe".into());
        }
        return if read_regular(&path, 16 * 1024)? == bytes {
            parent.sync()
        } else {
            Err("profile differs from requested tuple".into())
        };
    }
    match parent.publish_regular_noreplace("controller-profile.json", bytes, 0, gid, 0o640) {
        Ok(()) => Ok(()),
        Err(PublishError::Conflict)
            if path.exists() && read_regular(&path, 16 * 1024)? == bytes =>
        {
            parent.sync()
        }
        Err(error) => Err(error.to_string()),
    }
}

fn preflight_profile(root: &Path, bytes: &[u8], gid: u32) -> Result<(), String> {
    let path = root.join("controller-profile.json");
    if let Ok(existing) = fs::symlink_metadata(&path) {
        if existing.file_type().is_symlink()
            || !existing.is_file()
            || existing.uid() != 0
            || existing.gid() != gid
            || existing.mode() & 0o777 != 0o640
        {
            return Err("profile destination is unsafe".into());
        }
        if read_regular(&path, 16 * 1024)? != bytes {
            return Err("profile differs from requested tuple".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn agent_marker_identity_lives_in_the_journal_tuple() {
        use crate::install_continuation::marker_shape_test_support::published_marker_bytes;

        let build = "e".repeat(64);
        let bytes = published_marker_bytes(&build, "node-agent");
        let marker: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(marker["version"], json!(1));
        assert_eq!(marker["state"], json!("payload-published"));
        assert_eq!(marker["journal"]["buildId"], json!(build));
        assert_eq!(marker["journal"]["artifactClass"], json!("node-agent"));
        assert!(marker.get("buildId").is_none());
        assert!(marker.get("artifactClass").is_none());
    }

    #[test]
    fn endpoint_rejects_credentials_queries_and_fragments() {
        assert_eq!(
            rustzen_config::canonical_monitor_endpoint(" https://monitor.example/base/ ").unwrap(),
            "https://monitor.example/base"
        );
        for invalid in [
            "ftp://monitor.example",
            "https://u:p@monitor.example",
            "https://monitor.example/?q=1",
            "https://monitor.example/#x",
        ] {
            assert!(rustzen_config::canonical_monitor_endpoint(invalid).is_err());
        }
    }
}
