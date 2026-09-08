use crate::install_selection;
use crate::{
    install::{BinaryDigest, DigestRecord, Entry, Manifest},
    install_crypto::{canonical_json, hash},
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

mod contracts;
mod web;
pub(super) use contracts::validate_payload_contracts;

pub(super) fn parse_manifest(bytes: &[u8]) -> Result<Manifest, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| "manifest is invalid JSON")?;
    let object = value.as_object().ok_or("manifest is not an object")?;
    let class = object
        .get("artifactClass")
        .and_then(Value::as_str)
        .ok_or("manifest artifact class is invalid")?;
    let base = [
        "manifestVersion",
        "releaseClass",
        "releaseVersion",
        "target",
        "artifactClass",
        "preset",
        "capabilities",
        "services",
        "compositionId",
        "selectionDigest",
        "buildId",
        "sourceIdentity",
        "configDigest",
        "nativeLayoutDigest",
        "protocolArtifactDigest",
        "configOwners",
        "binaryDigests",
        "files",
        "agentProtocolContractId",
    ];
    let server = ["apiDigest", "schemaFingerprints", "dataContractIds", "webDigest"];
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = if class == "server" {
        base.iter().chain(server.iter())
    } else if class == "node-agent" {
        base.iter().chain([].iter())
    } else {
        return Err("manifest artifact class is invalid".into());
    }
    .copied()
    .collect();
    if actual != expected || canonical_json(&value)? != bytes {
        return Err("manifest fields or bytes are invalid".into());
    }
    let manifest: Manifest =
        serde_json::from_value(value).map_err(|_| "manifest fields are invalid")?;
    validate(&manifest)?;
    Ok(manifest)
}

fn validate(m: &Manifest) -> Result<(), String> {
    if m.manifest_version != 1
        || m.release_class != "production"
        || !nonempty(&m.release_version)
        || !nonempty(&m.source_identity)
        || !target(&m.target)
        || !hash_id(&m.composition_id)
        || !hash_id(&m.build_id)
        || !hash_id(&m.agent_protocol_contract_id)
        || !hash_id(&m.config_digest)
        || !hash_id(&m.native_layout_digest)
        || !hash_id(&m.protocol_artifact_digest)
    {
        return Err("manifest identity is invalid".into());
    }
    digest(&m.selection_digest, "resolved-selection")?;
    if m.selection_digest.sha256 != install_selection::digest(&m.preset, &m.target)? {
        return Err("manifest selection digest differs from resolver plan".into());
    }
    strings(&m.capabilities)?;
    strings(&m.services)?;
    strings(&m.config_owners)?;
    let files = file_map(&m.files)?;
    binaries(&m.binary_digests, &files)?;
    if !match m.artifact_class.as_str() {
        "server" => server(m, &files)?,
        "node-agent" => agent(m, &files)?,
        _ => return Err("manifest artifact class is invalid".into()),
    } {
        return Err("manifest payload inventory is invalid".into());
    }
    Ok(())
}

fn server(m: &Manifest, files: &BTreeMap<&str, &Entry>) -> Result<bool, String> {
    if m.preset != "monitor"
        || m.capabilities != ["access", "monitor"]
        || m.services != ["admin", "monitor"]
        || m.config_owners != ["access", "monitor"]
        || m.composition_id
            != hash(b"{\"artifactClass\":\"server\",\"capabilities\":[\"access\",\"monitor\"],\"capabilityContractVersion\":1}")
        || m.schema_fingerprints.as_ref().is_none_or(|x| !owners(x))
        || m.data_contract_ids.as_ref().is_none_or(|x| !owners(x))
    {
        return Err("server manifest differs from monitor selection".into());
    }
    if m.api_digest.as_deref() != Some(&file_hash(files, "contracts/api/api.json")?)
        || m.config_digest != file_hash(files, "contracts/config/config.json")?
        || m.native_layout_digest != file_hash(files, "contracts/native/native-layout.json")?
        || m.protocol_artifact_digest != file_hash(files, "contracts/protocol/protocol.json")?
    {
        return Err("server manifest contract digest differs from payload".into());
    }
    digest(m.web_digest.as_ref().ok_or("server web digest missing")?, "selected-web-files")?;
    Ok(exact_paths(
        files,
        &[
            "bin/rz-admin",
            "bin/rz-monitor",
            "contracts/api/api.json",
            "contracts/config/config.json",
            "contracts/native/native-layout.json",
            "contracts/protocol/protocol.json",
            "contracts/schema/schema.json",
            "contracts/web/binding.json",
            "systemd/rz-admin.service",
            "systemd/rz-monitor.service",
            "systemd/rz.target",
        ],
        true,
    ))
}

