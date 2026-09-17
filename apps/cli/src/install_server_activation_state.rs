use crate::{
    install_admission::{PrivateParent, PublishError},
    install_crypto::{hash, read_regular},
};
use std::{collections::BTreeMap, fs, os::unix::fs::MetadataExt, path::Path};

pub(super) struct ServerActivationState {
    parent: PrivateParent,
    _lock: fs::File,
    bytes: Vec<u8>,
}
impl ServerActivationState {
    pub(super) fn acquire(
        preset: &str,
        build: &str,
        admin: &[u8],
        monitor: &[u8],
        units: &BTreeMap<String, Vec<u8>>,
        schema_fingerprints: &BTreeMap<String, String>,
        data_contract_ids: &BTreeMap<String, String>,
    ) -> Result<Self, String> {
        let parent = PrivateParent::open(Path::new("/opt/rz/state"))?;
        let lock = parent.lock_exclusive(".monitor-server-activation.lock")?;
        let unit_hashes =
            units.iter().map(|(name, bytes)| (name, hash(bytes))).collect::<BTreeMap<_, _>>();
        let selection = crate::install_server_selection::for_preset(preset)?;
        if unit_hashes.keys().map(|name| (*name).as_str()).collect::<Vec<_>>() != selection.units {
            return Err("selected unit inventory is invalid".into());
        }
        let owners = owners_for_preset(preset)?;
        if schema_fingerprints.keys().map(String::as_str).collect::<Vec<_>>() != owners
            || data_contract_ids.keys().map(String::as_str).collect::<Vec<_>>() != owners
        {
            return Err("selected schema identity is invalid".into());
        }
        let value = serde_json::json!({
            "adminConfigSha256": hash(admin),
            "buildId": build,
            "monitorConfigSha256": hash(monitor),
            "schemaFingerprint": schema_fingerprints,
            "dataContractId": data_contract_ids,
            "state": "ready",
            "unitSha256": unit_hashes,
            "version": 2
        });
        Ok(Self {
            parent,
            _lock: lock,
            bytes: serde_json::to_vec(&value).map_err(|_| "activation marker encoding")?,
        })
    }
    pub(super) fn already_complete(&self) -> Result<bool, String> {
        let path = self.parent.child_path("monitor-server-activation.json")?;
        if !path.exists() {
            return Ok(false);
        }
        let m = fs::symlink_metadata(&path).map_err(|_| "activation marker is unavailable")?;
        if m.file_type().is_symlink()
            || m.uid() != 0
            || m.gid() != 0
            || m.mode() & 0o777 != 0o600
            || read_regular(&path, 16 * 1024)? != self.bytes
        {
            return Err("activation marker differs from requested tuple".into());
        }
        Ok(true)
    }
    pub(super) fn tuple_sha256(&self) -> String {
        hash(&self.bytes)
    }
    pub(super) fn complete(&self) -> Result<(), String> {
        let published = self.parent.publish_regular_noreplace(
            "monitor-server-activation.json",
            &self.bytes,
            0,
            0,
            0o600,
        );
        #[cfg(debug_assertions)]
        if published.is_ok()
            && std::env::var("RUSTZEN_MONITOR_SERVER_ACTIVATION_FAULT").ok().as_deref()
                == Some("marker-dirsync")
        {
            return self.reconcile_after_publication();
        }
        match published {
            Ok(()) => Ok(()),
            Err(_) if self.already_complete()? => self.reconcile_after_publication(),
            Err(PublishError::Conflict) => Err("activation marker publication conflict".into()),
            Err(error) => Err(error.to_string()),
        }
    }

    fn reconcile_after_publication(&self) -> Result<(), String> {
        self.parent.sync()?;
        if self.already_complete()? {
            Ok(())
        } else {
            Err("activation marker durability could not be reconciled".into())
        }
    }
}

fn owners_for_preset(preset: &str) -> Result<Vec<&'static str>, String> {
    match preset {
        "monitor" => Ok(vec!["admin", "monitor"]),
        "monitor-notify" => {
            Ok(vec!["admin", "admin-notifications", "monitor", "monitor-notifications"])
        }
        "analytics" => Ok(vec!["admin", "insights"]),
        _ => Err("selected server preset is invalid".into()),
    }
}
#[cfg(test)]
#[path = "install_server_activation_state_tests.rs"]
mod tests;
