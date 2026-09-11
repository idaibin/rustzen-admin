use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{
    install_cli::{ManifestPairArgs, ReleaseArgs},
    operations::Module,
};

#[derive(Debug, Parser)]
#[command(name = "rz", version, about = "Rustzen operations and fresh-root selected-release CLI")]
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
    /// Read health for all services or one fixed module.
    Status {
        #[arg(value_enum, default_value_t = Module::All)]
        module: Module,
    },
    /// Verify a detached selected-release triplet against an independent public key.
    Verify(ReleaseArgs),
    /// Verify and publish an immutable selected payload into a fresh Linux root; it is not runnable.
    Apply {
        #[command(flatten)]
        release: ReleaseArgs,
        #[arg(long)]
        destination: PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
    /// Show the publication marker; this does not establish a runnable installation.
    InstallStatus {
        #[arg(long)]
        destination: PathBuf,
    },
    /// Pin a signed Monitor Controller release to an already published Agent root.
    PinMonitorController {
        #[command(flatten)]
        release: ManifestPairArgs,
        #[arg(long)]
        controller_endpoint: String,
    },
    /// Prepare fixed /opt/rz access for the rz-monitor-agent service account.
    PrepareMonitorAgentAccess,
    /// Publish validated Agent configuration and activate its selected native unit.
    ActivateMonitorAgent {
        /// Root-only file containing the production Agent environment values.
        #[arg(long)]
        config: PathBuf,
    },
    /// Activate the signed Monitor server selection at the fixed /opt/rz root.
    ActivateMonitorServer {
        /// Root-only source file containing the Monitor server environment values.
        #[arg(long)]
        config: PathBuf,
    },
    /// Activate the signed Analytics server selection at the fixed /opt/rz root.
    ActivateAnalyticsServer {
        /// Root-only source file containing the Analytics server environment values.
        #[arg(long)]
        config: PathBuf,
    },
}