fn agent(m: &Manifest, files: &BTreeMap<&str, &Entry>) -> Result<bool, String> {
    if m.preset != "node-agent"
        || m.capabilities != ["monitor-agent"]
        || m.services != ["monitor-agent"]
        || m.config_owners != ["monitor-agent"]
        || m.composition_id
            != hash(b"{\"artifactClass\":\"node-agent\",\"capabilities\":[\"monitor-agent\"],\"capabilityContractVersion\":1}")
        || m.api_digest.is_some()
        || m.schema_fingerprints.is_some()
        || m.data_contract_ids.is_some()
        || m.web_digest.is_some()
    {
        return Err("node-agent manifest has server fields or selection".into());
    }
    if m.config_digest != file_hash(files, "contracts/config/config.json")?
        || m.native_layout_digest != file_hash(files, "contracts/native/native-layout.json")?
        || m.protocol_artifact_digest != file_hash(files, "contracts/protocol/protocol.json")?
    {
        return Err("node-agent contract digest differs from payload".into());
    }
    Ok(exact_paths(
        files,
        &[
            "bin/rz-monitor-agent",
            "contracts/config/config.json",
            "contracts/native/native-layout.json",
            "contracts/protocol/protocol.json",
            "systemd/rz-monitor-agent.service",
        ],
        false,
    ))
}

fn file_map(files: &[Entry]) -> Result<BTreeMap<&str, &Entry>, String> {
    if files.is_empty() {
        return Err("manifest files are empty".into());
    }
    let mut result = BTreeMap::new();
    for file in files {
        if file.kind != "file"
            || !path(&file.path)
            || !hash_id(&file.sha256)
            || (file.path.starts_with("bin/") && file.mode != "0755")
            || (!file.path.starts_with("bin/") && file.mode != "0644")
            || result.insert(file.path.as_str(), file).is_some()
        {
            return Err("manifest file inventory is invalid".into());
        }
    }
    if result.keys().copied().collect::<Vec<_>>()
        != files.iter().map(|x| x.path.as_str()).collect::<Vec<_>>()
    {
        return Err("manifest files must be sorted".into());
    }
    Ok(result)
}

fn binaries(values: &[BinaryDigest], files: &BTreeMap<&str, &Entry>) -> Result<(), String> {
    if values.is_empty() {
        return Err("manifest binary digests are empty".into());
    }
    let mut names = BTreeSet::new();
    for value in values {
        if value.source != "binary-file"
            || !path(&value.path)
            || !value.path.starts_with("bin/")
            || !hash_id(&value.sha256)
            || files.get(value.path.as_str()).is_none_or(|file| file.sha256 != value.sha256)
            || !names.insert(value.path.as_str())
        {
            return Err("manifest binary digest differs from payload".into());
        }
    }
    let actual = files.keys().filter(|x| x.starts_with("bin/")).copied().collect::<BTreeSet<_>>();
    if actual != names {
        return Err("binary digests must exactly name binary files".into());
    }
    Ok(())
}

fn exact_paths(files: &BTreeMap<&str, &Entry>, fixed: &[&str], web: bool) -> bool {
    files.keys().all(|path| fixed.contains(path) || (web && path.starts_with("web/")))
        && fixed.iter().all(|path| files.contains_key(path))
}
fn file_hash(files: &BTreeMap<&str, &Entry>, path: &str) -> Result<String, String> {
    Ok(files.get(path).ok_or("manifest required contract is missing")?.sha256.clone())
}
fn owners(values: &BTreeMap<String, String>) -> bool {
    values.keys().map(String::as_str).collect::<Vec<_>>() == ["admin", "monitor"]
        && values.values().all(|x| hash_id(x))
}
fn digest(value: &DigestRecord, source: &str) -> Result<(), String> {
    if value.source == source && hash_id(&value.sha256) {
        Ok(())
    } else {
        Err("manifest digest is invalid".into())
    }
}
fn strings(values: &[String]) -> Result<(), String> {
    if values.iter().all(|x| nonempty(x)) && values.windows(2).all(|x| x[0] < x[1]) {
        Ok(())
    } else {
        Err("manifest strings must be sorted and unique".into())
    }
}
fn hash_id(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|x| x.is_ascii_digit() || matches!(x, b'a'..=b'f'))
}
fn target(value: &str) -> bool {
    matches!(value, "x86_64-unknown-linux-musl" | "aarch64-unknown-linux-gnu")
}
fn nonempty(value: &str) -> bool {
    !value.is_empty()
}
fn path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains('\\')
        && ![
            "manifest.json",
            "release-manifest.json",
            "signature.json",
            "envelope.json",
            "signature-envelope.json",
        ]
        .contains(&value)
        && value.split('/').all(|x| !x.is_empty() && x != "." && x != "..")
}
