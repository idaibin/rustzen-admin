use crate::protocol::{
    AGENT_REPORT_AUTH_HEADER, AGENT_REPORT_METHOD, AgentReport, AgentReportStatus,
    next_agent_sequence, parse_agent_response,
};
use chrono::{DateTime, Utc};
use reqwest::Client;
use std::{
    collections::HashMap,
    future::Future,
    path::{Path as FsPath, PathBuf},
    time::Duration,
};
use sysinfo::{Disks, System};
use uuid::Uuid;

async fn run_agent_loop<C, S, R, SendFut, ReadyFut>(
    mut interval: tokio::time::Interval,
    initial_sequence: u64,
    cycle_limit: Option<usize>,
    mut collector: C,
    mut sender: S,
    mut report_ready: R,
) -> Result<u64, String>
where
    C: FnMut(u64) -> AgentReport,
    S: FnMut(AgentReport) -> SendFut,
    R: FnMut() -> ReadyFut,
    SendFut: Future<Output = Result<AgentReportStatus, String>>,
    ReadyFut: Future<Output = Result<(), String>>,
{
    let mut sequence = initial_sequence;
    let mut cycles = 0;
    let mut readiness_reported = false;
    loop {
        if cycle_limit.is_some_and(|limit| cycles >= limit) {
            return Ok(sequence);
        }
        interval.tick().await;
        let result = sender(collector(sequence)).await;
        let delivery_confirmed =
            matches!(result, Ok(AgentReportStatus::Accepted | AgentReportStatus::Duplicate));
        match &result {
            Ok(AgentReportStatus::Accepted | AgentReportStatus::Duplicate) => {
                tracing::debug!(sequence, ?result, "Monitor Agent report accepted");
            }
            Ok(AgentReportStatus::Stale) => {
                tracing::warn!(sequence, "Monitor Agent report was stale; retrying sequence");
            }
            Err(error) => {
                tracing::error!(sequence, %error, "Monitor Agent report failed; retaining sequence");
            }
        }
        if delivery_confirmed && !readiness_reported {
            match report_ready().await {
                Ok(()) => {
                    readiness_reported = true;
                    tracing::info!(sequence, "Monitor Agent is ready after Controller delivery");
                }
                Err(error) => {
                    tracing::warn!(sequence, %error, "Monitor Agent readiness notification failed");
                }
            }
        }
        sequence = next_agent_sequence(
            sequence,
            result.as_ref().map(|value| *value).map_err(|error| error.as_str()),
        )
        .ok_or_else(|| "monitor agent sequence exhausted".to_string())?;
        cycles += 1;
    }
}

pub async fn run_agent(
    node_id: String,
    endpoint: String,
    token: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let boot = Uuid::new_v4();
    let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
    let mut sys = System::new_all();
    let mut disks = Disks::new_with_refreshed_list();
    sys.refresh_all();
    disks.refresh(true);
    tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
    let collector_node_id = node_id.clone();
    let collector = move |sequence| {
        sys.refresh_all();
        disks.refresh(true);
        let cpu_percent = sys.global_cpu_usage() as f64;
        let memory = crate::protocol::ByteUsage {
            used_bytes: sys.used_memory(),
            total_bytes: sys.total_memory(),
        };
        let hostname = System::host_name().unwrap_or_else(|| collector_node_id.clone());
        let disk_values = disks
            .list()
            .iter()
            .map(|disk| {
                (
                    disk.name().to_string_lossy().to_string(),
                    disk.mount_point().to_path_buf(),
                    disk.total_space(),
                    disk.available_space(),
                )
            })
            .collect::<Vec<_>>();
        collect_report(
            &collector_node_id,
            boot,
            sequence,
            hostname,
            env!("CARGO_PKG_VERSION"),
            Utc::now(),
            cpu_percent,
            memory,
            disk_values,
        )
    };
    let sender_client = client.clone();
    let sender_endpoint = endpoint.clone();
    let sender_token = token.clone();
    let sender = move |report| {
        let client = sender_client.clone();
        let endpoint = sender_endpoint.clone();
        let token = sender_token.clone();
        async move { send_report(&client, &endpoint, &token, &report).await }
    };
    run_agent_loop(report_interval(), 1, None, collector, sender, notify_systemd_ready).await?;
    Ok(())
}

