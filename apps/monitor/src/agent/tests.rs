use super::*;
use crate::protocol::{AGENT_REPORT_PATH, ByteUsage, DiskUsage};
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
    assert_eq!(next_agent_sequence(1, Err("503")), Some(1));
    assert_eq!(next_agent_sequence(1, Err("timeout")), Some(1));
    assert_eq!(next_agent_sequence(1, Ok(AgentReportStatus::Stale)), Some(1));
    assert_eq!(next_agent_sequence(1, Ok(AgentReportStatus::Accepted)), Some(2));
    assert_eq!(next_agent_sequence(1, Ok(AgentReportStatus::Duplicate)), Some(2));
    assert_eq!(next_agent_sequence(i64::MAX as u64, Ok(AgentReportStatus::Accepted)), None);
}

#[test]
fn agent_response_parsing_keeps_503_and_success_semantics() {
    assert!(parse_report_response(reqwest::StatusCode::SERVICE_UNAVAILABLE, "not json").is_err());
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
    for status in [reqwest::StatusCode::UNAUTHORIZED, reqwest::StatusCode::SERVICE_UNAVAILABLE] {
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

#[cfg(unix)]
#[tokio::test]
async fn agent_sends_systemd_ready_notification_to_the_configured_socket() {
    use tokio::{net::UnixDatagram, time::timeout};

    let path = std::path::PathBuf::from(format!("/tmp/rz-ready-{}.sock", Uuid::new_v4()));
    let receiver = UnixDatagram::bind(&path).unwrap();
    notify_systemd_ready_at(path.clone()).await.unwrap();
    let mut buffer = [0_u8; 64];
    let received =
        timeout(Duration::from_secs(1), receiver.recv(&mut buffer)).await.unwrap().unwrap();
    assert_eq!(&buffer[..received], b"READY=1");
    std::fs::remove_file(path).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn agent_sends_systemd_ready_notification_to_an_abstract_socket() {
    let name = vec![b'r'; 107];
    // SAFETY: the bound address and receive buffer are valid for the documented libc calls, and
    // the descriptor is closed before the test returns.
    unsafe {
        let fd = libc::socket(libc::AF_UNIX, libc::SOCK_DGRAM | libc::O_CLOEXEC, 0);
        assert!(fd >= 0, "cannot create abstract readiness receiver");
        let mut address: libc::sockaddr_un = std::mem::zeroed();
        address.sun_family = libc::AF_UNIX as libc::sa_family_t;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            address.sun_path.as_mut_ptr().cast::<u8>().add(1),
            name.len(),
        );
        let length =
            (std::mem::offset_of!(libc::sockaddr_un, sun_path) + 1 + name.len()) as libc::socklen_t;
        assert_eq!(libc::bind(fd, std::ptr::from_ref(&address).cast(), length), 0);

        send_abstract_systemd_ready(&name).unwrap();
        assert!(send_abstract_systemd_ready(&vec![b'x'; 108]).is_err());
        let mut buffer = [0_u8; 64];
        let received = libc::recv(fd, buffer.as_mut_ptr().cast(), buffer.len(), 0);
        libc::close(fd);
        assert_eq!(&buffer[..received as usize], b"READY=1");
    }
}

#[tokio::test]
async fn agent_send_report_uses_the_shared_http_contract() {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let expected = report(
        "loopback-node",
        Uuid::new_v4(),
        1,
        Utc.with_ymd_and_hms(2026, 9, 2, 4, 0, 0).unwrap(),
        10.0,
        10,
        10,
    );
    let expected_for_server = expected.clone();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 1024];
        let header_end = loop {
            let count = socket.read(&mut buffer).await.unwrap();
            assert_ne!(count, 0, "Agent closed before completing headers");
            bytes.extend_from_slice(&buffer[..count]);
            if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                break index + 4;
            }
        };
        let headers = std::str::from_utf8(&bytes[..header_end]).unwrap();
        let expected_request_line = format!("{AGENT_REPORT_METHOD} {AGENT_REPORT_PATH} HTTP/1.1");
        assert_eq!(headers.lines().next(), Some(expected_request_line.as_str()));
        assert!(headers.lines().any(|line| {
            line.eq_ignore_ascii_case(&format!("{AGENT_REPORT_AUTH_HEADER}: shared-token"))
        }));
        assert!(headers.lines().any(|line| {
            line.to_ascii_lowercase().starts_with("content-type: application/json")
        }));
        let content_length = headers
            .lines()
            .find_map(|line| line.strip_prefix("content-length: "))
            .or_else(|| headers.lines().find_map(|line| line.strip_prefix("Content-Length: ")))
            .unwrap()
            .parse::<usize>()
            .unwrap();
        while bytes.len() < header_end + content_length {
            let count = socket.read(&mut buffer).await.unwrap();
            assert_ne!(count, 0, "Agent closed before completing body");
            bytes.extend_from_slice(&buffer[..count]);
        }
        assert_eq!(
            serde_json::from_slice::<AgentReport>(&bytes[header_end..header_end + content_length])
                .unwrap(),
            expected_for_server
        );
        let body = serde_json::to_string(&crate::protocol::AgentResponseEnvelope {
            code: crate::protocol::RESPONSE_SUCCESS_CODE,
            message: crate::protocol::RESPONSE_SUCCESS_MESSAGE.to_string(),
            data: crate::protocol::AgentResponseData { status: AgentReportStatus::Accepted },
        })
        .unwrap();
        socket
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                        body.len(),
                        body
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
    });
    let client = Client::new();
    assert_eq!(
        send_report(
            &client,
            &format!("http://{address}{AGENT_REPORT_PATH}"),
            "shared-token",
            &expected,
        )
        .await
        .unwrap(),
        AgentReportStatus::Accepted
    );
    server.await.unwrap();
}

