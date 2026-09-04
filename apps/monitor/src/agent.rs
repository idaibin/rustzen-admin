use crate::protocol::{AgentReport, AgentReportStatus};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::{
    collections::HashMap,
    future::Future,
    path::{Path as FsPath, PathBuf},
    time::Duration,
};
use sysinfo::{Disks, System};
use uuid::Uuid;

const MONITOR_AGENT_TOKEN_HEADER: &str = "x-rustzen-monitor-agent-token";

#[derive(Debug, Deserialize)]
struct ReportResult {
    status: AgentReportStatus,
}
#[derive(Debug, Deserialize)]
struct ReportEnvelope {
    code: i32,
    message: String,
    data: ReportResult,
}

async fn run_agent_loop<C, S, Fut>(
    mut interval: tokio::time::Interval,
    initial_sequence: u64,
    cycle_limit: Option<usize>,
    mut collector: C,
    mut sender: S,
) -> Result<u64, String>
where
    C: FnMut(u64) -> AgentReport,
    S: FnMut(AgentReport) -> Fut,
    Fut: Future<Output = Result<AgentReportStatus, String>>,
{
    let mut sequence = initial_sequence;
    let mut cycles = 0;
    loop {
        if cycle_limit.is_some_and(|limit| cycles >= limit) {
            return Ok(sequence);
        }
        interval.tick().await;
        let result = sender(collector(sequence)).await;
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
        sequence = next_sequence(
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
    run_agent_loop(report_interval(), 1, None, collector, sender).await?;
    Ok(())
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
    let response = client
        .post(endpoint)
        .header(MONITOR_AGENT_TOKEN_HEADER, token)
        .json(report)
        .send()
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let body = response.text().await.map_err(|error| error.to_string())?;
    parse_report_response(status, &body)
}

fn parse_report_response(
    status: reqwest::StatusCode,
    body: &str,
) -> Result<AgentReportStatus, String> {
    if !status.is_success() {
        return Err(format!("controller returned HTTP {status}"));
    }
    let envelope: ReportEnvelope = serde_json::from_str(body)
        .map_err(|error| format!("invalid controller response: {error}"))?;
    if envelope.code != 0 || envelope.message != "Success" {
        return Err(format!("controller rejected report: {}", envelope.message));
    }
    Ok(envelope.data.status)
}

fn next_sequence(sequence: u64, result: Result<AgentReportStatus, &str>) -> Option<u64> {
    match result {
        Ok(AgentReportStatus::Accepted | AgentReportStatus::Duplicate)
            if sequence < i64::MAX as u64 =>
        {
            sequence.checked_add(1)
        }
        Ok(AgentReportStatus::Accepted | AgentReportStatus::Duplicate) => None,
        Ok(AgentReportStatus::Stale) | Err(_) => Some(sequence),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{ByteUsage, DiskUsage};
    use chrono::TimeZone;
    fn report(
        node: &str,
        boot_id: Uuid,
        sequence: u64,
        collected_at: DateTime<Utc>,
        cpu: f64,
        root: u64,
        data: u64,
    ) -> AgentReport {
        AgentReport {
            node_id: node.to_string(),
            boot_id,
            sequence,
            hostname: node.to_string(),
            agent_version: "test".to_string(),
            collected_at,
            cpu_percent: cpu,
            memory: ByteUsage { used_bytes: 50, total_bytes: 100 },
            disks: vec![
                DiskUsage {
                    mount_point: "/".to_string(),
                    usage: ByteUsage { used_bytes: root, total_bytes: 100 },
                },
                DiskUsage {
                    mount_point: "/data".to_string(),
                    usage: ByteUsage { used_bytes: data, total_bytes: 100 },
                },
            ],
        }
    }

    #[test]
    fn agent_sequence_advances_only_after_accepted_or_duplicate() {
        assert_eq!(next_sequence(1, Err("503")), Some(1));
        assert_eq!(next_sequence(1, Err("timeout")), Some(1));
        assert_eq!(next_sequence(1, Ok(AgentReportStatus::Stale)), Some(1));
        assert_eq!(next_sequence(1, Ok(AgentReportStatus::Accepted)), Some(2));
        assert_eq!(next_sequence(1, Ok(AgentReportStatus::Duplicate)), Some(2));
        assert_eq!(next_sequence(i64::MAX as u64, Ok(AgentReportStatus::Accepted)), None);
    }

    #[test]
    fn agent_response_parsing_keeps_503_and_success_semantics() {
        assert!(
            parse_report_response(reqwest::StatusCode::SERVICE_UNAVAILABLE, "not json").is_err()
        );
        assert_eq!(
            parse_report_response(
                reqwest::StatusCode::OK,
                r#"{"code":0,"message":"Success","data":{"status":"accepted"}}"#,
            ),
            Ok(AgentReportStatus::Accepted)
        );
        assert!(parse_report_response(reqwest::StatusCode::OK, "{}").is_err());
    }

    #[test]
    fn agent_collection_maps_fixed_values_and_skips_invalid_devices() {
        let boot = Uuid::new_v4();
        let collected = Utc.with_ymd_and_hms(2026, 9, 2, 4, 0, 0).unwrap();
        let value = collect_report(
            "fixture-node",
            boot,
            7,
            "fixture-host".into(),
            "fixture-agent",
            collected,
            37.5,
            ByteUsage { used_bytes: 4, total_bytes: 10 },
            [
                ("/dev/sda1".into(), std::path::PathBuf::from("/root-bind"), 100, 25),
                ("/dev/sda1".into(), std::path::PathBuf::from("/"), 100, 25),
                ("/dev/vdb1".into(), std::path::PathBuf::from("/srv/data"), 200, 50),
                ("/dev/vdb1".into(), std::path::PathBuf::from("/data"), 200, 50),
                ("/dev/vdb2".into(), std::path::PathBuf::from("/data"), 400, 100),
                (
                    "/dev/mapper/runtime-docker".into(),
                    std::path::PathBuf::from("/var/lib/docker"),
                    300,
                    75,
                ),
                (
                    "overlay".into(),
                    std::path::PathBuf::from("/var/lib/docker/overlay2/abc/merged"),
                    100,
                    10,
                ),
                ("tmpfs".into(), std::path::PathBuf::from("/run"), 100, 10),
                ("/dev/loop0".into(), std::path::PathBuf::from("/media/app"), 100, 10),
                ("unknown".into(), std::path::PathBuf::new(), 100, 10),
                ("/dev/zero".into(), std::path::PathBuf::from("/zero"), 0, 0),
                ("/dev/broken".into(), std::path::PathBuf::from("/broken"), 100, 101),
                ("nfs.example:/volume".into(), std::path::PathBuf::from("/mnt/primary"), 500, 100),
                ("nfs.example:/volume".into(), std::path::PathBuf::from("/mnt/backup"), 500, 100),
            ],
        );
        assert_eq!(value.node_id, "fixture-node");
        assert_eq!(value.boot_id, boot);
        assert_eq!(value.sequence, 7);
        assert_eq!(value.cpu_percent, 37.5);
        assert_eq!(value.memory, ByteUsage { used_bytes: 4, total_bytes: 10 });
        assert_eq!(value.disks.len(), 5);
        assert_eq!(value.disks[0].mount_point, "/");
        assert_eq!(value.disks[0].usage.used_bytes, 75);
        assert_eq!(value.disks[1].mount_point, "/data");
        assert_eq!(value.disks[1].usage.used_bytes, 150);
        assert_eq!(value.disks[2].mount_point, "/var/lib/docker");
        assert_eq!(value.disks[2].usage.used_bytes, 225);
        assert_eq!(value.disks[3].mount_point, "/mnt/primary");
        assert_eq!(value.disks[4].mount_point, "/mnt/backup");
        value.validate().expect("fixture collection is wire-valid");
    }

    #[test]
    fn agent_response_statuses_and_failures_are_distinct() {
        for (status, body, expected) in [
            (
                reqwest::StatusCode::OK,
                r#"{"code":0,"message":"Success","data":{"status":"accepted"}}"#,
                Ok(AgentReportStatus::Accepted),
            ),
            (
                reqwest::StatusCode::OK,
                r#"{"code":0,"message":"Success","data":{"status":"duplicate"}}"#,
                Ok(AgentReportStatus::Duplicate),
            ),
            (
                reqwest::StatusCode::OK,
                r#"{"code":0,"message":"Success","data":{"status":"stale"}}"#,
                Ok(AgentReportStatus::Stale),
            ),
        ] {
            assert_eq!(parse_report_response(status, body), expected);
        }
        for status in [reqwest::StatusCode::UNAUTHORIZED, reqwest::StatusCode::SERVICE_UNAVAILABLE]
        {
            assert!(parse_report_response(status, "{}").is_err());
        }
        assert!(parse_report_response(reqwest::StatusCode::OK, "not-json").is_err());
    }

    #[tokio::test]
    async fn agent_send_report_observes_request_timeout() {
        use tokio::io::AsyncReadExt;
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("timeout fixture connection");
            let mut request = [0_u8; 4096];
            let bytes_read = socket.read(&mut request).await.expect("timeout fixture request");
            assert!(bytes_read > 0, "timeout fixture received an empty request");
            // Hold the connection open without responding; only the client timeout can finish.
            std::future::pending::<()>().await;
            drop(socket);
        });
        let client = Client::builder().timeout(Duration::from_millis(20)).build().unwrap();
        let result = send_report(
            &client,
            &format!("http://{address}/"),
            "token",
            &report(
                "timeout-node",
                Uuid::new_v4(),
                1,
                Utc.with_ymd_and_hms(2026, 9, 2, 4, 0, 0).unwrap(),
                10.0,
                10,
                10,
            ),
        )
        .await;
        assert!(result.is_err());
        server.abort();
    }

    #[tokio::test(start_paused = true)]
    async fn agent_loop_keeps_sequence_and_skips_missed_ticks_for_all_send_outcomes() {
        use std::{
            collections::VecDeque,
            sync::{Arc, Mutex},
        };

        let mut outcomes = VecDeque::from([
            (5, Ok(AgentReportStatus::Accepted)),
            (75, Err("503".to_string())),
            (0, Err("401".to_string())),
            (5, Err("timeout".to_string())),
            (0, Ok(AgentReportStatus::Accepted)),
        ]);
        let calls = Arc::new(Mutex::new(Vec::new()));
        let calls_for_sender = calls.clone();
        let final_sequence = run_agent_loop(
            report_interval(),
            1,
            Some(5),
            |sequence| {
                report("agent-loop-node", Uuid::new_v4(), sequence, Utc::now(), 10.0, 10, 10)
            },
            move |report| {
                calls_for_sender
                    .lock()
                    .unwrap()
                    .push((tokio::time::Instant::now(), report.sequence));
                let (advance_by, outcome) = outcomes.pop_front().unwrap();
                async move {
                    tokio::time::advance(Duration::from_secs(advance_by)).await;
                    outcome
                }
            },
        )
        .await
        .unwrap();
        assert_eq!(final_sequence, 3);
        let calls = calls.lock().unwrap();
        assert_eq!(
            calls.iter().map(|(_, sequence)| *sequence).collect::<Vec<_>>(),
            [1, 2, 2, 2, 2]
        );
        assert_eq!(calls[1].0.duration_since(calls[0].0), Duration::from_secs(30));
        assert_eq!(calls[2].0.duration_since(calls[1].0), Duration::from_secs(75));
        assert_eq!(calls[3].0.duration_since(calls[2].0), Duration::from_secs(15));
        assert_eq!(calls[4].0.duration_since(calls[3].0), Duration::from_secs(30));
    }
}
