use crate::install::Manifest;
use serde_json::Value;
use std::collections::BTreeMap;

pub(crate) fn validate_payload_contracts(
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
    {
        return Err("contract identity differs from manifest".into());
    }
    contract_selection_identity(value, manifest)
}

fn contract_selection_identity(
    value: &serde_json::Map<String, Value>,
    manifest: &Manifest,
) -> Result<(), String> {
    if value.get("preset").and_then(Value::as_str) != Some(manifest.preset.as_str())
        || value.get("compositionId").and_then(Value::as_str)
            != Some(manifest.composition_id.as_str())
    {
        Err("contract identity differs from manifest".into())
    } else {
        Ok(())
    }
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
