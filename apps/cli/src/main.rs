use std::{collections::BTreeMap, env, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::{Value, json};

mod install;
mod install_activation;
mod install_activation_state;
mod install_admission;
mod install_cli;
mod install_crypto;
mod install_fs;
mod install_manifest;
mod install_pairing;
mod install_selection;
mod operations;
use install_cli::{ManifestPairArgs, ReleaseArgs};
use operations::{Context, Module, module_count, read_statuses};
#[cfg(test)]
use operations::{is_loopback_host, read_allowed_config};

const SCHEMA_VERSION: u8 = 1;

#[used]
#[unsafe(no_mangle)]
pub static RUSTZEN_RELEASE_MARKER: &str = concat!(
    "RUSTZEN_RELEASE_MARKER\n",
    "artifact=rz-bundle-member\n",
    "binary=rz\n",
    "version=",
    env!("CARGO_PKG_VERSION"),
    "\n",
);

#[derive(Debug, Parser)]
#[command(name = "rz", version, about = "Rustzen operations and fresh-root selected-release CLI")]
struct Cli {
    /// Emit the stable JSON envelope on stdout.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
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
}

#[derive(Debug, Serialize)]
struct Success<T: Serialize> {
    schema_version: u8,
    ok: bool,
    command: String,
    data: T,
}

#[derive(Debug, Serialize)]
struct Failure {
    schema_version: u8,
    ok: bool,
    command: String,
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = env::args_os().collect::<Vec<_>>();
    let json_requested = args.iter().any(|arg| arg == "--json");
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            if json_requested {
                print_json(&Failure {
                    schema_version: SCHEMA_VERSION,
                    ok: false,
                    command: "parse".to_string(),
                    error: ErrorBody { code: "invalid_arguments", message: error.to_string() },
                });
                return ExitCode::from(2);
            }
            let _ = error.print();
            return ExitCode::from(2);
        }
    };

    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            print_json(&Failure {
                schema_version: SCHEMA_VERSION,
                ok: false,
                command: error.command,
                error: ErrorBody { code: error.code, message: error.message },
            });
            ExitCode::from(1)
        }
    }
}

#[derive(Debug)]
struct CliError {
    command: String,
    code: &'static str,
    message: String,
}

async fn run(cli: Cli) -> Result<(), CliError> {
    let context = Context::discover();
    match cli.command {
        Command::Doctor => {
            let statuses = read_statuses(&context, Module::All).await;
            let binaries = ["rz", "rz-admin", "rz-monitor", "rz-insights", "rz-reports"]
                .into_iter()
                .map(|binary| (binary, context.release_bin_dir.join(binary).is_file()))
                .collect::<BTreeMap<_, _>>();
            let healthy = statuses.iter().filter(|status| status.reachable).count();
            let data = json!({
                "runtime_root": context.runtime_root,
                "config": {
                    "path": context.config_path,
                    "present": context.config_path.is_file(),
                    "read_policy": "endpoint_allowlist_only",
                    "secret_values_retained": false
                },
                "current_release": context.installed_release,
                "binaries": binaries,
                "health": {
                    "reachable": healthy,
                    "expected": module_count(),
                    "services": statuses
                }
            });
            emit(cli.json, "doctor", data);
        }
        Command::Version => {
            let data = json!({
                "cli_version": env!("CARGO_PKG_VERSION"),
                "installed_release": context.installed_release,
                "version_source": if context.installed_release.is_some() {
                    "current_symlink"
                } else {
                    "unavailable"
                }
            });
            emit(cli.json, "version", data);
        }
        Command::Status { module } => {
            let statuses = read_statuses(&context, module).await;
            emit(cli.json, "status", json!({ "selection": module, "services": statuses }));
        }
        Command::Verify(args) => {
            let verified = install::verify(&args.into()).map_err(|message| CliError {
                command: "verify".into(),
                code: "release_verification_failed",
                message,
            })?;
            emit(cli.json, "verify", json!(verified));
        }
        Command::Apply { release, destination, dry_run } => {
            let verified =
                install::apply(&release.into(), &destination, dry_run).map_err(|message| {
                    CliError { command: "apply".into(), code: "release_apply_failed", message }
                })?;
            emit(cli.json, "apply", json!({"dry_run": dry_run, "release": verified}));
        }
        Command::InstallStatus { destination } => {
            let result = install::status(&destination).map_err(|message| CliError {
                command: "install-status".into(),
                code: "install_status_failed",
                message,
            })?;
            emit(cli.json, "install-status", result);
        }
        Command::PinMonitorController { release, controller_endpoint } => {
            let result = install_pairing::pin(&install_pairing::PinInputs {
                manifest: release.manifest,
                envelope: release.envelope,
                trusted_key: release.trusted_public_key,
                key_id: release.key_id,
                endpoint: controller_endpoint,
            })
            .map_err(|message| CliError {
                command: "pin-monitor-controller".into(),
                code: "controller_pairing_failed",
                message,
            })?;
            emit(cli.json, "pin-monitor-controller", json!(result));
        }
        Command::PrepareMonitorAgentAccess => {
            install_pairing::prepare_access().map_err(|message| CliError {
                command: "prepare-monitor-agent-access".into(),
                code: "agent_access_prepare_failed",
                message,
            })?;
            emit(
                cli.json,
                "prepare-monitor-agent-access",
                json!({"root":"/opt/rz","identity":"rz-monitor-agent"}),
            );
        }
        Command::ActivateMonitorAgent { config } => {
            let result =
                install_activation::activate(&install_activation::ActivationInput { config })
                    .map_err(|message| CliError {
                        command: "activate-monitor-agent".into(),
                        code: "agent_activation_failed",
                        message,
                    })?;
            emit(cli.json, "activate-monitor-agent", json!(result));
        }
    }
    Ok(())
}