#[tokio::test]
async fn agent_send_report_distinguishes_real_duplicate_401_and_tls_failures() {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let duplicate_body = r#"{"code":0,"message":"Success","data":{"status":"duplicate"}}"#;
    let duplicate_response = format!(
        "HTTP/1.1 200 OK\r\nconnection: close\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{duplicate_body}",
        duplicate_body.len()
    );
    let server = tokio::spawn(async move {
        for response in [
            duplicate_response,
            "HTTP/1.1 401 Unauthorized\r\nconnection: close\r\ncontent-length: 0\r\n\r\n".into(),
        ] {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 1024];
            assert!(socket.read(&mut request).await.unwrap() > 0);
            socket.write_all(response.as_bytes()).await.unwrap();
        }
    });
    let client = Client::new();
    let report = report("real-response-node", Uuid::new_v4(), 1, Utc::now(), 10.0, 10, 10);
    assert_eq!(
        send_report(&client, &format!("http://{address}{AGENT_REPORT_PATH}"), "token", &report)
            .await,
        Ok(AgentReportStatus::Duplicate)
    );
    assert!(
        send_report(&client, &format!("http://{address}{AGENT_REPORT_PATH}"), "token", &report)
            .await
            .is_err()
    );
    server.await.unwrap();

    let tls_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let tls_address = tls_listener.local_addr().unwrap();
    let tls_fixture = tokio::spawn(async move {
        let (mut socket, _) = tls_listener.accept().await.unwrap();
        let mut client_hello = [0_u8; 1024];
        assert!(socket.read(&mut client_hello).await.unwrap() > 0);
    });
    assert!(
        send_report(
            &client,
            &format!("https://{tls_address}{AGENT_REPORT_PATH}"),
            "token",
            &report
        )
        .await
        .is_err()
    );
    tls_fixture.await.unwrap();
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
    let readiness = Arc::new(Mutex::new(Vec::new()));
    let readiness_for_callback = readiness.clone();
    let final_sequence = run_agent_loop(
        report_interval(),
        1,
        Some(5),
        |sequence| report("agent-loop-node", Uuid::new_v4(), sequence, Utc::now(), 10.0, 10, 10),
        move |report| {
            calls_for_sender.lock().unwrap().push((tokio::time::Instant::now(), report.sequence));
            let (advance_by, outcome) = outcomes.pop_front().unwrap();
            async move {
                tokio::time::advance(Duration::from_secs(advance_by)).await;
                outcome
            }
        },
        move || {
            readiness_for_callback.lock().unwrap().push(tokio::time::Instant::now());
            async { Ok(()) }
        },
    )
    .await
    .unwrap();
    assert_eq!(final_sequence, 3);
    let calls = calls.lock().unwrap();
    assert_eq!(calls.iter().map(|(_, sequence)| *sequence).collect::<Vec<_>>(), [1, 2, 2, 2, 2]);
    assert_eq!(calls[1].0.duration_since(calls[0].0), Duration::from_secs(30));
    assert_eq!(calls[2].0.duration_since(calls[1].0), Duration::from_secs(75));
    assert_eq!(calls[3].0.duration_since(calls[2].0), Duration::from_secs(15));
    assert_eq!(calls[4].0.duration_since(calls[3].0), Duration::from_secs(30));
    assert_eq!(readiness.lock().unwrap().len(), 1);
}

#[tokio::test(start_paused = true)]
async fn agent_loop_stays_unready_until_duplicate_confirms_delivery() {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    let mut outcomes = VecDeque::from([
        Err("401".to_string()),
        Err("network".to_string()),
        Ok(AgentReportStatus::Duplicate),
        Ok(AgentReportStatus::Accepted),
    ]);
    let sequences = Arc::new(Mutex::new(Vec::new()));
    let sequences_for_sender = sequences.clone();
    let readiness = Arc::new(Mutex::new(0));
    let readiness_for_callback = readiness.clone();
    run_agent_loop(
        report_interval(),
        1,
        Some(4),
        |sequence| report("ready-node", Uuid::new_v4(), sequence, Utc::now(), 10.0, 10, 10),
        move |report| {
            sequences_for_sender.lock().unwrap().push(report.sequence);
            let outcome = outcomes.pop_front().unwrap();
            async move { outcome }
        },
        move || {
            *readiness_for_callback.lock().unwrap() += 1;
            async { Ok(()) }
        },
    )
    .await
    .unwrap();

    assert_eq!(*sequences.lock().unwrap(), [1, 1, 1, 2]);
    assert_eq!(*readiness.lock().unwrap(), 1);
}
