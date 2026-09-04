use crate::{canonical_monitor_endpoint, valid_monitor_agent_token};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

const MAX_BYTES: u64 = 16 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentEnvironment {
    pub environment: String,
    pub node_id: String,
    pub endpoint: String,
    pub token: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ControllerProfile {
    pub version: u8,
    pub endpoint: String,
    pub controller_build_id: String,
    pub controller_composition_id: String,
    pub agent_build_id: String,
    pub protocol_id: String,
    pub key_id: String,
    pub key_fingerprint: String,
    pub manifest_sha256: String,
    pub agent_manifest_sha256: String,
}

pub fn read_agent_environment(path: &Path, expected_gid: u32) -> Result<AgentEnvironment, String> {
    let bytes = read_root_owned_regular(path, expected_gid, 0o640, MAX_BYTES)?;
    parse_agent_environment_bytes(&bytes)
}

pub fn parse_agent_environment_bytes(bytes: &[u8]) -> Result<AgentEnvironment, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "Agent environment is not utf8")?;
    let mut values = BTreeMap::new();
    for line in text.split_terminator('\n') {
        let (key, value) = line.split_once('=').ok_or("Agent environment line is invalid")?;
        if key.is_empty() || value.is_empty() || values.insert(key, value).is_some() {
            return Err("Agent environment keys are invalid".into());
        }
    }
    if values.len() != 4
        || values.keys().copied().collect::<Vec<_>>()
            != [
                "RUSTZEN_ENV",
                "RUSTZEN_MONITOR_AGENT_TOKEN",
                "RUSTZEN_MONITOR_CONTROLLER_URL",
                "RUSTZEN_MONITOR_NODE_ID",
            ]
    {
        return Err("Agent environment keys are invalid".into());
    }
    let environment = values["RUSTZEN_ENV"];
    let node_id = values["RUSTZEN_MONITOR_NODE_ID"];
    let endpoint = canonical_monitor_endpoint(values["RUSTZEN_MONITOR_CONTROLLER_URL"])
        .map_err(|_| "Agent environment endpoint is invalid")?;
    let token = values["RUSTZEN_MONITOR_AGENT_TOKEN"];
    if environment != "production"
        || !endpoint.starts_with("https://")
        || node_id.is_empty()
        || node_id.len() > 128
        || !node_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        || !valid_monitor_agent_token(token)
    {
        return Err("Agent environment values are invalid".into());
    }
    let canonical = format!(
        "RUSTZEN_ENV={environment}\nRUSTZEN_MONITOR_AGENT_TOKEN={token}\nRUSTZEN_MONITOR_CONTROLLER_URL={endpoint}\nRUSTZEN_MONITOR_NODE_ID={node_id}\n"
    );
    if bytes != canonical.as_bytes() {
        return Err("Agent environment bytes are not canonical".into());
    }
    Ok(AgentEnvironment {
        environment: environment.to_owned(),
        node_id: node_id.to_owned(),
        endpoint,
        token: token.to_owned(),
    })
}

pub fn read_controller_profile(
    path: &Path,
    expected_gid: u32,
) -> Result<ControllerProfile, String> {
    let bytes = read_root_owned_regular(path, expected_gid, 0o640, MAX_BYTES)?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "controller profile is invalid JSON")?;
    if canonical_json(&value)? != bytes {
        return Err("controller profile bytes are not canonical".into());
    }
    let profile: ControllerProfile =
        serde_json::from_value(value).map_err(|_| "controller profile fields are invalid")?;
    if profile.version != 1
        || canonical_monitor_endpoint(&profile.endpoint).is_err()
        || !hash(&profile.controller_build_id)
        || !hash(&profile.controller_composition_id)
        || !hash(&profile.agent_build_id)
        || !hash(&profile.protocol_id)
        || !hash(&profile.key_fingerprint)
        || !hash(&profile.manifest_sha256)
        || !hash(&profile.agent_manifest_sha256)
        || !key_id(&profile.key_id)
    {
        return Err("controller profile fields are invalid".into());
    }
    Ok(profile)
}

