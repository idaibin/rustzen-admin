use crate::install_admission::{
    PrivateParent, destination_name, hash_id, host_target, remove_tree, valid_key_id,
};
use crate::install_crypto::{canonical_json, hash, io, read_regular, verify_signature_bytes};
use crate::install_fs::{
    canonical_ustar_header, fsync_tree, mkdir_private, write_relative, write_secure,
};
use crate::install_manifest::{parse_manifest, validate_payload_contracts};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;
const MAX_MEMBER: u64 = 128 * 1024 * 1024;
#[derive(Debug, Clone)]
pub struct Inputs {
    pub archive: PathBuf,
    pub manifest: PathBuf,
    pub envelope: PathBuf,
    pub trusted_key: PathBuf,
    pub key_id: String,
}
#[derive(Debug, Serialize)]
pub struct Verified {
    pub build_id: String,
    pub artifact_class: String,
    pub target: String,
    pub files: usize,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Manifest {
    pub(super) manifest_version: u8,
    pub(super) release_class: String,
    pub(super) release_version: String,
    pub(super) target: String,
    pub(super) artifact_class: String,
    pub(super) preset: String,
    pub(super) capabilities: Vec<String>,
    pub(super) services: Vec<String>,
    pub(super) composition_id: String,
    pub(super) selection_digest: DigestRecord,
    pub(super) build_id: String,
    pub(super) source_identity: String,
    pub(super) config_digest: String,
    pub(super) native_layout_digest: String,
    pub(super) protocol_artifact_digest: String,
    pub(super) config_owners: Vec<String>,
    pub(super) binary_digests: Vec<BinaryDigest>,
    pub(super) agent_protocol_contract_id: String,
    pub(super) files: Vec<Entry>,
    pub(super) api_digest: Option<String>,
    pub(super) schema_fingerprints: Option<BTreeMap<String, String>>,
    pub(super) data_contract_ids: Option<BTreeMap<String, String>>,
    pub(super) web_digest: Option<DigestRecord>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct DigestRecord {
    pub(super) sha256: String,
    pub(super) source: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct BinaryDigest {
    pub(super) path: String,
    pub(super) sha256: String,
    pub(super) source: String,
}
pub(super) struct Loaded {
    pub(super) archive: Vec<u8>,
    pub(super) manifest_bytes: Vec<u8>,
    pub(super) envelope_bytes: Vec<u8>,
    pub(super) trusted_key: Vec<u8>,
    pub(super) key_id: String,
    pub(super) manifest: Manifest,
    pub(super) verified: Verified,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Entry {
    pub(super) path: String,
    #[serde(rename = "type")]
    pub(super) kind: String,
    pub(super) mode: String,
    pub(super) size: u64,
    pub(super) sha256: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    payload: Payload,
    signature: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Payload {
    pub(super) domain: String,
    pub(super) envelope_version: u8,
    pub(super) algorithm: String,
    pub(super) key_id: String,
    pub(super) release_class: String,
    pub(super) release_version: String,
    pub(super) target: String,
    pub(super) artifact_class: String,
    pub(super) composition_id: String,
    pub(super) build_id: String,
    pub(super) archive_sha256: String,
    pub(super) manifest_sha256: String,
    pub(super) agent_protocol_contract_id: String,
}
pub fn verify(input: &Inputs) -> Result<Verified, String> {
    Ok(load(input)?.verified)
}
pub(super) fn load(input: &Inputs) -> Result<Loaded, String> {
    if !valid_key_id(&input.key_id) {
        return Err("trusted key ID is invalid".into());
    }
    let archive = read_regular(&input.archive, MAX_ARCHIVE)?;
    let manifest_bytes = read_regular(&input.manifest, 4 * 1024 * 1024)?;
    let envelope_bytes = read_regular(&input.envelope, 64 * 1024)?;
    let trusted_key = read_regular(&input.trusted_key, 64 * 1024)?;
    let manifest = parse_manifest(&manifest_bytes)?;
    if manifest.manifest_version != 1
        || manifest.release_class != "production"
        || !matches!(manifest.artifact_class.as_str(), "server" | "node-agent")
        || !hash_id(&manifest.build_id)
        || !hash_id(&manifest.composition_id)
        || !hash_id(&manifest.agent_protocol_contract_id)
    {
        return Err("manifest identity is invalid".into());
    }
    let envelope: Envelope = serde_json::from_slice(&envelope_bytes)
        .map_err(|_| "envelope is invalid JSON".to_string())?;
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
        || !valid_key_id(&payload.key_id)
        || payload.release_class != "production"
    {
        return Err("envelope identity is invalid".into());
    }
    if payload.archive_sha256 != hash(&archive) || payload.manifest_sha256 != hash(&manifest_bytes)
    {
        return Err("release digest differs from envelope".into());
    }
    if (
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
    ) {
        return Err("envelope and manifest tuple differs".into());
    }
    verify_signature_bytes(&trusted_key, payload, &envelope.signature)?;
    let members = scan_archive(&archive, &manifest, &manifest_bytes)?;
    if members != manifest.files.len() + 1 {
        return Err("archive member count differs from manifest".into());
    }
    let verified = Verified {
        build_id: manifest.build_id.clone(),
        artifact_class: manifest.artifact_class.clone(),
        target: manifest.target.clone(),
        files: manifest.files.len(),
    };
    Ok(Loaded {
        archive,
        manifest_bytes,
        envelope_bytes,
        trusted_key,
        key_id: input.key_id.clone(),
        manifest,
        verified,
    })
}
pub fn apply(input: &Inputs, destination: &Path, dry_run: bool) -> Result<Verified, String> {
    let loaded = load(input)?;
    if !cfg!(target_os = "linux") {
        return Err("apply supports Linux only".into());
    }
    if loaded.verified.target != host_target() {
        return Err("release target does not match this host".into());
    }
    let parent = destination.parent().ok_or("destination has no parent")?;
    let destination_name = destination_name(destination)?;
    let parent = PrivateParent::open(parent)?;
    parent.absent(&destination_name)?;
    if dry_run {
        return Ok(loaded.verified);
    }
    if unsafe { libc::geteuid() } != 0 {
        return Err("apply requires root".into());
    }
    let temp_name = format!(".{destination_name}-install-{}", std::process::id());
    parent.absent(&temp_name).map_err(|_| "installer temporary root already exists")?;
    parent.create_temp(&temp_name)?;
    let temp = parent.child_path(&temp_name)?;
    let result =
        publish(&loaded, &temp).and_then(|_| fault_after_publish()).and_then(|_| fsync_tree(&temp));
    if result.is_err() {
        remove_tree(&temp);
        return result.map(|_| loaded.verified);
    }
    if let Err(error) = parent.rename_noreplace(&temp_name, &destination_name) {
        remove_tree(&temp);
        return Err(error);
    }
    parent.sync()?;
    Ok(loaded.verified)
}
fn fault_after_publish() -> Result<(), String> {
    #[cfg(debug_assertions)]
    if std::env::var_os("RUSTZEN_INSTALLER_FAULT_AFTER_PUBLISH").is_some() {
        return Err("debug installer fault after payload publication".into());
    }
    Ok(())
}
pub fn status(destination: &Path) -> Result<serde_json::Value, String> {
    let marker = destination.join("state/publication-marker.json");
    let current = destination.join("current");
    Ok(
        serde_json::json!({"destination": destination, "present": destination.is_dir(), "markerPresent": marker.is_file(), "runnable": false, "current": fs::read_link(current).ok()}),
    )
}

fn publish(input: &Loaded, root: &Path) -> Result<(), String> {
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
    write_secure(&root.join("state/publication-marker.json"), serde_json::to_string(&serde_json::json!({"version":1,"buildId":input.verified.build_id,"artifactClass":input.verified.artifact_class,"state":"payload-published"})).map_err(|_| "marker encoding failed")?.as_bytes())?;
    std::os::unix::fs::symlink(
        format!("releases/{}/payload", input.verified.build_id),
        root.join("current"),
    )
    .map_err(io)?;
    Ok(())
}

fn extract(bytes: &[u8], parsed: &Manifest, payload: &Path) -> Result<(), String> {
    mkdir_private(payload, 0o750)?;
    let root = format!("rz-{}-{}", parsed.artifact_class, parsed.build_id);
    let mut seen = BTreeSet::new();
    let mut offset = 0usize;
    while offset + 512 <= bytes.len() && bytes[offset..offset + 512].iter().any(|b| *b != 0) {
        let h = &bytes[offset..offset + 512];
        let (path, size, mode) = canonical_ustar_header(h)?;
        if size > MAX_MEMBER {
            return Err("archive member exceeds limit".into());
        }
        let end = offset
            .checked_add(512)
            .and_then(|x| x.checked_add(size as usize))
            .ok_or("archive size overflow")?;
        if end > bytes.len() {
            return Err("archive payload truncated".into());
        }
        let padded = end.div_ceil(512) * 512;
        if padded > bytes.len() || bytes[end..padded].iter().any(|byte| *byte != 0) {
            return Err("archive padding is not canonical".into());
        }
        if path == format!("{root}/release-manifest.json") {
            offset = padded;
            continue;
        }
        let expected =
            { parsed.files.iter().find(|x| format!("{root}/payload/{}", x.path) == path) };
        let Some(entry) = expected else {
            return Err("archive member is not selected payload".into());
        };
        if entry.kind != "file"
            || entry.mode != mode
            || entry.size != size
            || entry.sha256 != hash(&bytes[offset + 512..end])
            || !seen.insert(entry.path.clone())
        {
            return Err("archive member differs from manifest".into());
        }
        write_relative(
            payload,
            &entry.path,
            &bytes[offset + 512..end],
            if mode == "0755" { 0o755 } else { 0o644 },
        )?;
        offset = padded;
    }
    if seen.len() != parsed.files.len()
        || offset + 1024 != bytes.len()
        || bytes[offset..].iter().any(|b| *b != 0)
    {
        return Err("archive omits manifest payload member".into());
    }
    Ok(())
}

fn scan_archive(bytes: &[u8], manifest: &Manifest, manifest_bytes: &[u8]) -> Result<usize, String> {
    let root = format!("rz-{}-{}", manifest.artifact_class, manifest.build_id);
    let mut expected = manifest
        .files
        .iter()
        .map(|entry| format!("{root}/payload/{}", entry.path))
        .collect::<Vec<_>>();
    expected.push(format!("{root}/release-manifest.json"));
    expected.sort();
    let mut count = 0;
    let mut seen = BTreeSet::new();
    let mut contracts = BTreeMap::new();
    let mut offset = 0;
    while offset + 512 <= bytes.len() && bytes[offset..offset + 512].iter().any(|b| *b != 0) {
        let (path, size, mode) = canonical_ustar_header(&bytes[offset..offset + 512])?;
        if expected.get(count).is_none_or(|item| item != &path) {
            return Err("archive member inventory or order differs from manifest".into());
        }
        if size > MAX_MEMBER {
            return Err("archive member exceeds limit".into());
        }
        let end = offset + 512 + size as usize;
        if end > bytes.len() {
            return Err("archive payload truncated".into());
        };
        let padded = end.div_ceil(512) * 512;
        if padded > bytes.len() || bytes[end..padded].iter().any(|byte| *byte != 0) {
            return Err("archive padding is not canonical".into());
        }
        if path == format!("{root}/release-manifest.json") {
            if mode != "0644" || &bytes[offset + 512..end] != manifest_bytes {
                return Err("embedded manifest differs".into());
            }
        } else {
            let e = manifest
                .files
                .iter()
                .find(|e| format!("{root}/payload/{}", e.path) == path)
                .ok_or("unexpected archive member")?;
            if e.mode != mode
                || e.size != size
                || e.sha256 != hash(&bytes[offset + 512..end])
                || !seen.insert(e.path.clone())
            {
                return Err("archive member differs from manifest".into());
            }
            if e.path.starts_with("contracts/") {
                contracts.insert(e.path.clone(), &bytes[offset + 512..end]);
            }
        };
        count += 1;
        offset = padded;
    }
    if seen.len() != manifest.files.len()
        || offset + 1024 != bytes.len()
        || bytes[offset..].iter().any(|b| *b != 0)
    {
        return Err("archive ordering, padding or trailing bytes are invalid".into());
    }
    validate_payload_contracts(manifest, &contracts)?;
    Ok(count)
}
