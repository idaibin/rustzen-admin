use crate::install_crypto::{canonical_json, hash};
use serde_json::Value;

/// Mirrors the resolver's canonical standalone Agent plan. The manifest is
/// accepted only when its resolver-derived selection digest agrees byte-for-byte.
pub(super) fn digest(preset: &str, target: &str) -> Result<String, String> {
    if !matches!(target, "x86_64-unknown-linux-musl" | "aarch64-unknown-linux-gnu") {
        return Err("selection target is unsupported".into());
    }
    let text = match preset {
        "node-agent" => format!(
            r#"{{"schemaVersion":1,"preset":"node-agent","releaseClass":"production","artifactClass":"node-agent","target":"{target}","capabilities":["monitor-agent"],"compositionId":"5e526bfcee61a36b2c574c4b9cdf5e213445a1c796a06aeb84fbf4bdf012fc1c","services":["monitor-agent"],"owners":["monitor-agent"],"packageTargets":[{{"package":"rustzen-monitor","binary":"rz-monitor-agent","status":"blocked","reason":"Agent configuration, protocol and native layout contracts are implemented; signed Controller pairing profile packaging, Agent installation and runtime certification are not complete.","capabilities":["monitor-agent"]}}],"webRoots":[],"schemaOwners":[],"configOwners":["monitor-agent"],"units":["rz-monitor-agent.service"],"producerReadiness":{{"ready":false,"blockers":["Agent configuration, protocol and native layout contracts are implemented; signed Controller pairing profile packaging, Agent installation and runtime certification are not complete."]}},"notes":["Route and permission inventories are code-derived from Rust registration.","Resolution is a selection plan, not evidence of physical pruning or a build command."]}}"#
        ),
        _ => return Err("selection preset is unsupported".into()),
    };
    let value: Value = serde_json::from_str(&text).map_err(|_| "selection plan encoding failed")?;
    Ok(hash(&canonical_json(&value)?))
}
