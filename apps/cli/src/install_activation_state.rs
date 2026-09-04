use crate::{
    install_admission::{PrivateParent, PublishError},
    install_crypto::read_regular,
};
use serde::Serialize;
use std::{fs, os::unix::fs::MetadataExt, path::Path};

const LOCK_NAME: &str = ".monitor-agent-activation.lock";
const MARKER_NAME: &str = "monitor-agent-activation.json";

pub(super) struct ActivationState {
    parent: PrivateParent,
    _lock: fs::File,
    bytes: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Marker<'a> {
    build_id: &'a str,
    config_sha256: &'a str,
    state: &'static str,
    unit_sha256: &'a str,
    version: u8,
}

impl ActivationState {
    pub(super) fn acquire(
        build_id: &str,
        config_sha256: &str,
        unit_sha256: &str,
    ) -> Result<Self, String> {
        let parent = PrivateParent::open(Path::new("/opt/rz/state"))?;
        let lock = parent.lock_exclusive(LOCK_NAME)?;
        let marker =
            Marker { build_id, config_sha256, state: "activated", unit_sha256, version: 1 };
        let bytes = serde_json::to_vec(&marker).map_err(|_| "activation marker encoding")?;
        Ok(Self { parent, _lock: lock, bytes })
    }

    pub(super) fn already_complete(&self) -> Result<bool, String> {
        let path = self.parent.child_path(MARKER_NAME)?;
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            return Ok(false);
        };
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.uid() != 0
            || metadata.gid() != 0
            || metadata.mode() & 0o777 != 0o600
        {
            return Err("activation marker is unsafe".into());
        }
        if read_regular(&path, 16 * 1024)? != self.bytes {
            return Err("activation marker differs from requested tuple".into());
        }
        self.parent.sync()?;
        Ok(true)
    }

    pub(super) fn complete(&self) -> Result<(), String> {
        let path = self.parent.child_path(MARKER_NAME)?;
        match self.parent.publish_regular_noreplace(MARKER_NAME, &self.bytes, 0, 0, 0o600) {
            Ok(()) => Ok(()),
            Err(PublishError::Conflict) => {
                let metadata = fs::symlink_metadata(&path)
                    .map_err(|_| "activation marker conflict is unreadable")?;
                if metadata.file_type().is_symlink()
                    || !metadata.is_file()
                    || metadata.uid() != 0
                    || metadata.gid() != 0
                    || metadata.mode() & 0o777 != 0o600
                    || read_regular(&path, 16 * 1024)? != self.bytes
                {
                    return Err("activation marker publication conflict".into());
                }
                self.parent.sync()
            }
            Err(error) => Err(error.to_string()),
        }
    }
}
