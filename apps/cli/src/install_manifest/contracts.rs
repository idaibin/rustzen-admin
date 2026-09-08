use super::hash_id;
use crate::install::Manifest;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn validate_payload_contracts(
    manifest: &Manifest,
    files: &BTreeMap<String, &[u8]>,
) -> Result<(), String> {
    let config = object(files, "contracts/config/config.json")?;
    contract_identity(&config, manifest)?;
    super::web::validate(manifest, files)?;
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
        schema_identity(&schema, manifest)?;
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

fn schema_identity(
    value: &serde_json::Map<String, Value>,
    manifest: &Manifest,
) -> Result<(), String> {
    if value.keys().map(String::as_str).collect::<BTreeSet<_>>()
        != BTreeSet::from(["compositionId", "owners", "preset"])
    {
        return Err("schema contract fields are invalid".into());
    }
    contract_selection_identity(value, manifest)?;
    let owners = value.get("owners").and_then(Value::as_object).ok_or("schema owners invalid")?;
    if owners.keys().map(String::as_str).collect::<BTreeSet<_>>()
        != BTreeSet::from(["admin", "monitor"])
    {
        return Err("schema owners invalid".into());
    }
    for owner in owners.values() {
        let owner = owner.as_object().ok_or("schema owner invalid")?;
        if owner.keys().map(String::as_str).collect::<BTreeSet<_>>()
            != BTreeSet::from(["dataContractId", "schemaSha256"])
            || !owner.get("dataContractId").and_then(Value::as_str).is_some_and(hash_id)
            || !owner.get("schemaSha256").and_then(Value::as_str).is_some_and(hash_id)
        {
            return Err("schema owner invalid".into());
        }
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