pub fn read_root_owned_regular(
    path: &Path,
    gid: u32,
    mode: u32,
    maximum: u64,
) -> Result<Vec<u8>, String> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "activation input is missing or unsafe")?;
    let before = file.metadata().map_err(|_| "activation input is unreadable")?;
    if !before.file_type().is_file()
        || before.uid() != 0
        || before.gid() != gid
        || before.mode() & 0o777 != mode
        || before.len() > maximum
    {
        return Err("activation input ownership or mode is invalid".into());
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    file.read_to_end(&mut bytes).map_err(|_| "activation input is unreadable")?;
    let after = file.metadata().map_err(|_| "activation input is unreadable")?;
    if bytes.len() as u64 != before.len()
        || before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.mode() != after.mode()
        || before.len() != after.len()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
    {
        return Err("activation input changed while read".into());
    }
    Ok(bytes)
}

fn canonical_json(value: &serde_json::Value) -> Result<Vec<u8>, String> {
    fn append(value: &serde_json::Value, output: &mut String) -> Result<(), String> {
        match value {
            serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::String(_) => {
                output.push_str(&value.to_string())
            }
            serde_json::Value::Number(value)
                if value.as_i64().is_some() || value.as_u64().is_some() =>
            {
                output.push_str(&value.to_string())
            }
            serde_json::Value::Number(_) => {
                return Err("controller profile has noninteger number".into());
            }
            serde_json::Value::Array(values) => {
                output.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    append(value, output)?;
                }
                output.push(']');
            }
            serde_json::Value::Object(values) => {
                output.push('{');
                let mut keys = values.keys().collect::<Vec<_>>();
                keys.sort();
                for (index, key) in keys.into_iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    output.push_str(
                        &serde_json::to_string(key).map_err(|_| "controller profile encoding")?,
                    );
                    output.push(':');
                    append(values.get(key).ok_or("controller profile key changed")?, output)?;
                }
                output.push('}');
            }
        }
        Ok(())
    }
    let mut output = String::new();
    append(value, &mut output)?;
    Ok(output.into_bytes())
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
    use super::parse_agent_environment_bytes;

    const ENV: &[u8] = b"RUSTZEN_ENV=production\nRUSTZEN_MONITOR_AGENT_TOKEN=fixture-token-is-valid\nRUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example\nRUSTZEN_MONITOR_NODE_ID=fixture-agent\n";

    #[test]
    fn agent_environment_requires_the_exact_canonical_four_key_bytes() {
        let environment = parse_agent_environment_bytes(ENV).expect("canonical environment");
        assert_eq!(environment.endpoint, "https://monitor.example");
        assert!(parse_agent_environment_bytes(
            b"RUSTZEN_ENV=production\nRUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example\nRUSTZEN_MONITOR_AGENT_TOKEN=fixture-token-is-valid\nRUSTZEN_MONITOR_NODE_ID=fixture-agent\n"
        )
        .is_err());
        assert!(parse_agent_environment_bytes(
            b"RUSTZEN_ENV=production\nRUSTZEN_MONITOR_AGENT_TOKEN=placeholder\nRUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example\nRUSTZEN_MONITOR_NODE_ID=fixture-agent\n"
        )
        .is_err());
        for endpoint in ["http://127.0.0.1:9801", "http://[::1]:9801"] {
            let bytes = format!(
                "RUSTZEN_ENV=production\nRUSTZEN_MONITOR_AGENT_TOKEN=fixture-token-is-valid\nRUSTZEN_MONITOR_CONTROLLER_URL={endpoint}\nRUSTZEN_MONITOR_NODE_ID=fixture-agent\n"
            );
            assert!(parse_agent_environment_bytes(bytes.as_bytes()).is_err());
        }
    }
}
