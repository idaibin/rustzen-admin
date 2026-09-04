use std::{
    collections::BTreeMap,
    env, fs,
    net::IpAddr,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

use clap::{Parser, Subcommand, ValueEnum};
use reqwest::Client;
use rustzen_ipc::HealthResponse;
use serde::Serialize;
use serde_json::{Value, json};

const SCHEMA_VERSION: u8 = 1;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_HEALTH_BODY_BYTES: usize = 16 * 1024;
const MODULES: [Module; 4] = [Module::Admin, Module::Monitor, Module::Insights, Module::Reports];

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
#[command(name = "rz", version, about = "Read-only operations CLI for a Rustzen installation")]
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
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
enum Module {
    All,
    Admin,
    Monitor,
    Insights,
    Reports,
}

impl Module {
    fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Admin => "admin",
            Self::Monitor => "monitor",
            Self::Insights => "insights",
            Self::Reports => "reports",
        }
    }

    fn default_port(self) -> u16 {
        match self {
            Self::All => 0,
            Self::Admin => 9801,
            Self::Monitor => 9802,
            Self::Insights => 9803,
            Self::Reports => 9804,
        }
    }
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

#[derive(Debug, Serialize)]
struct Status {
    module: Module,
    reachable: bool,
    state: &'static str,
    url: String,
    release_version: Option<String>,
    latency_ms: Option<u128>,
    error_code: Option<&'static str>,
}

#[derive(Debug)]
struct Context {
    runtime_root: PathBuf,
    config_path: PathBuf,
    release_bin_dir: PathBuf,
    installed_release: Option<String>,
    values: BTreeMap<String, String>,
}

impl Context {
    fn discover() -> Self {
        let executable = env::current_exe().ok();
        let installed_root = executable.as_deref().and_then(installed_root_from_executable);
        let runtime_root = env::var_os("RUSTZEN_RUNTIME_ROOT")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| installed_root.clone())
            .unwrap_or_else(|| PathBuf::from(".rustzen-admin"));
        let config_path = runtime_root.join("config/rz.env");
        let mut values = read_allowed_config(&config_path);
        for name in [
            "RUSTZEN_ADMIN_PORT",
            "RUSTZEN_INTERNAL_HOST",
            "RUSTZEN_MONITOR_PORT",
            "RUSTZEN_INSIGHTS_PORT",
            "RUSTZEN_REPORTS_PORT",
        ] {
            if let Ok(value) = env::var(name)
                && !value.trim().is_empty()
            {
                values.insert(name.to_string(), value);
            }
        }
        let current = runtime_root.join("current");
        let installed_release =
            fs::read_link(&current).ok().and_then(|target| release_name(&target));
        let release_bin_dir = if current.exists() {
            current.join("bin")
        } else {
            executable
                .as_deref()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_else(|| runtime_root.join("bin"))
        };
        Self { runtime_root, config_path, release_bin_dir, installed_release, values }
    }

    fn endpoint(&self, module: Module) -> Result<String, &'static str> {
        let host = if module == Module::Admin {
            "127.0.0.1".to_string()
        } else {
            self.values
                .get("RUSTZEN_INTERNAL_HOST")
                .cloned()
                .unwrap_or_else(|| "127.0.0.1".to_string())
        };
        if !is_loopback_host(&host) {
            return Err("non_loopback_endpoint");
        }
        let port_name = format!("RUSTZEN_{}_PORT", module.name().to_ascii_uppercase());
        let port = match self.values.get(&port_name) {
            Some(value) => value.parse::<u16>().map_err(|_| "invalid_port")?,
            None => module.default_port(),
        };
        if port == 0 {
            return Err("invalid_port");
        }
        let url_host = match host.parse::<IpAddr>() {
            Ok(IpAddr::V6(_)) => format!("[{host}]"),
            _ => host,
        };
        Ok(format!("http://{url_host}:{port}/health"))
    }
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
                    "expected": MODULES.len(),
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
    }
    Ok(())
}

async fn read_statuses(context: &Context, module: Module) -> Vec<Status> {
    let client = Client::builder().timeout(REQUEST_TIMEOUT).build();
    let Ok(client) = client else {
        return selected_modules(module)
            .map(|module| unavailable_status(context, module, "client_initialization_failed"))
            .collect();
    };
    match module {
        Module::All => {
            let (admin, monitor, insights, reports) = tokio::join!(
                read_status(&client, context, Module::Admin),
                read_status(&client, context, Module::Monitor),
                read_status(&client, context, Module::Insights),
                read_status(&client, context, Module::Reports),
            );
            vec![admin, monitor, insights, reports]
        }
        selected => vec![read_status(&client, context, selected).await],
    }
}

