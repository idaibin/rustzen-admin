use super::{
    invalid,
    validation::{detect_elf_arch, validate_path},
};
use crate::common::error::ServiceError;
use std::{
    collections::BTreeSet,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

pub fn installed_release_arch(release_dir: &Path) -> Result<&'static str, ServiceError> {
    let admin = release_dir.join("bin/rz-admin");
    let metadata = fs::symlink_metadata(&admin)
        .map_err(|_| invalid("Installed release Admin binary is unavailable"))?;
    if !metadata.file_type().is_file() {
        return Err(invalid("Installed release Admin binary must be a regular file"));
    }
    detect_elf_arch(
        &fs::read(admin).map_err(|_| invalid("Installed release Admin binary is unreadable"))?,
    )
}

#[cfg(unix)]
pub(super) fn installed_mode_matches(metadata: &fs::Metadata, expected: u32) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o777 == expected & 0o777
}

#[cfg(not(unix))]
pub(super) fn installed_mode_matches(_metadata: &fs::Metadata, _expected: u32) -> bool {
    true
}

pub(super) fn collect_installed_files(
    root: &Path,
    directory: &Path,
    files: &mut BTreeSet<PathBuf>,
) -> Result<(), ServiceError> {
    for entry in fs::read_dir(directory).map_err(|_| invalid("Installed release is unreadable"))? {
        let entry = entry.map_err(|_| invalid("Installed release is unreadable"))?;
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|_| invalid("Installed release is unreadable"))?;
        if metadata.file_type().is_symlink() {
            return Err(invalid("Installed release must not contain symlinks"));
        }
        if metadata.is_dir() {
            collect_installed_files(root, &entry.path(), files)?;
        } else if metadata.is_file() {
            files.insert(
                entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|_| invalid("Installed release path is invalid"))?
                    .to_path_buf(),
            );
        } else {
            return Err(invalid("Installed release contains a non-regular member"));
        }
    }
    Ok(())
}
pub(super) fn extract_archive(
    content: &[u8],
    version: &str,
    arch: &str,
    staging: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = format!("rz-{version}-{arch}");
    let mut archive = tar::Archive::new(Cursor::new(content));
    for entry in archive.entries()? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path()?.into_owned();
        validate_path(&path).map_err(|error| std::io::Error::other(error.to_string()))?;
        let relative = path.strip_prefix(&root)?;
        let destination = staging.join(relative);
        let parent = destination
            .parent()
            .ok_or_else(|| std::io::Error::other("invalid bundle destination"))?;
        fs::create_dir_all(parent)?;
        let mut output = fs::OpenOptions::new().write(true).create_new(true).open(&destination)?;
        std::io::copy(&mut entry, &mut output)?;
        set_mode(&destination, entry.header().mode()?)?;
        output.sync_all()?;
    }
    Ok(())
}

#[cfg(unix)]
pub(super) fn normalize_release_directory_modes(release: &Path) -> Result<(), std::io::Error> {
    for (directory, mode) in [
        (release.to_path_buf(), 0o755),
        (release.join("bin"), 0o755),
        (release.join("systemd"), 0o755),
        (release.join("identity"), 0o755),
        (release.join("config"), 0o700),
    ] {
        if !fs::symlink_metadata(&directory)?.file_type().is_dir() {
            return Err(std::io::Error::other("release member directory is invalid"));
        }
        set_mode(&directory, mode)?;
    }
    Ok(())
}

#[cfg(not(unix))]
pub(super) fn normalize_release_directory_modes(_release: &Path) -> Result<(), std::io::Error> {
    Ok(())
}

#[cfg(unix)]
pub(super) fn normalize_release_config_ownership(
    release: &Path,
    runtime_root: &Path,
) -> Result<(), std::io::Error> {
    use std::os::unix::fs::MetadataExt;

    let owner = fs::symlink_metadata(runtime_root.join("data/db/admin"))?;
    if !owner.file_type().is_dir() {
        return Err(std::io::Error::other("Admin data owner source is not a directory"));
    }
    for path in [
        release.join("config"),
        release.join("config/rz.env"),
        release.join("config/rz-reports.env"),
    ] {
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.uid() != owner.uid() || metadata.gid() != owner.gid() {
            std::os::unix::fs::chown(&path, Some(owner.uid()), Some(owner.gid()))?;
        }
    }
    Ok(())
}

#[cfg(not(unix))]
pub(super) fn normalize_release_config_ownership(
    _release: &Path,
    _runtime_root: &Path,
) -> Result<(), std::io::Error> {
    Ok(())
}

pub(super) fn sync_release_tree(root: &Path) -> Result<(), std::io::Error> {
    for directory in
        [root.join("bin"), root.join("systemd"), root.join("config"), root.join("identity")]
    {
        sync_directory(&directory)?;
    }
    sync_directory(root)
}

pub(super) fn sync_directory(path: &Path) -> Result<(), std::io::Error> {
    fs::File::open(path)?.sync_all()
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode & 0o777))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> Result<(), std::io::Error> {
    Ok(())
}
