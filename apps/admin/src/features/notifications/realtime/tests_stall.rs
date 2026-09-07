use super::test_support::*;
use crate::features::notifications::admission::AdmissionService;
use chrono::Utc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn wait_released(realtime: &super::RealtimeHub) {
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if realtime.counts(1) == (0, 0) && realtime.supervisor_count() == 0 {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn unpolled_body_hits_poll_deadline_releases_task_and_never_blocks_admission() {
    let database = Database::new().await;
    let now_epoch = Utc::now().timestamp();
    let (_, clock) = fixed_clock(now_epoch);
    let realtime = realtime_with_deadlines(
        database.pool.clone(),
        Duration::from_millis(60),
        Duration::from_secs(1),
        clock,
    );
    let codec = codec();
    let (_, token) = token(&database.pool, &codec, 1, "owner", now_epoch).await;
    let router = app(database.pool.clone(), realtime.clone(), codec);
    let held = request(router.clone(), Some(&token), "/stream").await;
    assert_eq!(realtime.counts(1), (1, 1));
    assert_eq!(realtime.supervisor_count(), 1);

    let service = AdmissionService::start_with_realtime(
        database.pool.clone(),
        database.path.clone(),
        policy(),
        realtime.clone(),
    )
    .await
    .unwrap();
    let now = Utc::now().naive_utc();
    tokio::time::timeout(
        Duration::from_millis(200),
        service.admit(&event(now, "unpolled-write", vec![1]), now),
    )
    .await
    .unwrap()
    .unwrap();
    wait_released(&realtime).await;
    assert_eq!(request(router, Some(&token), "/stream").await.status(), 200);
    drop(held);
    database.close().await;
}

async fn read_headers(socket: &mut tokio::net::TcpStream) -> Vec<u8> {
    let mut received = Vec::new();
    let mut buffer = [0_u8; 256];
    tokio::time::timeout(Duration::from_secs(1), async {
        while !received.windows(4).any(|window| window == b"\r\n\r\n") {
            let count = socket.read(&mut buffer).await.unwrap();
            assert!(count > 0, "socket closed before response headers");
            received.extend_from_slice(&buffer[..count]);
        }
    })
    .await
    .unwrap();
    received
}

#[tokio::test]
async fn tcp_client_that_stops_reading_is_closed_at_maximum_connection_age() {
    let database = Database::new().await;
    let now_epoch = Utc::now().timestamp();
    let (_, clock) = fixed_clock(now_epoch);
    let realtime = realtime_with_deadlines(
        database.pool.clone(),
        Duration::from_millis(60),
        Duration::from_millis(150),
        clock,
    );
    let codec = codec();
    let (_, token) = token(&database.pool, &codec, 1, "owner", now_epoch).await;
    let router = app(database.pool.clone(), realtime.clone(), codec);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    let mut socket = tokio::net::TcpStream::connect(address).await.unwrap();
    socket
        .write_all(
            format!(
                "GET /stream HTTP/1.1\r\nHost: {address}\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    let headers = read_headers(&mut socket).await;
    assert!(String::from_utf8_lossy(&headers).starts_with("HTTP/1.1 200"));
    assert_eq!(realtime.counts(1), (1, 1));

    tokio::time::sleep(Duration::from_millis(220)).await;
    wait_released(&realtime).await;
    let mut tail = Vec::new();
    tokio::time::timeout(Duration::from_secs(1), socket.read_to_end(&mut tail))
        .await
        .unwrap()
        .unwrap();
    let mut replacement = tokio::net::TcpStream::connect(address).await.unwrap();
    replacement
        .write_all(
            format!(
                "GET /stream HTTP/1.1\r\nHost: {address}\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    let headers = read_headers(&mut replacement).await;
    assert!(String::from_utf8_lossy(&headers).starts_with("HTTP/1.1 200"));
    drop(replacement);
    server.abort();
    let _ = server.await;
    wait_released(&realtime).await;
    database.close().await;
}
