use crate::install_crypto::{canonical_json, hash};
use serde_json::Value;

/// Mirrors the resolver's canonical monitor and Agent plans.  The manifest is
/// accepted only when its resolver-derived selection digest agrees byte-for-byte.
pub(super) fn digest(preset: &str, target: &str) -> Result<String, String> {
    if !matches!(target, "x86_64-unknown-linux-musl" | "aarch64-unknown-linux-gnu") {
        return Err("selection target is unsupported".into());
    }
    let text = match preset {
        "monitor" => format!(
            r#"{{"schemaVersion":1,"preset":"monitor","releaseClass":"production","artifactClass":"server","target":"{target}","capabilities":["access","monitor"],"compositionId":"8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b","services":["admin","monitor"],"owners":["access","monitor"],"packageTargets":[{{"package":"rustzen-admin","binary":"rz-admin","status":"blocked","reason":"Minimal Admin, selected Web and selected API/schema/config/native contracts are implemented; signed native packaging, installation and runtime certification are not complete.","capabilities":["access"]}},{{"package":"rustzen-monitor","binary":"rz-monitor","status":"blocked","reason":"Selected Controller configuration and protocol contracts are implemented; signed native packaging, installation and runtime certification are not complete.","capabilities":["monitor"]}}],"webRoots":["apps/web/src/routes/__root.tsx","apps/web/src/routes/403.tsx","apps/web/src/routes/404.tsx","apps/web/src/routes/login.tsx","apps/web/src/routes/monitoring.tsx","apps/web/src/routes/monitoring/incidents.tsx","apps/web/src/routes/monitoring/nodes.tsx","apps/web/src/routes/monitoring/overview.tsx","apps/web/src/routes/monitoring/summaries.tsx","apps/web/src/routes/profile.tsx","apps/web/src/routes/system/role.tsx","apps/web/src/routes/system/user.tsx"],"schemaOwners":["admin","monitor"],"configOwners":["access","monitor"],"units":["rz-admin.service","rz-monitor.service"],"producerReadiness":{{"ready":false,"blockers":["Minimal Admin, selected Web and selected API/schema/config/native contracts are implemented; signed native packaging, installation and runtime certification are not complete.","Selected Controller configuration and protocol contracts are implemented; signed native packaging, installation and runtime certification are not complete."]}},"notes":["Route and permission inventories are code-derived from Rust registration.","Resolution is a selection plan, not evidence of physical pruning or a build command."]}}"#
        ),
        "monitor-notify" => {
            return match target {
                "x86_64-unknown-linux-musl" => {
                    Ok("225408b0137c11840e14983a9cafd2ab2477423834865998a30abaccf67fe464".into())
                }
                "aarch64-unknown-linux-gnu" => {
                    Ok("d024bf9211a4984f1a9c1170583b7476f5ab246720c319e4c5791c9e33c1e310".into())
                }
                _ => Err("selection target is unsupported".into()),
            };
        }
        "node-agent" => format!(
            r#"{{"schemaVersion":1,"preset":"node-agent","releaseClass":"production","artifactClass":"node-agent","target":"{target}","capabilities":["monitor-agent"],"compositionId":"5e526bfcee61a36b2c574c4b9cdf5e213445a1c796a06aeb84fbf4bdf012fc1c","services":["monitor-agent"],"owners":["monitor-agent"],"packageTargets":[{{"package":"rustzen-monitor","binary":"rz-monitor-agent","status":"blocked","reason":"Agent configuration, protocol and native layout contracts are implemented; signed Controller pairing profile packaging, Agent installation and runtime certification are not complete.","capabilities":["monitor-agent"]}}],"webRoots":[],"schemaOwners":[],"configOwners":["monitor-agent"],"units":["rz-monitor-agent.service"],"producerReadiness":{{"ready":false,"blockers":["Agent configuration, protocol and native layout contracts are implemented; signed Controller pairing profile packaging, Agent installation and runtime certification are not complete."]}},"notes":["Route and permission inventories are code-derived from Rust registration.","Resolution is a selection plan, not evidence of physical pruning or a build command."]}}"#
        ),
        _ => return Err("selection preset is unsupported".into()),
    };
    let value: Value = serde_json::from_str(&text).map_err(|_| "selection plan encoding failed")?;
    Ok(hash(&canonical_json(&value)?))
}

#[cfg(test)]
#[path = "install_selection_tests.rs"]
mod tests;
