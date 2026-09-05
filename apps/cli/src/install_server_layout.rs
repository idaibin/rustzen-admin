use crate::install_server_release::ServerRelease;
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::PathBuf,
};
const ROOT: &str = "/opt/rz";

pub(super) fn make_payload_executable(release: &ServerRelease) -> Result<(), String> {
    for path in [
        PathBuf::from(ROOT),
        PathBuf::from(ROOT).join("releases"),
        PathBuf::from(ROOT).join("releases").join(&release.build_id),
        PathBuf::from(ROOT).join("releases").join(&release.build_id).join("payload"),
        PathBuf::from(ROOT).join("releases").join(&release.build_id).join("payload/bin"),
    ] {
        let meta =
            fs::symlink_metadata(&path).map_err(|_| "selected payload directory is unavailable")?;
        if meta.file_type().is_symlink() || !meta.is_dir() || meta.uid() != 0 {
            return Err("selected payload directory is unsafe".into());
        }
        let value = std::ffi::CString::new(path.as_os_str().as_bytes())
            .map_err(|_| "selected payload directory is invalid")?;
        if unsafe { libc::chmod(value.as_ptr(), 0o755) } != 0 {
            return Err("selected payload directory permissions could not be set".into());
        }
    }
    Ok(())
}