async fn notify_systemd_ready() -> Result<(), String> {
    let Some(path) = std::env::var_os("NOTIFY_SOCKET") else {
        return Ok(());
    };
    notify_systemd_ready_at(PathBuf::from(path)).await
}

#[cfg(unix)]
async fn notify_systemd_ready_at(path: PathBuf) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    if let Some(name) = std::os::unix::ffi::OsStrExt::as_bytes(path.as_os_str()).strip_prefix(b"@")
    {
        return send_abstract_systemd_ready(name);
    }
    let socket = tokio::net::UnixDatagram::unbound()
        .map_err(|error| format!("cannot create readiness socket: {error}"))?;
    socket.connect(path).map_err(|error| format!("cannot connect readiness socket: {error}"))?;
    socket
        .send(b"READY=1")
        .await
        .map_err(|error| format!("cannot send readiness notification: {error}"))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn send_abstract_systemd_ready(name: &[u8]) -> Result<(), String> {
    use std::os::fd::RawFd;

    if name.is_empty() {
        return Err("systemd readiness socket name is invalid".into());
    }
    // SAFETY: the file descriptor is closed on every path below, and the sockaddr is fully
    // initialized before its bounded byte slice is passed to sendto.
    unsafe {
        let fd: RawFd = libc::socket(libc::AF_UNIX, libc::SOCK_DGRAM | libc::O_CLOEXEC, 0);
        if fd < 0 {
            return Err(format!(
                "cannot create readiness socket: {}",
                std::io::Error::last_os_error()
            ));
        }
        let mut address: libc::sockaddr_un = std::mem::zeroed();
        let max_name = address.sun_path.len() - 1;
        if name.len() > max_name {
            libc::close(fd);
            return Err("systemd readiness socket name is invalid".into());
        }
        address.sun_family = libc::AF_UNIX as libc::sa_family_t;
        let destination = address.sun_path.as_mut_ptr().cast::<u8>();
        std::ptr::copy_nonoverlapping(name.as_ptr(), destination.add(1), name.len());
        let address_len =
            (std::mem::offset_of!(libc::sockaddr_un, sun_path) + 1 + name.len()) as libc::socklen_t;
        let sent = libc::sendto(
            fd,
            b"READY=1".as_ptr().cast(),
            b"READY=1".len(),
            libc::MSG_NOSIGNAL,
            std::ptr::from_ref(&address).cast(),
            address_len,
        );
        let result = if sent == b"READY=1".len() as isize {
            Ok(())
        } else {
            Err(format!("cannot send readiness notification: {}", std::io::Error::last_os_error()))
        };
        libc::close(fd);
        result
    }
}

#[cfg(not(unix))]
async fn notify_systemd_ready_at(_: PathBuf) -> Result<(), String> {
    Err("system-service readiness notifications require a Unix socket".into())
}

fn report_interval() -> tokio::time::Interval {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval
}

#[expect(
    clippy::too_many_arguments,
    reason = "the collector maps one complete wire report without hiding fixed fields"
)]
fn collect_report(
    node_id: &str,
    boot_id: Uuid,
    sequence: u64,
    hostname: String,
    agent_version: &str,
    collected_at: DateTime<Utc>,
    cpu_percent: f64,
    memory: crate::protocol::ByteUsage,
    disks: impl IntoIterator<Item = (String, PathBuf, u64, u64)>,
) -> AgentReport {
    AgentReport {
        node_id: node_id.to_string(),
        boot_id,
        sequence,
        hostname,
        agent_version: agent_version.to_string(),
        collected_at,
        cpu_percent,
        memory,
        disks: compact_disk_usages(disks),
    }
}

fn compact_disk_usages(
    disks: impl IntoIterator<Item = (String, PathBuf, u64, u64)>,
) -> Vec<crate::protocol::DiskUsage> {
    let mut compact: Vec<crate::protocol::DiskUsage> = Vec::new();
    let mut mount_indices = HashMap::<String, usize>::new();
    let mut local_device_indices = HashMap::<String, usize>::new();

    for (name, path, total_bytes, available_bytes) in disks {
        let Some(usage) = disk_usage(&name, &path, total_bytes, available_bytes) else {
            continue;
        };
        if mount_indices.contains_key(&usage.mount_point) {
            continue;
        }

        let local_device = local_block_device_name(&name);
        if let Some(index) =
            local_device.as_ref().and_then(|device| local_device_indices.get(device)).copied()
        {
            if prefer_mount_point(&usage.mount_point, &compact[index].mount_point) {
                mount_indices.remove(&compact[index].mount_point);
                mount_indices.insert(usage.mount_point.clone(), index);
                compact[index] = usage;
            }
            continue;
        }

        let index = compact.len();
        mount_indices.insert(usage.mount_point.clone(), index);
        compact.push(usage);
        if let Some(device) = local_device {
            local_device_indices.insert(device, index);
        }
    }

    compact
}

