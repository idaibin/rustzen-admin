use crate::install_server_config::SourceConfig;
use std::{
    path::Path,
    process::{Command, Stdio},
};

pub(super) fn systemctl(args: &[&str]) -> Result<(), String> {
    #[cfg(debug_assertions)]
    let binary = std::env::var_os("RUSTZEN_SYSTEMCTL_RECORDER")
        .unwrap_or_else(|| "/usr/bin/systemctl".into());
    #[cfg(not(debug_assertions))]
    let binary = std::ffi::OsString::from("/usr/bin/systemctl");
    if Command::new(binary)
        .args(args)
        .stdout(Stdio::null())
        .status()
        .map_err(|_| "systemctl invocation failed")?
        .success()
    {
        Ok(())
    } else {
        Err("systemctl activation failed".into())
    }
}

pub(super) fn validate_selected_config(
    source: &SourceConfig,
    release: &crate::install_server_release::ServerRelease,
) -> Result<(), String> {
    for (binary, config) in [
        ("rz-admin", &source.admin),
        (release.selection.secondary_binary(), &source.secondary),
    ] {
        let mut command = Command::new(Path::new("/opt/rz/current/bin").join(binary));
        command.arg("validate-config").stdout(Stdio::null());
        for line in std::str::from_utf8(config).map_err(|_| "selected config is invalid")?.lines() {
            let (key, value) = line.split_once('=').ok_or("selected config is invalid")?;
            command.env(key, value);
        }
        if !command.status().map_err(|_| "selected config validation could not start")?.success() {
            return Err("selected config validation failed".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configuration_validation_keeps_child_stdout_private() {
        assert_eq!(
            Path::new("/opt/rz/current/bin").join("rz-admin"),
            Path::new("/opt/rz/current/bin/rz-admin")
        );
    }
}
