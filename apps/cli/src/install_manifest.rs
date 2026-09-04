use crate::install_selection;
use crate::{
    install::{BinaryDigest, DigestRecord, Entry, Manifest},
    install_crypto::{canonical_json, hash},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

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
    let web = files.iter().filter(|(path, _)| path.starts_with("web/")).map(|(path, entry)| json!({"path": path.strip_prefix("web/").unwrap_or(path), "sha256": entry.sha256})).collect::<Vec<_>>();
    if web.is_empty()
        || m.web_digest.as_ref().map(|d| &d.sha256)
            != Some(&hash(&canonical_json(&Value::Array(web))?))
    {
        return Err("server web digest differs from payload".into());
    }
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

pub(super) fn validate_payload_contracts(
    manifest: &Manifest,
    files: &BTreeMap<String, &[u8]>,
) -> Result<(), String> {
    let config = object(files, "contracts/config/config.json")?;
    contract_identity(&config, manifest)?;
    if sorted_keys(config.get("owners"))? != manifest.config_owners {
        return Err("config contract owners differ from manifest".into());
    }
    let native = object(files, "contracts/native/native-layout.json")?;
    contract_identity(&native, manifest)?;
    if sorted_strings_value(native.get("configOwners"))? != manifest.config_owners {
        return Err("native contract owners differ from manifest".into());
    }
    let units = native.get("units").and_then(Value::as_array).ok_or("native units invalid")?;
    let expected = manifest
        .files
        .iter()
        .filter(|x| x.path.starts_with("systemd/"))
        .map(|x| (&x.path, &x.sha256))
        .map(|(path, digest)| (path.as_str(), digest.as_str()))
        .collect::<BTreeMap<_, _>>();
    let actual = units
        .iter()
        .map(|unit| {
            let unit = unit.as_object().ok_or("native unit invalid")?;
            Ok((
                unit.get("path").and_then(Value::as_str).ok_or("native unit invalid")?,
                unit.get("sha256").and_then(Value::as_str).ok_or("native unit invalid")?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    if actual != expected {
        return Err("native units differ from manifest payload".into());
    }
    let protocol = object(files, "contracts/protocol/protocol.json")?;
    contract_identity(&protocol, manifest)?;
    if protocol.get("digest").and_then(Value::as_str)
        != Some(manifest.agent_protocol_contract_id.as_str())
    {
        return Err("protocol contract ID differs from manifest".into());
    }
    if manifest.artifact_class == "server" {
        let schema = object(files, "contracts/schema/schema.json")?;
        contract_identity(&schema, manifest)?;
        let owners =
            schema.get("owners").and_then(Value::as_object).ok_or("schema owners invalid")?;
        let schemas = owners
            .iter()
            .map(|(name, value)| {
                Ok((
                    name.clone(),
                    value
                        .get("schemaSha256")
                        .and_then(Value::as_str)
                        .ok_or("schema owner invalid")?
                        .to_owned(),
                ))
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        let data = owners
            .iter()
            .map(|(name, value)| {
                Ok((
                    name.clone(),
                    value
                        .get("dataContractId")
                        .and_then(Value::as_str)
                        .ok_or("schema owner invalid")?
                        .to_owned(),
                ))
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?;
        if manifest.schema_fingerprints.as_ref() != Some(&schemas)
            || manifest.data_contract_ids.as_ref() != Some(&data)
        {
            return Err("schema contracts differ from manifest".into());
        }
    }
    Ok(())
}

fn object(
    files: &BTreeMap<String, &[u8]>,
    path: &str,
) -> Result<serde_json::Map<String, Value>, String> {
    serde_json::from_slice::<Value>(files.get(path).ok_or("contract file missing")?)
        .map_err(|_| "contract JSON invalid")?
        .as_object()
        .cloned()
        .ok_or("contract JSON invalid".into())
}
fn contract_identity(
    value: &serde_json::Map<String, Value>,
    manifest: &Manifest,
) -> Result<(), String> {
    if value.get("artifactClass").and_then(Value::as_str) != Some(manifest.artifact_class.as_str())
        || value.get("preset").and_then(Value::as_str) != Some(manifest.preset.as_str())
        || value.get("compositionId").and_then(Value::as_str)
            != Some(manifest.composition_id.as_str())
    {
        return Err("contract identity differs from manifest".into());
    }
    Ok(())
}
fn sorted_keys(value: Option<&Value>) -> Result<Vec<String>, String> {
    Ok(value.and_then(Value::as_object).ok_or("contract owners invalid")?.keys().cloned().collect())
}
fn sorted_strings_value(value: Option<&Value>) -> Result<Vec<String>, String> {
    value
        .and_then(Value::as_array)
        .ok_or("contract owners invalid")?
        .iter()
        .map(|x| x.as_str().map(str::to_owned).ok_or("contract owners invalid".into()))
        .collect()
}
