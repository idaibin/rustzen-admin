use crate::protocol_contract::CONTRACT_PROTOCOL_SHA256;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::OpenOptions,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

const MAX_PROFILE_BYTES: u64 = 16 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ControllerProfile {
    version: u8,
    endpoint: String,
    controller_build_id: String,
    controller_composition_id: String,
    agent_build_id: String,
    protocol_id: String,
    key_id: String,
    key_fingerprint: String,
    manifest_sha256: String,
    agent_manifest_sha256: String,
}

pub fn validate_profile(
    path: &Path,
    configured_endpoint: &str,
    agent_root: &Path,
) -> Result<(), String> {
    let bytes = read_root_owned_regular(path, MAX_PROFILE_BYTES)
        .map_err(|error| format!("profile: {error}"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "controller profile is invalid JSON")?;
    if canonical_json(&value)? != bytes {
        return Err("controller profile bytes are not canonical".into());
    }
    let profile: ControllerProfile =
        serde_json::from_value(value).map_err(|_| "controller profile fields are invalid")?;
    if profile.version != 1
        || !same_endpoint(&profile.endpoint, configured_endpoint)
        || rustzen_config::canonical_monitor_endpoint(&profile.endpoint).is_err()
        || profile.protocol_id != CONTRACT_PROTOCOL_SHA256
        || !hash(&profile.controller_build_id)
        || !hash(&profile.controller_composition_id)
        || !hash(&profile.agent_build_id)
        || !hash(&profile.manifest_sha256)
        || !hash(&profile.agent_manifest_sha256)
        || !hash(&profile.key_fingerprint)
        || !key_id(&profile.key_id)
        || !matches_agent_release(agent_root, &profile)
    {
        return Err("controller profile does not match this Agent configuration".into());
    }
    Ok(())
}

/// The Agent is only valid when the executing inode is the published current
/// payload binary named by the retained signed manifest. A copied debug binary
/// cannot opt into a legitimate root by setting environment variables.
pub fn validate_running_binary(agent_root: &Path) -> Result<(), String> {
    let executable = std::fs::read_link("/proc/self/exe")
        .map_err(|_| "Agent executable identity is unavailable")?;
    let root = std::fs::canonicalize(agent_root).map_err(|_| "Agent root is unavailable")?;
    let current = root.join("current");
    let published = std::fs::canonicalize(current.join("bin/rz-monitor-agent"))
        .map_err(|_| "published Agent binary is unavailable")?;
    if executable != published {
        return Err("Agent executable is not the published current binary".into());
    }
    let release = published
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or("published Agent binary path is invalid")?;
    let manifest =
        read_root_owned_regular(&release.join("release-manifest.json"), MAX_PROFILE_BYTES)
            .map_err(|error| format!("retained manifest: {error}"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&manifest).map_err(|_| "Agent manifest is invalid")?;
    if canonical_json(&value)? != manifest {
        return Err("Agent manifest bytes are not canonical".into());
    }
    let expected = value["binaryDigests"]
        .as_array()
        .and_then(|entries| {
            entries
                .iter()
                .find(|entry| entry["path"] == "bin/rz-monitor-agent")
                .and_then(|entry| entry["sha256"].as_str())
        })
        .ok_or("Agent manifest binary digest is unavailable")?;
    let bytes = read_root_owned_regular(&published, 256 * 1024 * 1024)
        .map_err(|error| format!("published binary: {error}"))?;
    if format!("{:x}", Sha256::digest(bytes)) != expected {
        return Err("published Agent binary digest differs from manifest".into());
    }
    Ok(())
}

fn matches_agent_release(root: &Path, profile: &ControllerProfile) -> bool {
    match std::fs::symlink_metadata(root) {
        Ok(value)
            if !value.file_type().is_symlink()
                && value.is_dir()
                && value.uid() == 0
                && value.mode() & 0o022 == 0 => {}
        _ => return false,
    }
    let target = match std::fs::read_link(root.join("current"))
        .ok()
        .and_then(|value| value.into_os_string().into_string().ok())
    {
        Some(value) => value,
        None => return false,
    };
    if target != format!("releases/{}/payload", profile.agent_build_id) {
        return false;
    }
    let release = root.join("releases").join(&profile.agent_build_id);
    match std::fs::symlink_metadata(&release) {
        Ok(value)
            if !value.file_type().is_symlink()
                && value.is_dir()
                && value.uid() == 0
                && value.mode() & 0o022 == 0 => {}
        _ => return false,
    }
    let bytes =
        match read_root_owned_regular(&release.join("release-manifest.json"), MAX_PROFILE_BYTES) {
            Ok(value) => value,
            Err(_) => return false,
        };
    if format!("{:x}", Sha256::digest(&bytes)) != profile.agent_manifest_sha256
        || canonical_json(&serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or_default())
            .ok()
            .as_deref()
            != Some(bytes.as_slice())
    {
        return false;
    }
    let value: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(_) => return false,
    };
    value["buildId"] == profile.agent_build_id
        && value["artifactClass"] == "node-agent"
        && value["agentProtocolContractId"] == profile.protocol_id
}

fn read_root_owned_regular(path: &Path, maximum: u64) -> Result<Vec<u8>, String> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "controller profile is missing or unsafe")?;
    let before = file.metadata().map_err(|_| "controller profile is unreadable")?;
    if !before.file_type().is_file()
        || before.uid() != 0
        || before.mode() & 0o022 != 0
        || before.len() > maximum
    {
        return Err(format!(
            "controller profile ownership or mode is invalid (uid={}, mode={:o}, size={})",
            before.uid(),
            before.mode() & 0o777,
            before.len()
        ));
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    file.read_to_end(&mut bytes).map_err(|_| "controller profile is unreadable")?;
    let after = file.metadata().map_err(|_| "controller profile is unreadable")?;
    if bytes.len() as u64 != before.len()
        || before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
    {
        return Err("controller profile changed while read".into());
    }
    Ok(bytes)
}

