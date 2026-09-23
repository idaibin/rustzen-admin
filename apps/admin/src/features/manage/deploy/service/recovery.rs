use super::*;

pub(super) fn load_installed_bundle(
    runtime_root: &Path,
    version: &str,
    arch: &str,
) -> Result<(Vec<u8>, BundleInfo), Box<dyn std::error::Error>> {
    let path = runtime_root.join("data/releases").join(format!("rz-{version}-{arch}.tar"));
    let data = fs::read(&path).map_err(|error| {
        std::io::Error::other(format!(
            "installed release bundle is unavailable at {}: {error}",
            path.display()
        ))
    })?;
    let bundle = validate_bundle(
        &data,
        version,
        CONFIG.deploy_signature_required,
        CONFIG.deploy_verify_key.as_deref(),
    )?;
    let expected_name = format!("rz-{version}-{}.tar", bundle.arch);
    if path.file_name().and_then(|value| value.to_str()) != Some(expected_name.as_str()) {
        return Err(std::io::Error::other("stored installed bundle architecture mismatch").into());
    }
    Ok((data, bundle))
}

pub(super) fn service_probes() -> [(&'static str, String); 4] {
    [
        ("rz-monitor.service", format!("{}/health", CONFIG.monitor_base_url())),
        ("rz-insights.service", format!("{}/health", CONFIG.insights_base_url())),
        ("rz-reports.service", format!("{}/health", CONFIG.reports_base_url())),
        ("rz-admin.service", format!("http://127.0.0.1:{}/health", CONFIG.admin_port())),
    ]
}

pub(super) async fn roll_services(
    journal: &mut UpdateJournal,
    journal_path: &Path,
    expected_version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().timeout(Duration::from_secs(2)).build()?;
    roll_services_with(journal, journal_path, |unit, url| {
        let client = client.clone();
        let expected_version = expected_version.to_string();
        async move { restart_and_verify(&unit, &url, &expected_version, &client).await }
    })
    .await
}

pub(super) async fn roll_services_with<F, Fut>(
    journal: &mut UpdateJournal,
    journal_path: &Path,
    mut restart: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(String, String) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error>>>,
{
    for (unit, url) in service_probes() {
        journal.stage = format!("restarting:{unit}");
        if !journal.restarted_units.iter().any(|value| value == unit) {
            journal.restarted_units.push(unit.to_string());
        }
        write_update_journal_at(journal_path, journal)?;
        restart(unit.to_string(), url).await?;
    }
    Ok(())
}

pub(super) async fn restart_and_verify(
    unit: &str,
    url: &str,
    expected_version: &str,
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    systemctl("stop", &[unit]).await?;
    systemctl("start", &[unit]).await?;
    ensure_unit_active(unit).await?;
    wait_for_health(client, url, expected_version).await
}

pub(super) async fn rollback_update(
    journal: &UpdateJournal,
) -> Result<(), Box<dyn std::error::Error>> {
    validate_rollback_release(journal)?;
    let units = journal.restarted_units.iter().map(String::as_str).collect::<Vec<_>>();
    if !units.is_empty() {
        systemctl("stop", &units).await?;
    }
    restore_release_state(journal, &units)?;
    daemon_reload().await?;
    let old_version = release_version_from_target(&journal.old_target)?;
    let client = reqwest::Client::builder().timeout(Duration::from_secs(2)).build()?;
    for (unit, url) in service_probes() {
        if units.contains(&unit) {
            systemctl("start", &[unit]).await?;
            ensure_unit_active(unit).await?;
            wait_for_health(&client, &url, old_version).await?;
        }
    }
    cleanup_failed_release(journal)?;
    Ok(())
}

pub(super) fn restore_release_state(
    journal: &UpdateJournal,
    units: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    restore_release_state_with_paths(journal, units, &database_paths())
}

pub(super) fn restore_release_state_with_paths(
    journal: &UpdateJournal,
    units: &[&str],
    paths: &[PathBuf; 4],
) -> Result<(), Box<dyn std::error::Error>> {
    swap_symlink(&journal.link, &journal.old_target)?;
    restore_databases_for_units_at(&journal.backup_dir, units, paths)
}

pub(super) fn cleanup_failed_release(journal: &UpdateJournal) -> Result<(), std::io::Error> {
    if journal.installed_by_update
        && journal.stage != "backedUp"
        && journal.new_release_dir.exists()
    {
        fs::remove_dir_all(&journal.new_release_dir)?;
    }
    if journal.install_staging_dir.exists() {
        fs::remove_dir_all(&journal.install_staging_dir)?;
    }
    Ok(())
}

pub(super) async fn systemctl(
    action: &str,
    units: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    if !matches!(action, "start" | "stop") || units.iter().any(|unit| !SYSTEMD_UNITS.contains(unit))
    {
        return Err(std::io::Error::other("invalid fixed service operation").into());
    }
    let status = Command::new("systemctl").arg(action).args(units).status().await?;
    if !status.success() {
        return Err(std::io::Error::other(format!("systemctl {action} failed")).into());
    }
    Ok(())
}

pub(super) async fn requeue_services_after_recovery() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("systemctl")
        .args(["--no-block", "start"])
        .args(SYSTEMD_UNITS)
        .status()
        .await?;
    if !status.success() {
        return Err(std::io::Error::other(
            "systemctl failed to requeue services after release recovery",
        )
        .into());
    }
    Ok(())
}

pub(super) async fn daemon_reload() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("systemctl").arg("daemon-reload").status().await?;
    if !status.success() {
        return Err(std::io::Error::other("systemctl daemon-reload failed").into());
    }
    Ok(())
}

pub(super) async fn ensure_unit_active(unit: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !SYSTEMD_UNITS.contains(&unit) {
        return Err(std::io::Error::other("invalid fixed service operation").into());
    }
    let status = Command::new("systemctl").args(["is-active", "--quiet", unit]).status().await?;
    if !status.success() {
        return Err(std::io::Error::other(format!("systemd unit is not active: {unit}")).into());
    }
    Ok(())
}

pub(super) async fn wait_for_health(
    client: &reqwest::Client,
    url: &str,
    expected_version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let probe = async {
        loop {
            if let Ok(response) = client.get(url).send().await
                && response.status().is_success()
                && let Ok(body) = response.json::<HealthResponse>().await
                && body.status == "ok"
                && body.release_version == expected_version
            {
                return;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    };
    if tokio::time::timeout(Duration::from_secs(30), probe).await.is_ok() {
        return Ok(());
    }
    Err(std::io::Error::other(format!("health gate failed for release {expected_version}: {url}"))
        .into())
}

pub(super) fn release_version_from_target(
    target: &Path,
) -> Result<&str, Box<dyn std::error::Error>> {
    target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| std::io::Error::other("current release target has no version").into())
}
