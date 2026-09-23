use std::{collections::BTreeMap, env, path::Path, process::ExitCode};

use clap::Parser;
use serde_json::json;

mod cli_contract;
mod cli_output;
mod install;
mod install_activation;
mod install_activation_state;
mod install_admission;
mod install_archive;
mod install_continuation;
mod install_crypto;
mod install_fs;
mod install_manifest;
mod install_pairing;
mod install_selection;
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
        Command::Start => service_control("start", cli.json)?,
        Command::Stop => service_control("stop", cli.json)?,
        Command::Restart => service_control("restart", cli.json)?,
        Command::Status => service_control("status", cli.json)?,
        Command::InstallAgent { release, destination, dry_run } => {
            let verified =
                install::apply(&release.into(), &destination, dry_run).map_err(|message| {
                    CliError {
                        command: "install-agent".into(),
                        code: "agent_install_failed",
                        message,
                    }
                })?;
            emit(cli.json, "install-agent", json!({"dry_run": dry_run, "release": verified}));
        }
        Command::PinMonitorController { bundle, controller_endpoint } => {
            let result = install_pairing::pin(&install_pairing::PinInputs {
                bundle,
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

fn service_control(action: &str, json_output: bool) -> Result<(), CliError> {
    let systemctl = Path::new("/usr/bin/systemctl");
    if action == "status" {
        return service_status(systemctl, json_output);
    }
    if action != "status" && unsafe { libc::geteuid() } != 0 {
        return Err(CliError {
            command: action.into(),
            code: "root_required",
            message: "rz service control must be run as root".into(),
        });
    }
    let status = std::process::Command::new(systemctl)
        .arg(action)
        .arg("rz-full.service")
        .stdout(if json_output {
            std::process::Stdio::null()
        } else {
            std::process::Stdio::inherit()
        })
        .stderr(if json_output {
            std::process::Stdio::null()
        } else {
            std::process::Stdio::inherit()
        })
        .status()
        .map_err(|_| CliError {
            command: action.into(),
            code: "systemctl_unavailable",
            message: "could not start systemctl".into(),
        })?;
    if !status.success() {
        return Err(CliError {
            command: action.into(),
            code: "systemctl_failed",
            message: format!("systemctl {action} rz-full.service failed"),
        });
    }
    let services = if matches!(action, "start" | "restart") {
        Some(wait_for_active_services(
            systemctl,
            action,
            80,
            std::time::Duration::from_millis(250),
        )?)
    } else {
        None
    };
    emit(
        json_output,
        action,
        json!({
            "unit": "rz-full.service",
            "action": action,
            "services": services.map(service_state_json)
        }),
    );
    Ok(())
}

const FULL_SERVICE_UNITS: [&str; 4] =
    ["rz-admin.service", "rz-monitor.service", "rz-insights.service", "rz-reports.service"];

fn service_status(systemctl: &Path, json_output: bool) -> Result<(), CliError> {
    let services = read_service_states(systemctl)?;
    require_all_services_active("status", &services)?;
    emit(
        json_output,
        "status",
        json!({
            "unit": "rz-full.service",
            "services": service_state_json(services)
        }),
    );
    Ok(())
}

fn wait_for_active_services(
    systemctl: &Path,
    command: &str,
    attempts: usize,
    delay: std::time::Duration,
) -> Result<Vec<(String, String, bool)>, CliError> {
    let mut services = read_service_states(systemctl)?;
    for _ in 1..attempts {
        if services.iter().all(|(_, _, active)| *active) {
            return Ok(services);
        }
        std::thread::sleep(delay);
        services = read_service_states(systemctl)?;
    }
    require_all_services_active(command, &services)?;
    Ok(services)
}

fn require_all_services_active(
    command: &str,
    services: &[(String, String, bool)],
) -> Result<(), CliError> {
    let inactive = services
        .iter()
        .filter(|(_, _, active)| !active)
        .map(|(unit, state, _)| format!("{unit}={state}"))
        .collect::<Vec<_>>();
    if inactive.is_empty() {
        Ok(())
    } else {
        Err(CliError {
            command: command.into(),
            code: "service_unhealthy",
            message: format!("Rustzen services are not all active: {}", inactive.join(", ")),
        })
    }
}

fn service_state_json(services: Vec<(String, String, bool)>) -> Vec<serde_json::Value> {
    services
        .into_iter()
        .map(|(unit, state, active)| json!({ "unit": unit, "state": state, "active": active }))
        .collect()
}

fn read_service_states(systemctl: &Path) -> Result<Vec<(String, String, bool)>, CliError> {
    FULL_SERVICE_UNITS
        .into_iter()
        .map(|unit| {
            let output =
                std::process::Command::new(systemctl).arg("is-active").arg(unit).output().map_err(
                    |_| CliError {
                        command: "status".into(),
                        code: "systemctl_unavailable",
                        message: "could not start systemctl".into(),
                    },
                )?;
            let state = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let state = if state.is_empty() { "unknown".to_string() } else { state };
            Ok((unit.to_string(), state, output.status.success()))
        })
        .collect()
}

#[cfg(test)]
mod main_tests;
