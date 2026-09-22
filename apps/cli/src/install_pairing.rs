use crate::{
    install::Manifest,
    install_admission::{
        PrivateParent, PublishError, hash_id, valid_key_id, validate_root_owned_path,
    },
    install_crypto::{canonical_json, hash, read_regular},
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey, pkcs8::DecodePublicKey};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read},
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::{Path, PathBuf},
};

const SIGNED_MARKER_BEGIN: &[u8] = b"\nRUSTZEN_BUNDLE_SIGNED_MARKER_BEGIN\n";
const SIGNED_MARKER_END: &[u8] = b"\nRUSTZEN_BUNDLE_SIGNED_MARKER_END\n";
const SIGNATURE_PAYLOAD_VERSION: &str = "rustzen-release-v2";
const BINARIES: [&str; 5] = ["rz", "rz-admin", "rz-monitor", "rz-insights", "rz-reports"];

#[derive(Debug, Clone)]
pub(super) struct PinInputs {
    pub(super) bundle: PathBuf,
    pub(super) endpoint: String,
}

#[derive(Debug, Serialize)]
pub(super) struct Pinned {
    controller_version: String,
    controller_arch: String,
    controller_content_sha256: String,
    protocol_id: String,
    profile: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Profile<'a> {
    version: u8,
    endpoint: &'a str,
    controller_version: &'a str,
    controller_arch: &'a str,
    controller_content_sha256: &'a str,
    controller_frontend_sha256: &'a str,
    controller_backend_sha256: &'a str,
    controller_monitor_sha256: &'a str,
    agent_build_id: &'a str,
    protocol_id: &'a str,
    key_fingerprint: String,
    agent_manifest_sha256: String,
}

pub(super) fn pin(input: &PinInputs) -> Result<Pinned, String> {
    let agent_root = Path::new("/opt/rz");
    let (agent, agent_bytes) = agent_manifest(agent_root)?;
    let controller = controller_bundle(input, agent_root)?;
    let endpoint = rustzen_config::canonical_monitor_endpoint(&input.endpoint)
        .map_err(|_| "Controller endpoint is invalid")?;
    let agent_gid = service_gid()?;
    if agent.artifact_class != "node-agent"
        || agent.preset != "node-agent"
        || agent.agent_protocol_contract_id != controller.identity.agent_protocol_contract_id
    {
        return Err("Agent manifest does not exactly match full Controller protocol".into());
    }
    let profile = Profile {
        version: 2,
        endpoint: &endpoint,
        controller_version: &controller.marker.version,
        controller_arch: &controller.marker.arch,
        controller_content_sha256: &controller.marker.content_sha256,
        controller_frontend_sha256: &controller.marker.frontend_sha256,
        controller_backend_sha256: &controller.marker.backend_sha256,
        controller_monitor_sha256: &controller.identity.monitor_binary_sha256,
        agent_build_id: &agent.build_id,
        protocol_id: &controller.identity.agent_protocol_contract_id,
        key_fingerprint: hash(&controller.trusted_key),
        agent_manifest_sha256: hash(&agent_bytes),
    };
    let bytes = canonical_json(&serde_json::to_value(profile).map_err(|_| "profile encoding")?)?;
    preflight_profile(agent_root, &bytes, agent_gid)?;
    validate_agent_traversal(agent_root, agent_gid)?;
    let profile_path = agent_root.join("controller-profile.json");
    write_profile(agent_root, &bytes, agent_gid)?;
    Ok(Pinned {
        controller_version: controller.marker.version,
        controller_arch: controller.marker.arch,
        controller_content_sha256: controller.marker.content_sha256,
        protocol_id: controller.identity.agent_protocol_contract_id,
        profile: profile_path,
    })
}