async fn read_status(client: &Client, context: &Context, module: Module) -> Status {
    let Ok(url) = context.endpoint(module) else {
        return unavailable_status(context, module, "invalid_endpoint");
    };
    let started = Instant::now();
    match client.get(&url).send().await {
        Ok(response) if response.status().is_success() => {
            match read_health_response(response).await {
                Ok(health) if health.status == "ok" => Status {
                    module,
                    reachable: true,
                    state: "healthy",
                    url,
                    release_version: Some(health.release_version),
                    latency_ms: Some(started.elapsed().as_millis()),
                    error_code: None,
                },
                Ok(_) => Status {
                    module,
                    reachable: true,
                    state: "degraded",
                    url,
                    release_version: None,
                    latency_ms: Some(started.elapsed().as_millis()),
                    error_code: Some("unexpected_health_status"),
                },
                Err(error_code) => Status {
                    module,
                    reachable: true,
                    state: "degraded",
                    url,
                    release_version: None,
                    latency_ms: Some(started.elapsed().as_millis()),
                    error_code: Some(error_code),
                },
            }
        }
        Ok(_) => Status {
            module,
            reachable: false,
            state: "unavailable",
            url,
            release_version: None,
            latency_ms: Some(started.elapsed().as_millis()),
            error_code: Some("http_error"),
        },
        Err(error) => Status {
            module,
            reachable: false,
            state: "unavailable",
            url,
            release_version: None,
            latency_ms: Some(started.elapsed().as_millis()),
            error_code: Some(if error.is_timeout() { "timeout" } else { "connection_failed" }),
        },
    }
}

async fn read_health_response(
    mut response: reqwest::Response,
) -> Result<HealthResponse, &'static str> {
    if response.content_length().is_some_and(|length| length > MAX_HEALTH_BODY_BYTES as u64) {
        return Err("health_response_too_large");
    }
    let mut body = Vec::new();
    loop {
        let chunk = response.chunk().await.map_err(|_| "health_response_read_failed")?;
        let Some(chunk) = chunk else {
            break;
        };
        if body.len().saturating_add(chunk.len()) > MAX_HEALTH_BODY_BYTES {
            return Err("health_response_too_large");
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(|_| "invalid_health_response")
}

fn unavailable_status(context: &Context, module: Module, code: &'static str) -> Status {
    Status {
        module,
        reachable: false,
        state: "unavailable",
        url: context.endpoint(module).unwrap_or_else(|_| "unavailable".to_string()),
        release_version: None,
        latency_ms: None,
        error_code: Some(code),
    }
}

fn selected_modules(module: Module) -> impl Iterator<Item = Module> {
    MODULES.into_iter().filter(move |candidate| module == Module::All || *candidate == module)
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

fn installed_root_from_executable(executable: &Path) -> Option<PathBuf> {
    let bin = executable.parent()?;
    if bin.file_name()? != "bin" {
        return None;
    }
    let release = bin.parent()?;
    let releases = release.parent()?;
    if releases.file_name()? != "releases" {
        return None;
    }
    releases.parent().map(Path::to_path_buf)
}

fn release_name(target: &Path) -> Option<String> {
    target.file_name()?.to_str().filter(|value| !value.is_empty()).map(str::to_string)
}

fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host.parse::<IpAddr>().is_ok_and(|address| address.is_loopback())
}

fn read_allowed_config(path: &Path) -> BTreeMap<String, String> {
    const ALLOWED: [&str; 5] = [
        "RUSTZEN_ADMIN_PORT",
        "RUSTZEN_INTERNAL_HOST",
        "RUSTZEN_MONITOR_PORT",
        "RUSTZEN_INSIGHTS_PORT",
        "RUSTZEN_REPORTS_PORT",
    ];
    let Ok(contents) = fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    contents
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (name, value) = line.split_once('=')?;
            ALLOWED.contains(&name).then(|| (name.to_string(), value.trim().to_string()))
        })
        .collect()
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
        fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("rz.env");
        fs::write(
            &path,
            "RUSTZEN_INTERNAL_HOST=127.0.0.9\nRUSTZEN_ADMIN_PORT=19001\nRUSTZEN_JWT_SECRET=secret\nRUSTZEN_IPC_TOKEN=secret\n",
        )
        .expect("fixture config");
        let values = read_allowed_config(&path);
        assert_eq!(values.get("RUSTZEN_INTERNAL_HOST").map(String::as_str), Some("127.0.0.9"));
        assert_eq!(values.get("RUSTZEN_ADMIN_PORT").map(String::as_str), Some("19001"));
        assert!(!values.keys().any(|key| key.contains("SECRET") || key.contains("TOKEN")));
        fs::remove_file(path).expect("remove fixture");
        fs::remove_dir(directory).expect("remove temporary directory");
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
