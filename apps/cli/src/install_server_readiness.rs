use std::{
    fs,
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    path::Path,
    process::Command,
    thread,
    time::Duration,
};

pub(super) fn wait(
    admin_port: u16,
    monitor_port: u16,
    build_id: &str,
    composition_id: &str,
) -> Result<(), String> {
    #[cfg(debug_assertions)]
    if std::env::var_os("RUSTZEN_SKIP_SERVER_READINESS").is_some() {
        return Ok(());
    }
    for _ in 0..30 {
        if health(admin_port, build_id, composition_id)
            && health(monitor_port, build_id, composition_id)
        {
            return Ok(());
        }
        thread::sleep(Duration::from_secs(1));
    }
    Err("selected server readiness failed".into())
}
fn health(port: u16, build_id: &str, composition_id: &str) -> bool {
    std::net::TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}").parse().unwrap(),
        Duration::from_secs(1),
    )
    .and_then(|mut stream| {
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;
        stream.write_all(b"GET /health HTTP/1.0\r\nHost: localhost\r\n\r\n")?;
        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        let (head, body) = response
            .split_once("\r\n\r\n")
            .ok_or_else(|| std::io::Error::other("malformed health response"))?;
        if !head.starts_with("HTTP/1.1 200") && !head.starts_with("HTTP/1.0 200") {
            return Ok(false);
        }
        let body: serde_json::Value = serde_json::from_str(body).map_err(std::io::Error::other)?;
        Ok(body.get("status").and_then(serde_json::Value::as_str) == Some("ok")
            && body.pointer("/selectedBinding/buildId").and_then(serde_json::Value::as_str)
                == Some(build_id)
            && body.pointer("/selectedBinding/compositionId").and_then(serde_json::Value::as_str)
                == Some(composition_id))
    })
    .unwrap_or(false)
}

pub(super) fn verify_service_process(unit: &str, binary: &str) -> Result<(), String> {
    let output = Command::new("/usr/bin/systemctl")
        .args(["show", "--property=ActiveState", "--property=MainPID", unit])
        .output()
        .map_err(|_| "systemd process identity is unavailable")?;
    if !output.status.success() {
        return Err("systemd process identity is unavailable".into());
    }
    let values =
        String::from_utf8(output.stdout).map_err(|_| "systemd process identity is invalid")?;
    if values.lines().find_map(|line| line.strip_prefix("ActiveState=")) != Some("active") {
        return Err("selected service is not active".into());
    }
    let pid = values
        .lines()
        .find_map(|line| line.strip_prefix("MainPID="))
        .ok_or("systemd MainPID is unavailable")?
        .parse::<u32>()
        .map_err(|_| "systemd MainPID is invalid")?;
    if pid == 0 {
        return Err("systemd MainPID is invalid".into());
    }
    let expected = fs::metadata(Path::new("/opt/rz").join("current/bin").join(binary))
        .map_err(|_| "selected binary identity is unavailable")?;
    let running = fs::metadata(format!("/proc/{pid}/exe"))
        .map_err(|_| "selected service process is unavailable")?;
    if expected.dev() != running.dev() || expected.ino() != running.ino() {
        return Err("selected service process differs from payload".into());
    }
    Ok(())
}
