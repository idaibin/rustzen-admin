use crate::{
    install_admission::{PrivateParent, PublishError},
    install_crypto::hash,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

const NAME: &str = "monitor-server-database-journal.json";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct DatabaseJournal {
    version: u8,
    tuple_sha256: String,
    pub(super) admin_stage: String,
    pub(super) monitor_stage: String,
    pub(super) admin_sha256: String,
    pub(super) monitor_sha256: String,
    state: String,
}
impl DatabaseJournal {
    pub(super) fn new(
        tuple_sha256: String,
        admin_stage: String,
        monitor_stage: String,
        admin: &[u8],
        monitor: &[u8],
    ) -> Self {
        Self {
            version: 1,
            tuple_sha256,
            admin_stage,
            monitor_stage,
            admin_sha256: hash(admin),
            monitor_sha256: hash(monitor),
            state: "databases-staged".into(),
        }
    }

    pub(super) fn load(tuple_sha256: &str) -> Result<Option<Self>, String> {
        let parent = PrivateParent::open(Path::new("/opt/rz/state"))?;
        if !parent.exists(NAME)? {
            return Ok(None);
        }
        let bytes = parent.read_regular_owned(NAME, 16 * 1024, 0o600)?;
        let journal: Self =
            serde_json::from_slice(&bytes).map_err(|_| "activation database journal is invalid")?;
        if journal.version != 1
            || journal.state != "databases-staged"
            || journal.tuple_sha256 != tuple_sha256
        {
            return Err("activation database journal differs from requested tuple".into());
        }
        Ok(Some(journal))
    }

    pub(super) fn publish(&self) -> Result<(), String> {
        let parent = PrivateParent::open(Path::new("/opt/rz/state"))?;
        let bytes = serde_json::to_vec(self).map_err(|_| "activation journal encoding failed")?;
        match parent.publish_regular_noreplace(NAME, &bytes, 0, 0, 0o600) {
            Ok(()) => Ok(()),
            Err(PublishError::Conflict)
                if Self::load(&self.tuple_sha256)?.as_ref() == Some(self) =>
            {
                Ok(())
            }
            Err(PublishError::Conflict) => {
                Err("activation database journal publication conflict".into())
            }
            Err(error) => Err(error.to_string()),
        }
    }

    pub(super) fn remove(tuple_sha256: &str) -> Result<(), String> {
        if Self::load(tuple_sha256)?.is_some() {
            fault("journal-remove")?;
            PrivateParent::open(Path::new("/opt/rz/state"))?.remove_regular_owned(NAME, 0o600)?;
        }
        Ok(())
    }
}

pub(super) fn fault(_stage: &str) -> Result<(), String> {
    #[cfg(debug_assertions)]
    if std::env::var("RUSTZEN_MONITOR_SERVER_ACTIVATION_FAULT").ok().as_deref() == Some(_stage) {
        return Err(format!("debug monitor server activation fault at {_stage}"));
    }
    Ok(())
}
