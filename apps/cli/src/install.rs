use crate::install_admission::{
    PrivateParent, destination_name, hash_id, host_target, remove_tree,
};
use crate::install_continuation as continuation;
use crate::install_crypto::{canonical_json, hash, read_regular, verify_signature_bytes};
use crate::install_fs::fsync_tree;
use crate::install_manifest::parse_manifest;
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;
pub(super) const OFFICIAL_KEY_ID: &str = "rustzen-release";
#[derive(Debug, Clone)]
pub struct Inputs {
    pub archive: PathBuf,
    pub manifest: PathBuf,
    pub envelope: PathBuf,
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
pub(super) fn load(input: &Inputs) -> Result<Loaded, String> {
    let archive = read_regular(&input.archive, MAX_ARCHIVE)?;
    let manifest_bytes = read_regular(&input.manifest, 4 * 1024 * 1024)?;
    let envelope_bytes = read_regular(&input.envelope, 64 * 1024)?;
    let trusted_key = embedded_trusted_key()?;
    let manifest = parse_manifest(&manifest_bytes)?;
    if manifest.manifest_version != 1
        || manifest.release_class != "production"
        || manifest.artifact_class != "node-agent"
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
        || payload.key_id != OFFICIAL_KEY_ID
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
    let members = crate::install_archive::scan_archive(&archive, &manifest, &manifest_bytes)?;
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
        key_id: OFFICIAL_KEY_ID.into(),
        manifest,
        verified,
    })
}

fn embedded_trusted_key() -> Result<Vec<u8>, String> {
    let encoded = option_env!("RUSTZEN_DEPLOY_VERIFY_KEY")
        .ok_or("this rz binary was built without the official release verification key")?;
    let bytes = hex::decode(encoded).map_err(|_| "embedded release verification key is invalid")?;
    let bytes: [u8; 32] =
        bytes.try_into().map_err(|_| "embedded release verification key is invalid")?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| "embedded release verification key is invalid")?;
    let mut der = Vec::with_capacity(44);
    der.extend_from_slice(&[
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
    ]);
    der.extend_from_slice(&bytes);
    Ok(format!("-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n", STANDARD.encode(der))
        .into_bytes())
}
pub fn apply(input: &Inputs, destination: &Path, dry_run: bool) -> Result<Verified, String> {
    let loaded = load(input)?;
    if loaded.verified.artifact_class != "node-agent" {
        return Err("only standalone Monitor Agent artifacts can be installed here".into());
    }
    if !cfg!(target_os = "linux") {
        return Err("apply supports Linux only".into());
    }
    if loaded.verified.target != host_target() {
        return Err("release target does not match this host".into());
    }
    let parent = destination.parent().ok_or("destination has no parent")?;
    let destination_name = destination_name(destination)?;
    let parent = PrivateParent::open(parent)?;
    if dry_run {
        parent.absent(&destination_name)?;
        return Ok(loaded.verified);
    }
    if unsafe { libc::geteuid() } != 0 {
        return Err("apply requires root".into());
    }
    let _lock = parent.lock_exclusive(&format!(".{destination_name}-install.lock"))?;
    let journal_name = format!(".{destination_name}-install.journal");
    let journal = if parent.exists(&journal_name)? {
        continuation::validate_continuation(&parent, &journal_name, &loaded)?
    } else {
        if parent.exists(&destination_name)? {
            continuation::validate_completed_destination(&parent, &destination_name, &loaded)?;
            return Ok(loaded.verified);
        }
        let journal =
            continuation::ContinuationJournal::from_loaded(&loaded, continuation::random_nonce()?);
        let journal_bytes =
            serde_json::to_vec(&journal).map_err(|_| "continuation journal encoding failed")?;
        parent
            .publish_regular_noreplace(&journal_name, &journal_bytes, 0, 0, 0o600)
            .map_err(|error| error.to_string())?;
        journal
    };
    let temp_name = format!(".{destination_name}-install-{}", journal.work_root_nonce);
    let temp = parent.child_path(&temp_name)?;
    if parent.exists(&destination_name)? {
        continuation::validate_published_destination(
            &parent,
            &destination_name,
            &journal,
            &loaded,
        )?;
        continuation::clear_work_marker_if_present(&parent, &destination_name)?;
        continuation::fault_after_work_marker_cleanup()?;
        parent.remove_regular_owned(&journal_name, 0o600)?;
        parent.sync()?;
        continuation::fault_after_journal_cleanup()?;
        return Ok(loaded.verified);
    }
    if parent.exists(&temp_name)? {
        continuation::validate_work_root(&parent, &temp_name, &journal.work_root_nonce)?;
        remove_tree(&temp)?;
        parent.sync()?;
    }
    parent.create_temp(&temp_name)?;
    parent.sync()?;
    continuation::write_work_marker(&parent, &temp_name, &journal.work_root_nonce)?;
    let result = continuation::publish(&loaded, &temp, &journal)
        .and_then(|_| continuation::fault_after_publish())
        .and_then(|_| fsync_tree(&temp));
    if result.is_err() {
        // The sibling journal is the only continuation state. A subsequent
        // apply must revalidate the exact tuple before it can replace this
        // owned work root.
        return result.map(|_| loaded.verified);
    }
    if let Err(error) = parent.rename_noreplace(&temp_name, &destination_name) {
        let _ = remove_tree(&temp);
        return Err(error);
    }
    parent.sync()?;
    continuation::fault_after_rename()?;
    continuation::clear_work_marker_if_present(&parent, &destination_name)?;
    continuation::fault_after_work_marker_cleanup()?;
    parent.remove_regular_owned(&journal_name, 0o600)?;
    parent.sync()?;
    continuation::fault_after_journal_cleanup()?;
    Ok(loaded.verified)
}