fn canonical_json(value: &serde_json::Value) -> Result<Vec<u8>, String> {
    fn append(value: &serde_json::Value, out: &mut String) -> Result<(), String> {
        match value {
            serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::String(_) => {
                out.push_str(&value.to_string())
            }
            serde_json::Value::Number(value)
                if value.as_i64().is_some() || value.as_u64().is_some() =>
            {
                out.push_str(&value.to_string())
            }
            serde_json::Value::Number(_) => {
                return Err("controller profile has noninteger number".into());
            }
            serde_json::Value::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i != 0 {
                        out.push(',');
                    }
                    append(item, out)?;
                }
                out.push(']');
            }
            serde_json::Value::Object(items) => {
                out.push('{');
                let mut keys = items.keys().collect::<Vec<_>>();
                keys.sort();
                for (i, key) in keys.into_iter().enumerate() {
                    if i != 0 {
                        out.push(',');
                    }
                    out.push_str(
                        &serde_json::to_string(key).map_err(|_| "controller profile encoding")?,
                    );
                    out.push(':');
                    append(items.get(key).ok_or("controller profile key changed")?, out)?;
                }
                out.push('}');
            }
        }
        Ok(())
    }
    let mut text = String::new();
    append(value, &mut text)?;
    Ok(text.into_bytes())
}

fn same_endpoint(left: &str, right: &str) -> bool {
    rustzen_config::canonical_monitor_endpoint(left).ok()
        == rustzen_config::canonical_monitor_endpoint(right).ok()
}
fn hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
}
fn key_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_alphanumeric() || (i != 0 && matches!(b, b'.' | b'_' | b'-')))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt};

    fn profile(endpoint: &str, protocol: &str) -> Vec<u8> {
        format!(r#"{{"agentBuildId":"{}","agentManifestSha256":"{}","controllerBuildId":"{}","controllerCompositionId":"{}","endpoint":"{endpoint}","keyFingerprint":"{}","keyId":"release","manifestSha256":"{}","protocolId":"{protocol}","version":1}}"#, "d".repeat(64), "e".repeat(64), "a".repeat(64), "b".repeat(64), "f".repeat(64), "c".repeat(64)).into_bytes()
    }
    #[test]
    fn profile_requires_root_owned_canonical_matching_values() {
        let path = std::env::temp_dir().join(format!("rz-profile-{}", std::process::id()));
        fs::write(&path, profile("https://monitor.example", CONTRACT_PROTOCOL_SHA256)).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(validate_profile(&path, "https://monitor.example/", &path).is_err());
        fs::write(&path, b"{\"version\":1}").unwrap();
        assert!(validate_profile(&path, "https://monitor.example", &path).is_err());
        let _ = fs::remove_file(path);
    }
}
