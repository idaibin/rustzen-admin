use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, clap::Args)]
pub(crate) struct AgentReleaseArgs {
    #[arg(long)]
    pub(crate) archive: PathBuf,
    #[arg(long)]
    pub(crate) manifest: PathBuf,
    #[arg(long)]
    pub(crate) envelope: PathBuf,
}

impl From<AgentReleaseArgs> for crate::install::Inputs {
    fn from(value: AgentReleaseArgs) -> Self {
        Self { archive: value.archive, manifest: value.manifest, envelope: value.envelope }
    }
}

#[derive(Debug, Parser)]
#[command(name = "rz", version, about = "Rustzen operations CLI")]
pub(super) struct Cli {
    /// Emit the stable JSON envelope on stdout.
    #[arg(long, global = true)]
    pub(super) json: bool,
    #[command(subcommand)]
    pub(super) command: Command,
}

#[derive(Debug, Subcommand)]
pub(super) enum Command {
    /// Check local installation layout and service health without reading secrets.
    Doctor,
    /// Show CLI, release-link, and service version information.
    Version,
    /// Start the complete Rustzen service set.
    Start,
    /// Stop the complete Rustzen service set.
    Stop,
    /// Restart the complete Rustzen service set.
    Restart,
    /// Show the complete Rustzen service-set status.
    Status,
    /// Verify and install a signed standalone Monitor Agent artifact.
    InstallAgent {
        #[command(flatten)]
        release: AgentReleaseArgs,
        #[arg(long)]
        destination: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// Pin the signed Controller identity for an installed Monitor Agent.
    PinMonitorController {
        /// Signed complete Rustzen server bundle.
        #[arg(long)]
        bundle: PathBuf,
        #[arg(long)]
        controller_endpoint: String,
    },
    /// Prepare /opt/rz access for the rz-monitor-agent service account.
    PrepareMonitorAgentAccess,
    /// Publish validated Agent configuration and start its native service.
    ActivateMonitorAgent {
        #[arg(long)]
        config: PathBuf,
    },
}
