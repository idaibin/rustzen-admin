use crate::{
    install::Manifest,
    install_admission::validate_root_owned_path,
    install_crypto::{hash, read_regular},
    install_manifest::parse_manifest,
};
use std::{collections::BTreeMap, fs, path::Path};

const ROOT: &str = "/opt/rz";
const UNITS: [&str; 3] = ["rz-admin.service", "rz-monitor.service", "rz.target"];
pub(super) struct ServerRelease {
    pub(super) preset: String,
    pub(super) build_id: String,
    pub(super) composition_id: String,
    pub(super) units: BTreeMap<String, Vec<u8>>,
    pub(super) schema_fingerprints: BTreeMap<String, String>,
    pub(super) data_contract_ids: BTreeMap<String, String>,
}
impl ServerRelease {
    pub(super) fn load() -> Result<Self, String> {
        let root = Path::new(ROOT);
        validate_root_owned_path(root)?;
        let current = fs::read_link(root.join("current"))
            .map_err(|_| "Monitor server current link is missing")?;
        let current = current.to_str().ok_or("Monitor server current link is invalid")?;
        let build_id = current
            .strip_prefix("releases/")
            .and_then(|x| x.strip_suffix("/payload"))
            .ok_or("Monitor server current link is invalid")?;
        let manifest: Manifest = parse_manifest(&read_regular(
            &root.join("releases").join(build_id).join("release-manifest.json"),
            4 * 1024 * 1024,
        )?)?;
        if manifest.build_id != build_id
            || manifest.artifact_class != "server"
            || !(manifest.preset == "monitor" || manifest.preset == "monitor-notify")
            || manifest.capabilities.as_slice()
                != if manifest.preset == "monitor-notify" {
                    ["access", "monitor", "notifications"].as_slice()
                } else {
                    ["access", "monitor"].as_slice()
                }
            || manifest.services != ["admin", "monitor"]
        {
            return Err("installed release is not the production Monitor server selection".into());
        }
        let mut units = BTreeMap::new();
        for name in UNITS {
            let bytes = read_regular(&root.join(current).join("systemd").join(name), 64 * 1024)?;
            let expected = manifest
                .files
                .iter()
                .find(|entry| entry.path == format!("systemd/{name}"))
                .ok_or("selected unit digest is unavailable")?;
            if expected.sha256 != hash(&bytes) {
                return Err("selected native unit differs from manifest".into());
            }
            units.insert(name.into(), bytes);
        }
        for binary in ["rz-admin", "rz-monitor"] {
            let bytes =
                read_regular(&root.join(current).join("bin").join(binary), 256 * 1024 * 1024)?;
            if manifest
                .binary_digests
                .iter()
                .find(|entry| entry.path == format!("bin/{binary}"))
                .is_none_or(|entry| entry.sha256 != hash(&bytes))
            {
                return Err("selected server binary differs from manifest".into());
            }
        }
        Ok(Self {
            preset: manifest.preset,
            build_id: build_id.into(),
            composition_id: manifest.composition_id,
            units,
            schema_fingerprints: manifest
                .schema_fingerprints
                .ok_or("selected schema identity is unavailable")?,
            data_contract_ids: manifest
                .data_contract_ids
                .ok_or("selected data contract identity is unavailable")?,
        })
    }
}
