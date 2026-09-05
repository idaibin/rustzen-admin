use crate::install_admission::PrivateParent;
use std::{
    ffi::CString,
    fs::{File, OpenOptions},
    io::Read,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
    path::Path,
};

/// A service-owned runtime parent held through a verified directory descriptor.
pub(super) struct ServiceParent(File);

impl ServiceParent {
    pub(super) fn create(
        parent: &PrivateParent,
        value: &str,
        uid: u32,
        gid: u32,
    ) -> Result<Self, String> {
        let value = name(value)?;
        let created = if unsafe { libc::mkdirat(parent.directory_fd(), value.as_ptr(), 0o700) } == 0
        {
            true
        } else if std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST) {
            false
        } else {
            return Err("service runtime parent could not be created".into());
        };
        let raw = unsafe {
            libc::openat(
                parent.directory_fd(),
                value.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err("service runtime parent is unavailable".into());
        }
        let file = unsafe { File::from_raw_fd(raw) };
        if created
            && (unsafe { libc::fchown(file.as_raw_fd(), uid, gid) } != 0
                || unsafe { libc::fchmod(file.as_raw_fd(), 0o750) } != 0)
        {
            return Err("service runtime parent ownership could not be published".into());
        }
        validate(&file, uid, gid)?;
        file.sync_all().map_err(io)?;
        Ok(Self(file))
    }

    pub(super) fn open(path: &Path, uid: u32, gid: u32) -> Result<Self, String> {
        let file = open_directory_path(path)?;
        validate(&file, uid, gid)?;
        Ok(Self(file))
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
                    libc::RENAME_NOREPLACE,
                )
            } != 0
            {
                return Err(std::io::Error::last_os_error().to_string());
            }
            return self.0.sync_all().map_err(io);
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (from, to);
            Err("service database publication supports Linux only".into())
        }
    }

    pub(super) fn absent(&self, value: &str) -> Result<(), String> {
        if self.exists(value)? { Err("service database must be absent".into()) } else { Ok(()) }
    }

    pub(super) fn exists(&self, value: &str) -> Result<bool, String> {
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
            Ok(true)
        } else if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
            Ok(false)
        } else {
            Err(std::io::Error::last_os_error().to_string())
        }
    }

    pub(super) fn require_regular_owned(
        &self,
        value: &str,
        uid: u32,
        gid: u32,
    ) -> Result<(), String> {
        let value = name(value)?;
        let raw = unsafe {
            libc::openat(
                self.0.as_raw_fd(),
                value.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err("service database is unavailable".into());
        }
        let file = unsafe { File::from_raw_fd(raw) };
        let metadata = file.metadata().map_err(io)?;
        if !metadata.is_file()
            || metadata.uid() != uid
            || metadata.gid() != gid
            || metadata.mode() & 0o022 != 0
        {
            return Err("service database is unsafe".into());
        }
        Ok(())
    }

    pub(super) fn read_regular_owned(
        &self,
        value: &str,
        maximum: usize,
        uid: u32,
        gid: u32,
    ) -> Result<Vec<u8>, String> {
        self.require_regular_owned(value, uid, gid)?;
        let value = name(value)?;
        let raw = unsafe {
            libc::openat(
                self.0.as_raw_fd(),
                value.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err("service database is unavailable".into());
        }
        let mut file = unsafe { File::from_raw_fd(raw) };
        let metadata = file.metadata().map_err(io)?;
        if metadata.len() > maximum as u64 {
            return Err("service database is unsafe".into());
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.read_to_end(&mut bytes).map_err(io)?;
        Ok(bytes)
    }

    pub(super) fn remove_regular_owned(
        &self,
        value: &str,
        uid: u32,
        gid: u32,
    ) -> Result<(), String> {
        if !self.exists(value)? {
            return Ok(());
        }
        self.require_regular_owned(value, uid, gid)?;
        let value = name(value)?;
        if unsafe { libc::unlinkat(self.0.as_raw_fd(), value.as_ptr(), 0) } != 0 {
            return Err("service staging database removal failed".into());
        }
        self.0.sync_all().map_err(io)
    }
}

fn validate(file: &File, uid: u32, gid: u32) -> Result<(), String> {
    let meta = file.metadata().map_err(io)?;
    if !meta.is_dir() || meta.uid() != uid || meta.gid() != gid || meta.mode() & 0o777 != 0o750 {
        Err("service runtime parent is unsafe".into())
    } else {
        Ok(())
    }
}

fn open_directory_path(path: &Path) -> Result<File, String> {
    let mut current = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open("/")
        .map_err(io)?;
    for part in path.components() {
        let part = match part {
            std::path::Component::RootDir | std::path::Component::CurDir => continue,
            std::path::Component::Normal(part) => part,
            _ => return Err("service runtime parent contains an unsafe path component".into()),
        };
        let part =
            CString::new(part.as_bytes()).map_err(|_| "service runtime parent is invalid")?;
        let raw = unsafe {
            libc::openat(
                current.as_raw_fd(),
                part.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err("service runtime parent is unavailable".into());
        }
        current = unsafe { File::from_raw_fd(raw) };
    }
    Ok(current)
}

fn name(value: &str) -> Result<CString, String> {
    if value.is_empty() || value.as_bytes().contains(&b'/') {
        return Err("service filename is invalid".into());
    }
    CString::new(value).map_err(|_| "service filename is invalid".into())
}

fn io(error: std::io::Error) -> String {
    error.to_string()
}
