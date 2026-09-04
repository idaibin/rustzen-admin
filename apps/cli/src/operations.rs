use reqwest::Client;
use rustzen_ipc::HealthResponse;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    env, fs,
    net::IpAddr,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_HEALTH_BODY_BYTES: usize = 16 * 1024;
const MODULES: [Module; 4] = [Module::Admin, Module::Monitor, Module::Insights, Module::Reports];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Module {
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
pub(crate) struct Status {
    pub(crate) module: Module,
    pub(crate) reachable: bool,
    pub(crate) state: &'static str,
    pub(crate) url: String,
    pub(crate) release_version: Option<String>,
    pub(crate) latency_ms: Option<u128>,
    pub(crate) error_code: Option<&'static str>,
}

#[derive(Debug)]
pub(crate) struct Context {
    pub(crate) runtime_root: PathBuf,
    pub(crate) config_path: PathBuf,
    pub(crate) release_bin_dir: PathBuf,
    pub(crate) installed_release: Option<String>,
    values: BTreeMap<String, String>,
}
impl Context {
    pub(crate) fn discover() -> Self {
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
        let port = match self
            .values
            .get(&format!("RUSTZEN_{}_PORT", module.name().to_ascii_uppercase()))
        {
            Some(value) => value.parse::<u16>().map_err(|_| "invalid_port")?,
            None => module.default_port(),
        };
        if port == 0 {
            return Err("invalid_port");
        }
        let host = match host.parse::<IpAddr>() {
            Ok(IpAddr::V6(_)) => format!("[{host}]"),
            _ => host,
        };
        Ok(format!("http://{host}:{port}/health"))
    }
}

pub(crate) const fn module_count() -> usize {
    MODULES.len()
}
pub(crate) async fn read_statuses(context: &Context, module: Module) -> Vec<Status> {
    let Ok(client) = Client::builder().timeout(REQUEST_TIMEOUT).build() else {
        return selected_modules(module)
            .map(|module| unavailable(context, module, "client_initialization_failed"))
            .collect();
    };
    match module {
        Module::All => {
            let (admin, monitor, insights, reports) = tokio::join!(
                read_status(&client, context, Module::Admin),
                read_status(&client, context, Module::Monitor),
                read_status(&client, context, Module::Insights),
                read_status(&client, context, Module::Reports)
            );
            vec![admin, monitor, insights, reports]
        }
        selected => vec![read_status(&client, context, selected).await],
    }
}
async fn read_status(client: &Client, context: &Context, module: Module) -> Status {
    let Ok(url) = context.endpoint(module) else {
        return unavailable(context, module, "invalid_endpoint");
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
    while let Some(chunk) = response.chunk().await.map_err(|_| "health_response_read_failed")? {
        if body.len().saturating_add(chunk.len()) > MAX_HEALTH_BODY_BYTES {
            return Err("health_response_too_large");
        }
        body.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&body).map_err(|_| "invalid_health_response")
}
fn unavailable(context: &Context, module: Module, code: &'static str) -> Status {
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
pub(crate) fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host.parse::<IpAddr>().is_ok_and(|address| address.is_loopback())
}
pub(crate) fn read_allowed_config(path: &Path) -> BTreeMap<String, String> {
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