fn disk_usage(
    name: &str,
    path: &FsPath,
    total_bytes: u64,
    available_bytes: u64,
) -> Option<crate::protocol::DiskUsage> {
    if total_bytes == 0 || available_bytes > total_bytes {
        return None;
    }
    let mount_point = path.to_string_lossy().trim().to_string();
    if !should_keep_metric_disk(name, &mount_point) {
        return None;
    }
    Some(crate::protocol::DiskUsage {
        mount_point,
        usage: crate::protocol::ByteUsage {
            used_bytes: total_bytes - available_bytes,
            total_bytes,
        },
    })
}

fn should_keep_metric_disk(name: &str, mount_point: &str) -> bool {
    !is_ephemeral_mount_point(mount_point) && !is_ephemeral_disk_name(name)
}

fn is_ephemeral_mount_point(mount_point: &str) -> bool {
    const PREFIXES: &[&str] = &["/proc", "/sys", "/dev", "/run", "/var/run", "/snap"];
    const CONTAINS: &[&str] = &[
        "/pods/",
        "/plugins/kubernetes.io/",
        "/docker/overlay2/",
        "/containerd/",
        "/containers/",
        "/overlay2/",
    ];

    mount_point.is_empty()
        || PREFIXES.iter().any(|prefix| path_has_prefix(mount_point, prefix))
        || CONTAINS.iter().any(|value| mount_point.contains(value))
}

fn path_has_prefix(path: &str, prefix: &str) -> bool {
    path == prefix || path.strip_prefix(prefix).is_some_and(|suffix| suffix.starts_with('/'))
}

fn is_ephemeral_disk_name(name: &str) -> bool {
    let name = name.trim().to_ascii_lowercase();
    const EXACT_NAMES: &[&str] = &[
        "overlay",
        "tmpfs",
        "devtmpfs",
        "proc",
        "sysfs",
        "cgroup",
        "cgroup2",
        "mqueue",
        "shm",
        "udev",
        "devfs",
        "none",
        "nsfs",
        "tracefs",
        "debugfs",
        "securityfs",
        "pstore",
        "bpf",
        "fusectl",
        "rpc_pipefs",
        "autofs",
        "configfs",
        "hugetlbfs",
        "binfmt_misc",
        "ramfs",
    ];

    EXACT_NAMES.iter().any(|value| name == *value)
        || name.starts_with("/dev/loop")
        || name.starts_with("loop")
}

fn local_block_device_name(name: &str) -> Option<String> {
    let name = name.trim();
    name.starts_with("/dev/").then(|| name.to_string())
}

fn prefer_mount_point(candidate: &str, current: &str) -> bool {
    fn rank(mount_point: &str) -> (u8, usize, &str) {
        (if mount_point == "/" { 0 } else { 1 }, mount_point.len(), mount_point)
    }

    rank(candidate) < rank(current)
}

async fn send_report(
    client: &Client,
    endpoint: &str,
    token: &str,
    report: &AgentReport,
) -> Result<AgentReportStatus, String> {
    let method = reqwest::Method::from_bytes(AGENT_REPORT_METHOD.as_bytes())
        .map_err(|error| format!("invalid shared Agent report method: {error}"))?;
    let response = client
        .request(method, endpoint)
        .header(AGENT_REPORT_AUTH_HEADER, token)
        .json(report)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let body = response.text().await.map_err(|error| error.to_string())?;
    parse_agent_response(status.is_success(), &body)
}

#[cfg(test)]
fn parse_report_response(
    status: reqwest::StatusCode,
    body: &str,
) -> Result<AgentReportStatus, String> {
    parse_agent_response(status.is_success(), body)
}

#[cfg(test)]
mod tests;
