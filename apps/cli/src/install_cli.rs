use crate::install;
use std::path::PathBuf;

#[derive(Debug, clap::Args)]
pub(crate) struct ReleaseArgs {
    #[arg(long)]
    pub(crate) archive: PathBuf,
    #[arg(long)]
    pub(crate) manifest: PathBuf,
    #[arg(long)]
    pub(crate) envelope: PathBuf,
    #[arg(long)]
    pub(crate) trusted_public_key: PathBuf,
    #[arg(long)]
    pub(crate) key_id: String,
}

#[derive(Debug, clap::Args)]
pub(crate) struct ManifestPairArgs {
    #[arg(long)]
    pub(crate) manifest: PathBuf,
    #[arg(long)]
    pub(crate) envelope: PathBuf,
    #[arg(long)]
    pub(crate) trusted_public_key: PathBuf,
    #[arg(long)]
    pub(crate) key_id: String,
}

impl From<ReleaseArgs> for install::Inputs {
    fn from(value: ReleaseArgs) -> Self {
        Self {
            archive: value.archive,
            manifest: value.manifest,
            envelope: value.envelope,
            trusted_key: value.trusted_public_key,
            key_id: value.key_id,
        }
    }
}
