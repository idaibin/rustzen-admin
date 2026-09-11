use crate::install_server_config::SourceConfig;
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

pub(super) struct ServiceIdentity {
    pub(super) uid: u32,
    pub(super) gid: u32,
}

enum IdentityPlan {
    Existing(ServiceIdentity),
    Create { name: &'static str, create_group: bool },
}

pub(super) fn ensure_pair(
    selection: &crate::install_server_selection::ServerSelection,
) -> Result<(ServiceIdentity, ServiceIdentity), String> {
    let secondary_name = match selection.secondary {
        "monitor" => "rz-monitor",
        "insights" => "rz-insights",
        _ => return Err("selected server preset is invalid".into()),
    };
    let admin = plan("rz-admin")?;
    let secondary = plan(secondary_name)?;
    apply(admin).and_then(|admin| apply(secondary).map(|secondary| (admin, secondary)))
}

fn plan(name: &'static str) -> Result<IdentityPlan, String> {
    let passwd =
        fs::read_to_string("/etc/passwd").map_err(|_| "selected service account is unavailable")?;
    let group =
        fs::read_to_string("/etc/group").map_err(|_| "selected service group is unavailable")?;
    let user_exists = passwd.lines().any(|line| line.split(':').next() == Some(name));
    let group_exists = group.lines().any(|line| line.split(':').next() == Some(name));
    if user_exists {
        if !group_exists {
            return Err("selected service group is unavailable".into());
        }
        validate_groups(&group, name)?;
        return inspect(name).map(IdentityPlan::Existing);
    }
    if group_exists {
        validate_groups(&group, name)?;
    }
    Ok(IdentityPlan::Create { name, create_group: !group_exists })
}

fn apply(plan: IdentityPlan) -> Result<ServiceIdentity, String> {
    match plan {
        IdentityPlan::Existing(identity) => Ok(identity),
        IdentityPlan::Create { name, create_group } => {
            if create_group {
                run("/usr/sbin/groupadd", &["--system", name], "service group creation failed")?;
            }
            run(
                "/usr/sbin/useradd",
                &[
                    "--system",
                    "--no-create-home",
                    "--home-dir",
                    "/nonexistent",
                    "--shell",
                    "/usr/sbin/nologin",
                    "--gid",
                    name,
                    name,
                ],
                "service account creation failed",
            )?;
            inspect(name)
        }
    }
}

fn inspect(name: &str) -> Result<ServiceIdentity, String> {
    let passwd =
        fs::read_to_string("/etc/passwd").map_err(|_| "selected service account is unavailable")?;
    let fields = passwd
        .lines()
        .find(|line| line.split(':').next() == Some(name))
        .map(|line| line.split(':').collect::<Vec<_>>())
        .ok_or("selected service account is unavailable")?;
    let group =
        fs::read_to_string("/etc/group").map_err(|_| "selected service group is unavailable")?;
    let group_gid = validate_groups(&group, name)?;
    let uid = fields
        .get(2)
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or("selected service account is invalid")?;
    let gid = fields
        .get(3)
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or("selected service account is invalid")?;
    if fields.len() < 7
        || uid == 0
        || gid == 0
        || gid != group_gid
        || fields[5] != "/nonexistent"
        || fields[6] != "/usr/sbin/nologin"
    {
        return Err("selected service account is invalid".into());
    }
    Ok(ServiceIdentity { uid, gid })
}

fn validate_groups(group: &str, name: &str) -> Result<u32, String> {
    let mut primary_gid = None;
    for line in group.lines() {
        let fields = line.split(':').collect::<Vec<_>>();
        if fields.len() < 4 {
            return Err("selected service group is invalid".into());
        }
        let members = fields[3].split(',').filter(|member| !member.is_empty()).collect::<Vec<_>>();
        if fields[0] == name {
            if !members.is_empty() {
                return Err("selected service group has additional members".into());
            }
            primary_gid = fields[2].parse::<u32>().ok().filter(|gid| *gid != 0);
        } else if members.contains(&name) {
            return Err("selected service account has supplementary groups".into());
        }
    }
    primary_gid.ok_or("selected service group is invalid".into())
}

fn run(binary: &str, args: &[&str], message: &str) -> Result<(), String> {
    if Command::new(binary)
        .args(args)
        .stdout(Stdio::null())
        .status()
        .map_err(|_| message)?
        .success()
    {
        Ok(())
    } else {
        Err(message.into())
    }
}

fn owner_command(
    source: &SourceConfig,
    verify: bool,
    database: Option<&Path>,
) -> Result<Command, String> {
    let mut command = Command::new("/usr/sbin/runuser");
    command.args([
        "-u",
        "rz-admin",
        "--",
        "/opt/rz/current/bin/rz-admin",
        if verify { "verify-owner" } else { "bootstrap-owner" },
    ]);
    for line in
        std::str::from_utf8(&source.admin).map_err(|_| "selected Admin config is invalid")?.lines()
    {
        let (key, value) = line.split_once('=').ok_or("selected Admin config is invalid")?;
        command.env(
            key,
            if key == "RUSTZEN_ADMIN_SQLITE_PATH" {
                database.and_then(Path::to_str).unwrap_or(value)
            } else {
                value
            },
        );
    }
    command.stdin(Stdio::piped());
    command.stdout(Stdio::null());
    Ok(command)
}

fn run_owner(source: &SourceConfig, verify: bool, database: Option<&Path>) -> Result<(), String> {
    let mut child = owner_command(source, verify, database)?
        .spawn()
        .map_err(|_| "owner command could not start")?;
    child
        .stdin
        .take()
        .ok_or("owner command input is unavailable")?
        .write_all(&source.owner_password)
        .map_err(|_| "owner command input failed")?;
    if child.wait().map_err(|_| "owner command failed")?.success() {
        Ok(())
    } else if verify {
        Err("owner credential differs from requested tuple".into())
    } else {
        Err("bootstrap owner command failed".into())
    }
}

pub(super) fn bootstrap_at(source: &SourceConfig, database: &Path) -> Result<(), String> {
    run_owner(source, false, Some(database))
}

pub(super) fn verify_owner(source: &SourceConfig) -> Result<(), String> {
    run_owner(source, true, None)
}
