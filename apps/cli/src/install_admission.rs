use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
    path::{Path, PathBuf},
};

/// A verified parent held open for the complete fresh-root publication.
/// Every destination operation below it uses this descriptor, never a reopened parent path.
pub(super) struct PrivateParent(File);

impl PrivateParent {
    pub(super) fn open(path: &Path) -> Result<Self, String> {
        let file = open_directory_path(path)?;
        let metadata = file.metadata().map_err(io)?;
        if !metadata.is_dir() || metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
            return Err("destination parent must be root-owned and not group/world writable".into());
        }
        Ok(Self(file))
    }

    pub(super) fn absent(&self, value: &str) -> Result<(), String> {
        let value = name(value)?;
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        let result = unsafe {
            libc::fstatat(
                self.0.as_raw_fd(),
                value.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result == 0 {
            return Err("destination must be a fresh absent root".into());
        }
        if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error().to_string())
        }
    }

    pub(super) fn create_temp(&self, value: &str) -> Result<(), String> {
        let value = name(value)?;
        if unsafe { libc::mkdirat(self.0.as_raw_fd(), value.as_ptr(), 0o700) } != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(())
    }

    pub(super) fn rename_noreplace(&self, from: &str, to: &str) -> Result<(), String> {
        let from = name(from)?;
        let to = name(to)?;
        #[cfg(target_os = "linux")]
        {
            if unsafe {
                libc::syscall(
                    libc::SYS_renameat2,
                    self.0.as_raw_fd(),
                    from.as_ptr(),
                    self.0.as_raw_fd(),
                    to.as_ptr(),
                    1u32,
                )
            } != 0
            {
                return Err(std::io::Error::last_os_error().to_string());
            }
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (from, to);
            Err("apply supports Linux only".into())
        }
    }

    pub(super) fn child_path(&self, value: &str) -> Result<PathBuf, String> {
        let value = name(value)?;
        Ok(PathBuf::from(format!("/proc/self/fd/{}", self.0.as_raw_fd()))
            .join(std::str::from_utf8(value.as_bytes()).map_err(|_| "invalid path")?))
    }

    pub(super) fn sync(&self) -> Result<(), String> {
        self.0.sync_all().map_err(io)
    }
}

fn open_directory_path(path: &Path) -> Result<File, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(io)?.join(path)
    };
    let mut current = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open("/")
        .map_err(io)?;
    for part in absolute.components() {
        let part = match part {
            std::path::Component::RootDir | std::path::Component::CurDir => continue,
            std::path::Component::ParentDir | std::path::Component::Prefix(_) => {
                return Err("destination parent contains an unsafe path component".into());
            }
            std::path::Component::Normal(part) => part,
        };
        let part = CString::new(part.as_bytes()).map_err(|_| "destination parent is invalid")?;
        let next = unsafe {
            libc::openat(
                current.as_raw_fd(),
                part.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if next < 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        current = unsafe { File::from_raw_fd(next) };
    }
    Ok(current)
}

fn name(value: &str) -> Result<CString, String> {
    if value.is_empty() || value.as_bytes().contains(&b'/') {
        return Err("destination name is invalid".into());
    }
    CString::new(value).map_err(|_| "destination name is invalid".into())
}

pub(super) fn destination_name(destination: &Path) -> Result<String, String> {
    let value = destination.file_name().ok_or("destination has no final name")?;
    if value.as_bytes().is_empty() || value.as_bytes().contains(&b'/') {
        return Err("destination name is invalid".into());
    }
    value.to_str().map(str::to_owned).ok_or("destination name is invalid".into())
}
pub(super) fn remove_tree(path: &Path) {
    let _ = fs::remove_dir_all(path);
}
pub(super) fn valid_key_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric() || (index != 0 && matches!(byte, b'.' | b'_' | b'-'))
        })
}
pub(super) fn hash_id(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
pub(super) fn host_target() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x86_64-unknown-linux-musl"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64-unknown-linux-gnu"
    } else {
        "unsupported"
    }
}
fn io(error: std::io::Error) -> String {
    error.to_string()
}
