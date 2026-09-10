use std::{collections::BTreeMap, env, process::ExitCode};

use clap::Parser;
use serde_json::json;

mod cli_contract;
mod cli_output;
mod install;
mod install_activation;
mod install_activation_state;
mod install_admission;
mod install_archive;
mod install_cli;
mod install_continuation;
mod install_crypto;
mod install_fs;
mod install_manifest;
mod install_pairing;
mod install_selection;
mod install_server_activation;
mod install_server_activation_journal;
mod install_server_activation_process;
mod install_server_activation_state;
mod install_server_config;
mod install_server_database;
mod install_server_identity;
mod install_server_layout;
mod install_server_notification_config;
mod install_server_readiness;
mod install_server_release;
mod install_service_parent;
mod install_terminal;
mod operations;
use cli_contract::{Cli, Command};
use cli_output::{ErrorBody, Failure, SCHEMA_VERSION, emit, print_json};
use operations::{Context, Module, module_count, read_statuses};
#[cfg(test)]
use operations::{is_loopback_host, read_allowed_config};

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
        Command::ActivateMonitorServer { config } => {
            let result =
                install_server_activation::activate(&install_server_activation::ActivationInput {
                    config,
                })
                .map_err(|message| CliError {
                    command: "activate-monitor-server".into(),
                    code: "monitor_server_activation_failed",
                    message,
                })?;
            emit(cli.json, "activate-monitor-server", json!(result));
        }
    }
    Ok(())
}

#[cfg(test)]
mod main_tests;