fn emit(json_output: bool, command: &str, data: Value) {
    let output =
        Success { schema_version: SCHEMA_VERSION, ok: true, command: command.to_string(), data };
    if json_output {
        print_json(&output);
    } else {
        println!("{}", human_output(command, &output.data));
    }
}

fn human_output(command: &str, data: &Value) -> String {
    match command {
        "version" => format!(
            "rz {} (installed release: {})",
            data["cli_version"].as_str().unwrap_or("unknown"),
            data["installed_release"].as_str().unwrap_or("unavailable")
        ),
        _ => serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_string()),
    }
}

fn print_json(value: &impl Serialize) {
    match serde_json::to_string(value) {
        Ok(output) => println!("{output}"),
        Err(_) => {
            let fallback = r#"{"schema_version":1,"ok":false,"command":"serialize","error":{"code":"serialization_failed","message":"failed to serialize output"}}"#;
            println!("{fallback}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_surface_uses_fixed_module_enum() {
        let cli = Cli::try_parse_from(["rz", "--json", "status", "reports"]).expect("valid CLI");
        assert!(cli.json);
        assert!(matches!(cli.command, Command::Status { module: Module::Reports }));
        assert!(Cli::try_parse_from(["rz", "status", "unknown"]).is_err());
    }

    #[test]
    fn success_and_error_envelopes_are_stable() {
        let success = serde_json::to_value(Success {
            schema_version: SCHEMA_VERSION,
            ok: true,
            command: "version".to_string(),
            data: json!({"cli_version": "0.5.0"}),
        })
        .expect("success JSON");
        assert_eq!(success["schema_version"], 1);
        assert_eq!(success["ok"], true);
        assert_eq!(success["command"], "version");
        assert!(success.get("data").is_some());

        let failure = serde_json::to_value(Failure {
            schema_version: SCHEMA_VERSION,
            ok: false,
            command: "parse".to_string(),
            error: ErrorBody { code: "invalid_arguments", message: "invalid".to_string() },
        })
        .expect("failure JSON");
        assert_eq!(failure["ok"], false);
        assert_eq!(failure["error"]["code"], "invalid_arguments");
    }

    #[test]
    fn config_reader_only_accepts_non_secret_endpoint_keys() {
        let directory = std::env::temp_dir().join(format!("rz-cli-{}", std::process::id()));
        std::fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("rz.env");
        std::fs::write(
            &path,
            "RUSTZEN_INTERNAL_HOST=127.0.0.9\nRUSTZEN_ADMIN_PORT=19001\nRUSTZEN_JWT_SECRET=secret\nRUSTZEN_IPC_TOKEN=secret\n",
        )
        .expect("fixture config");
        let values = read_allowed_config(&path);
        assert_eq!(values.get("RUSTZEN_INTERNAL_HOST").map(String::as_str), Some("127.0.0.9"));
        assert_eq!(values.get("RUSTZEN_ADMIN_PORT").map(String::as_str), Some("19001"));
        assert!(!values.keys().any(|key| key.contains("SECRET") || key.contains("TOKEN")));
        std::fs::remove_file(path).expect("remove fixture");
        std::fs::remove_dir(directory).expect("remove temporary directory");
    }

    #[test]
    fn endpoint_policy_rejects_non_loopback_reads() {
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("::1"));
        assert!(is_loopback_host("localhost"));
        assert!(!is_loopback_host("example.com"));
        assert!(!is_loopback_host("10.0.0.2"));
    }
}
