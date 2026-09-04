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

/// Failure class for no-replace publication. Callers may inspect only a
/// confirmed rename conflict: all I/O and durability failures remain failures.
#[derive(Debug)]
pub(super) enum PublishError {
    Conflict,
    Durability(String),
    Io(String),
    #[cfg(debug_assertions)]
    Fault(String),
}

impl std::fmt::Display for PublishError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conflict => formatter.write_str("profile publication conflict"),
            Self::Durability(message) | Self::Io(message) => formatter.write_str(message),
            #[cfg(debug_assertions)]
            Self::Fault(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for PublishError {}

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

    pub(super) fn open_child_directory(&self, value: &str) -> Result<Self, String> {
        let value = name(value)?;
        let raw = unsafe {
            libc::openat(
                self.0.as_raw_fd(),
                value.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let file = unsafe { File::from_raw_fd(raw) };
        let metadata = file.metadata().map_err(io)?;
        if !metadata.is_dir() || metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
            return Err("destination parent must be root-owned and not group/world writable".into());
        }
        Ok(Self(file))
    }

    pub(super) fn sync(&self) -> Result<(), String> {
        self.0.sync_all().map_err(io)
    }

    pub(super) fn metadata(&self) -> Result<std::fs::Metadata, String> {
        self.0.metadata().map_err(io)
    }

    pub(super) fn lock_exclusive(&self, value: &str) -> Result<File, String> {
        let value = name(value)?;
        let raw = unsafe {
            libc::openat(
                self.0.as_raw_fd(),
                value.as_ptr(),
                libc::O_RDWR | libc::O_CREAT | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        if raw < 0 {
            return Err("activation lock could not be opened".into());
        }
        let file = unsafe { File::from_raw_fd(raw) };
        let metadata = file.metadata().map_err(io)?;
        if !metadata.is_file()
            || metadata.uid() != 0
            || metadata.gid() != 0
            || metadata.mode() & 0o777 != 0o600
        {
            return Err("activation lock is unsafe".into());
        }
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err("activation lock could not be acquired".into());
        }
        self.sync()?;
        Ok(file)
    }

    /// Publishes one fully written regular file in this already-open directory.
    /// The final rename is no-replace, so concurrent publishers can only race to
    /// publish identical bytes; callers inspect the winner for idempotence.
    pub(super) fn publish_regular_noreplace(
        &self,
        destination: &str,
        bytes: &[u8],
        uid: u32,
        gid: u32,
        mode: u32,
    ) -> Result<(), PublishError> {
        let destination = name(destination).map_err(PublishError::Io)?;
        for nonce in 0..32u32 {
            profile_publish_fault("create")?;
            let temporary = name(&format!(".rz-publish-{}-{nonce}", std::process::id()))
                .map_err(PublishError::Io)?;
            let raw = unsafe {
                libc::openat(
                    self.0.as_raw_fd(),
                    temporary.as_ptr(),
                    libc::O_WRONLY
                        | libc::O_CREAT
                        | libc::O_EXCL
                        | libc::O_NOFOLLOW
                        | libc::O_CLOEXEC,
                    mode,
                )
            };
            if raw < 0 {
                if std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST) {
                    continue;
                }
                return Err(PublishError::Io("profile temporary publication failed".into()));
            }
            let mut file = unsafe { File::from_raw_fd(raw) };
            let result = (|| {
                profile_publish_fault("write")?;
                if unsafe { libc::fchown(file.as_raw_fd(), uid, gid) } != 0
                    || unsafe { libc::fchmod(file.as_raw_fd(), mode as libc::mode_t) } != 0
                {
                    return Err(PublishError::Io("profile ownership publication failed".into()));
                }
                use std::io::Write;
                file.write_all(bytes).map_err(|error| PublishError::Io(io(error)))?;
                profile_publish_fault("fsync")?;
                file.sync_all().map_err(|error| PublishError::Durability(io(error)))?;
                profile_publish_fault("rename")?;
                self.rename_noreplace_cstr(&temporary, &destination)?;
                profile_publish_fault("dirsync")?;
                self.sync().map_err(PublishError::Durability)
            })();
            if result.is_err() {
                unsafe { libc::unlinkat(self.0.as_raw_fd(), temporary.as_ptr(), 0) };
            }
            return result;
        }
        Err(PublishError::Conflict)
    }

    fn rename_noreplace_cstr(&self, from: &CString, to: &CString) -> Result<(), PublishError> {
        #[cfg(target_os = "linux")]
        {
            if unsafe {
                libc::syscall(
                    libc::SYS_renameat2,
                    self.0.as_raw_fd(),
                    from.as_ptr(),
                    self.0.as_raw_fd(),
                    to.as_ptr(),
                    libc::RENAME_NOREPLACE,
                )
            } != 0
            {
                return if std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST) {
                    Err(PublishError::Conflict)
                } else {
                    Err(PublishError::Io("profile publication failed".into()))
                };
            }
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (from, to);
            Err(PublishError::Io("profile publication supports Linux only".into()))
        }
    }
}

fn profile_publish_fault(_stage: &str) -> Result<(), PublishError> {
    #[cfg(debug_assertions)]
    if std::env::var("RUSTZEN_PROFILE_PUBLISH_FAULT").ok().as_deref() == Some(_stage) {
        return Err(PublishError::Fault(format!("debug profile publication fault at {_stage}")));
    }
    Ok(())
}

/// Opens every absolute component through a directory descriptor and requires an
/// immutable root-owned path. This is for installed roots, unlike `open`, whose
/// parent may intentionally be a sticky temporary directory during tests.
pub(super) fn validate_root_owned_path(path: &Path) -> Result<(), String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        return Err("Agent root must be an absolute path".into());
    };
    let mut current = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open("/")
        .map_err(io)?;
    let root = current.metadata().map_err(io)?;
    if root.uid() != 0 || root.mode() & 0o022 != 0 {
        return Err("filesystem root is not a safe root-owned directory".into());
    }
    for part in absolute.components() {
        let part = match part {
            std::path::Component::RootDir | std::path::Component::CurDir => continue,
            std::path::Component::ParentDir | std::path::Component::Prefix(_) => {
                return Err("Agent root contains an unsafe path component".into());
            }
            std::path::Component::Normal(part) => part,
        };
        let part = CString::new(part.as_bytes()).map_err(|_| "Agent root is invalid")?;
        let next = unsafe {
            libc::openat(
                current.as_raw_fd(),
                part.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if next < 0 {
            return Err("Agent root is unavailable".into());
        }
        current = unsafe { File::from_raw_fd(next) };
        let metadata = current.metadata().map_err(io)?;
        if !metadata.is_dir() || metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
            return Err("Agent root path must be root-owned and not group/world writable".into());
        }
    }
    Ok(())
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