struct Controller {
    marker: BundleSignatureMarker,
    identity: ControllerIdentity,
    trusted_key: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BundleSignatureMarker {
    schema_version: u8,
    component: String,
    version: String,
    arch: String,
    content_sha256: String,
    frontend_sha256: String,
    backend_sha256: String,
    signature: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ControllerIdentity {
    schema_version: u8,
    artifact_class: String,
    version: String,
    arch: String,
    monitor_binary_sha256: String,
    agent_protocol_contract_id: String,
}

fn controller_bundle(input: &PinInputs, agent_root: &Path) -> Result<Controller, String> {
    let bundle = read_regular(&input.bundle, 1024 * 1024 * 1024)?;
    verify_controller_bundle(&bundle, trusted_verify_key(agent_root)?)
}

fn verify_controller_bundle(bundle: &[u8], trusted_key: [u8; 32]) -> Result<Controller, String> {
    let begin = find_last(bundle, SIGNED_MARKER_BEGIN)
        .ok_or("full Controller bundle signature is missing")?;
    let marker_start = begin + SIGNED_MARKER_BEGIN.len();
    let marker_end = bundle[marker_start..]
        .windows(SIGNED_MARKER_END.len())
        .position(|window| window == SIGNED_MARKER_END)
        .map(|offset| marker_start + offset)
        .ok_or("full Controller bundle signature is invalid")?;
    if marker_end + SIGNED_MARKER_END.len() != bundle.len() {
        return Err("full Controller bundle signature must terminate the artifact".into());
    }
    let marker: BundleSignatureMarker =
        serde_json::from_slice(&bundle[marker_start..marker_end])
            .map_err(|_| "full Controller signature metadata is invalid")?;
    let content = &bundle[..begin];
    if marker.schema_version != 2
        || marker.component != "release"
        || !valid_release_version(&marker.version)
        || !matches!(marker.arch.as_str(), "x86_64" | "aarch64")
        || marker.content_sha256 != hash(content)
        || !hash_id(&marker.frontend_sha256)
        || !hash_id(&marker.backend_sha256)
    {
        return Err("full Controller signature metadata does not match the artifact".into());
    }
    verify_v2_signature(&marker, &trusted_key)?;
    let files = read_controller_files(content, &marker)?;
    let root = format!("rz-{}-{}", marker.version, marker.arch);
    let monitor = files
        .get(&format!("{root}/bin/rz-monitor"))
        .ok_or("full Controller bundle is missing rz-monitor")?;
    let admin = files
        .get(&format!("{root}/bin/rz-admin"))
        .ok_or("full Controller bundle is missing rz-admin")?;
    let identity_bytes = files
        .get(&format!("{root}/identity/controller.json"))
        .ok_or("full Controller bundle is missing Controller identity")?;
    let identity: ControllerIdentity = serde_json::from_slice(identity_bytes)
        .map_err(|_| "full Controller identity is invalid")?;
    for binary in BINARIES {
        validate_release_binary(
            files
                .get(&format!("{root}/bin/{binary}"))
                .ok_or("full Controller bundle identity members are incomplete")?,
            binary,
            &marker.version,
            &marker.arch,
        )?;
    }
    if identity.schema_version != 1
        || identity.artifact_class != "full-controller"
        || identity.version != marker.version
        || identity.arch != marker.arch
        || identity.monitor_binary_sha256 != hash(monitor)
        || !hash_id(&identity.agent_protocol_contract_id)
        || frontend_digest(admin, &marker.version)? != marker.frontend_sha256
        || backend_digest(&files, &root)? != marker.backend_sha256
    {
        return Err("full Controller identity does not match the signed release".into());
    }
    Ok(Controller { marker, identity, trusted_key: trusted_key.to_vec() })
}

fn validate_release_binary(
    bytes: &[u8],
    binary: &str,
    version: &str,
    arch: &str,
) -> Result<(), String> {
    let machine = bytes.get(18..20).map(|value| u16::from_le_bytes([value[0], value[1]]));
    if bytes.get(..4) != Some(b"\x7fELF")
        || machine != Some(if arch == "x86_64" { 62 } else { 183 })
    {
        return Err("full Controller bundle contains an invalid release binary".into());
    }
    let marker = format!(
        "RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary={binary}\nversion={version}\n"
    );
    if bytes.windows(marker.len()).filter(|value| *value == marker.as_bytes()).count() != 1 {
        return Err("full Controller bundle binary identity is invalid".into());
    }
    Ok(())
}

fn trusted_verify_key(agent_root: &Path) -> Result<[u8; 32], String> {
    let marker = read_regular(&agent_root.join("state/publication-marker.json"), 16 * 1024)?;
    let marker: serde_json::Value =
        serde_json::from_slice(&marker).map_err(|_| "Agent publication marker is invalid")?;
    let key_id = marker["journal"]["keyId"]
        .as_str()
        .filter(|value| valid_key_id(value))
        .ok_or("Agent trusted key identity is invalid")?;
    let path = agent_root.join("trust").join(format!("{key_id}.pem"));
    let metadata = fs::symlink_metadata(&path).map_err(|_| "Agent trusted key is unavailable")?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.uid() != 0
        || metadata.mode() & 0o022 != 0
    {
        return Err("Agent trusted key is unsafe".into());
    }
    let pem = read_regular(&path, 64 * 1024)?;
    let pem = String::from_utf8(pem).map_err(|_| "Agent trusted key is invalid")?;
    VerifyingKey::from_public_key_pem(&pem)
        .map(|key| key.to_bytes())
        .map_err(|_| "Agent trusted key is invalid".into())
}

fn verify_v2_signature(marker: &BundleSignatureMarker, key: &[u8; 32]) -> Result<(), String> {
    let signature = hex::decode(&marker.signature)
        .ok()
        .and_then(|bytes| Signature::from_slice(&bytes).ok())
        .ok_or("full Controller signature is invalid")?;
    let payload = format!(
        "{SIGNATURE_PAYLOAD_VERSION}\ncomponent=release\nversion={}\narch={}\ncontent_sha256={}\nfrontend_sha256={}\nbackend_sha256={}\n",
        marker.version,
        marker.arch,
        marker.content_sha256,
        marker.frontend_sha256,
        marker.backend_sha256,
    );
    VerifyingKey::from_bytes(key)
        .map_err(|_| "full Controller verify key is invalid")?
        .verify(payload.as_bytes(), &signature)
        .map_err(|_| "full Controller signature verification failed".into())
}

fn read_controller_files(
    content: &[u8],
    marker: &BundleSignatureMarker,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let root = format!("rz-{}-{}", marker.version, marker.arch);
    let expected = BINARIES
        .iter()
        .map(|name| format!("{root}/bin/{name}"))
        .chain([format!("{root}/identity/controller.json")])
        .collect::<BTreeSet<_>>();
    let mut found = BTreeMap::new();
    let mut archive = tar::Archive::new(Cursor::new(content));
    for entry in archive.entries().map_err(|_| "full Controller bundle is not a valid tar")? {
        let mut entry = entry.map_err(|_| "full Controller bundle contains an invalid entry")?;
        let path = entry
            .path()
            .map_err(|_| "full Controller bundle contains an invalid path")?
            .into_owned();
        if path.is_absolute()
            || path
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err("full Controller bundle contains an unsafe path".into());
        }
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = path.to_str().ok_or("full Controller bundle path is not UTF-8")?.to_string();
        if !expected.contains(&path) {
            continue;
        }
        if found.contains_key(&path) {
            return Err("full Controller bundle contains a duplicate identity member".into());
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(|_| "full Controller bundle member is unreadable")?;
        found.insert(path, bytes);
    }
    if found.len() != expected.len() {
        return Err("full Controller bundle identity members are incomplete".into());
    }
    Ok(found)
}

fn frontend_digest(admin: &[u8], version: &str) -> Result<String, String> {
    let prefix = format!(
        "RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary=rz-admin\nversion={version}\nfrontend_sha256="
    );
    let starts = admin
        .windows(prefix.len())
        .enumerate()
        .filter_map(|(index, bytes)| (bytes == prefix.as_bytes()).then_some(index))
        .collect::<Vec<_>>();
    if starts.len() != 1 {
        return Err("full Controller Admin frontend identity is missing or duplicated".into());
    }
    let start = starts[0] + prefix.len();
    let value =
        admin.get(start..start + 64).ok_or("full Controller Admin frontend identity is invalid")?;
    if admin.get(start + 64) != Some(&b'\n')
        || !value.iter().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err("full Controller Admin frontend identity is invalid".into());
    }
    String::from_utf8(value.to_vec())
        .map_err(|_| "full Controller Admin frontend identity is invalid".into())
}

fn backend_digest(files: &BTreeMap<String, Vec<u8>>, root: &str) -> Result<String, String> {
    let mut inventory = String::new();
    for binary in BINARIES {
        let bytes = files
            .get(&format!("{root}/bin/{binary}"))
            .ok_or("full Controller backend identity is incomplete")?;
        inventory.push_str(&format!(
            "binary={binary}\nsize={}\nsha256={}\n",
            bytes.len(),
            hash(bytes)
        ));
    }
    Ok(hash(inventory.as_bytes()))
}

fn valid_release_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn find_last(data: &[u8], needle: &[u8]) -> Option<usize> {
    data.windows(needle.len()).rposition(|window| window == needle)
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
    // getgrnam_r initialized the caller-owned value when it returned success and a non-null result.
    let group = unsafe { group.assume_init() };
    let gid = group.gr_gid;
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
    if result != 0 || user.is_null() {
        return Err("rz-monitor-agent user/group identity is invalid".into());
    }
    // getpwnam_r initialized the caller-owned value when it returned success and a non-null result.
    let passwd = unsafe { passwd.assume_init() };
    if passwd.pw_uid == 0 || passwd.pw_gid != gid {
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
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::json;

    fn full_bundle() -> (Vec<u8>, [u8; 32]) {
        let version = "0.5.0";
        let arch = "x86_64";
        let root = format!("rz-{version}-{arch}");
        let frontend = "a".repeat(64);
        let protocol = "b".repeat(64);
        let mut files = BTreeMap::new();
        for binary in BINARIES {
            let mut bytes = vec![0_u8; 512];
            bytes[..4].copy_from_slice(b"\x7fELF");
            bytes[18..20].copy_from_slice(&62_u16.to_le_bytes());
            let marker = format!(
                "RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary={binary}\nversion={version}\n{}",
                if binary == "rz-admin" {
                    format!("frontend_sha256={frontend}\n")
                } else {
                    String::new()
                }
            );
            bytes[64..64 + marker.len()].copy_from_slice(marker.as_bytes());
            files.insert(format!("{root}/bin/{binary}"), bytes);
        }
        let monitor_hash = hash(&files[&format!("{root}/bin/rz-monitor")]);
        let backend = backend_digest(&files, &root).unwrap();
        files.insert(
            format!("{root}/identity/controller.json"),
            format!(
                "{{\"schemaVersion\":1,\"artifactClass\":\"full-controller\",\"version\":\"{version}\",\"arch\":\"{arch}\",\"monitorBinarySha256\":\"{monitor_hash}\",\"agentProtocolContractId\":\"{protocol}\"}}\n"
            )
            .into_bytes(),
        );
        let mut content = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut content);
            for (path, bytes) in &files {
                let mut header = tar::Header::new_gnu();
                header.set_path(path).unwrap();
                header.set_size(bytes.len() as u64);
                header.set_mode(if path.contains("/bin/") { 0o755 } else { 0o644 });
                header.set_cksum();
                builder.append(&header, bytes.as_slice()).unwrap();
            }
            builder.finish().unwrap();
        }
        let content_hash = hash(&content);
        let signing = SigningKey::from_bytes(&[7_u8; 32]);
        let payload = format!(
            "{SIGNATURE_PAYLOAD_VERSION}\ncomponent=release\nversion={version}\narch={arch}\ncontent_sha256={content_hash}\nfrontend_sha256={frontend}\nbackend_sha256={backend}\n"
        );
        let marker = BundleSignatureMarker {
            schema_version: 2,
            component: "release".into(),
            version: version.into(),
            arch: arch.into(),
            content_sha256: content_hash,
            frontend_sha256: frontend,
            backend_sha256: backend,
            signature: hex::encode(signing.sign(payload.as_bytes()).to_bytes()),
        };
        content.extend_from_slice(SIGNED_MARKER_BEGIN);
        content.extend_from_slice(&serde_json::to_vec(&marker).unwrap());
        content.extend_from_slice(SIGNED_MARKER_END);
        (content, signing.verifying_key().to_bytes())
    }

    #[test]
    fn full_v2_controller_identity_is_verified_and_v1_or_tampering_is_rejected() {
        let (bundle, key) = full_bundle();
        let controller = verify_controller_bundle(&bundle, key).unwrap();
        assert_eq!(controller.marker.version, "0.5.0");
        assert_eq!(controller.identity.agent_protocol_contract_id, "b".repeat(64));

        let mut tampered = bundle.clone();
        tampered[64] ^= 1;
        assert!(verify_controller_bundle(&tampered, key).is_err());
        assert!(
            verify_controller_bundle(br#"{"domain":"rustzen-selected-release-v1"}"#, key).is_err()
        );
        assert!(verify_controller_bundle(&bundle, [9_u8; 32]).is_err());
    }

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
