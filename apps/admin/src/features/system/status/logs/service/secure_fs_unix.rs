use std::{
    ffi::CString,
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::ffi::OsStrExt,
    },
    path::Path,
};

use super::{FileSignature, digest_bytes, same_signature, signature};
use crate::common::error::ServiceError;
use uuid::Uuid;

pub struct SecureDirectory {
    file: File,
}

pub struct SecureFile {
    file: File,
}

pub fn open_directory(path: &Path) -> Result<Option<SecureDirectory>, ServiceError> {
    let path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
        ServiceError::InvalidOperation("Configured log directory contains an invalid byte".into())
    })?;
    let fd = unsafe {
        // SAFETY: `path` is a NUL-terminated CString and the returned descriptor is owned below.
        libc::open(
            path.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if fd < 0 {
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::NotFound {
            return Ok(None);
        }
        return Err(ServiceError::InvalidOperation(
            "Configured log directory is unavailable or unsafe".into(),
        ));
    }
    let file = unsafe {
        // SAFETY: `fd` is a valid descriptor returned by `open`; this call transfers its ownership.
        File::from_raw_fd(fd)
    };
    Ok(Some(SecureDirectory { file }))
}

impl SecureDirectory {
    pub fn open_file(&self, name: &str) -> Result<SecureFile, ServiceError> {
        let name = component(name)?;
        let fd = unsafe {
            // SAFETY: the directory descriptor and NUL-terminated name are valid for `openat`.
            libc::openat(
                self.file.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            return Err(ServiceError::InvalidOperation(
                "Module log file is unavailable or unsafe".into(),
            ));
        }
        let file = unsafe {
            // SAFETY: `fd` is a valid descriptor returned by `openat`; this call transfers its ownership.
            File::from_raw_fd(fd)
        };
        let metadata = file.metadata().map_err(io_error("stat module log"))?;
        if !metadata.is_file() {
            return Err(ServiceError::InvalidOperation(
                "Module log file is not a regular file".into(),
            ));
        }
        Ok(SecureFile { file })
    }

    pub fn unlink_if_unchanged(
        &self,
        name: &str,
        expected: FileSignature,
        expected_digest: &str,
    ) -> Result<(), ServiceError> {
        self.unlink_if_unchanged_impl(name, expected, expected_digest, || {})
    }

    #[cfg(test)]
    pub fn unlink_if_unchanged_with_barrier<F>(
        &self,
        name: &str,
        expected: FileSignature,
        expected_digest: &str,
        barrier: F,
    ) -> Result<(), ServiceError>
    where
        F: FnOnce(),
    {
        self.unlink_if_unchanged_impl(name, expected, expected_digest, barrier)
    }

    fn unlink_if_unchanged_impl<F>(
        &self,
        name: &str,
        expected: FileSignature,
        expected_digest: &str,
        barrier: F,
    ) -> Result<(), ServiceError>
    where
        F: FnOnce(),
    {
        let first = self.open_file(name)?;
        let first_signature = first.signature()?;
        if !same_signature(first_signature, expected) {
            return Err(ServiceError::InvalidOperation("Module log changed since preview".into()));
        }
        let first_bytes = first.read_all()?;
        if digest_bytes(&first_bytes) != expected_digest
            || !same_signature(first.signature()?, expected)
        {
            return Err(ServiceError::InvalidOperation("Module log changed since preview".into()));
        }

        // Re-open relative to the same directory immediately before the transaction. The
        // descriptor identity and digest reject ordinary replacement barriers.
        let current = self.open_file(name)?;
        if !same_signature(current.signature()?, expected)
            || digest_bytes(&current.read_all()?) != expected_digest
        {
            return Err(ServiceError::InvalidOperation("Module log changed since preview".into()));
        }

        // Move the entry to a private descriptor-relative name using an atomic no-replace
        // rename. The name is then validated again before unlinking. A same-name replacement
        // racing this point is moved to the private name and fails the identity/digest check;
        // it is restored only with a non-overwriting hard-link transaction.
        barrier();
        let source_name = component(name)?;
        let private_name = component(&format!(".rustzen-module-log-{}", Uuid::new_v4().simple()))?;
        rename_noreplace(self.file.as_raw_fd(), source_name.as_ptr(), private_name.as_ptr())?;
        let moved = match self.open_file(private_name.to_str().unwrap_or_default()) {
            Ok(file) => file,
            Err(error) => {
                self.restore_if_absent(&private_name, &source_name);
                return Err(error);
            }
        };
        if !same_signature(moved.signature()?, expected)
            || digest_bytes(&moved.read_all()?) != expected_digest
        {
            self.restore_if_absent(&private_name, &source_name);
            return Err(ServiceError::InvalidOperation("Module log changed during cleanup".into()));
        }
        let result = unsafe {
            // SAFETY: the directory descriptor and private basename are valid for `unlinkat`.
            libc::unlinkat(self.file.as_raw_fd(), private_name.as_ptr(), 0)
        };
        if result != 0 {
            self.restore_if_absent(&private_name, &source_name);
            return Err(ServiceError::InvalidOperation("Module log cleanup failed".into()));
        }
        if self.exists(&private_name)? {
            return Err(ServiceError::InvalidOperation(
                "Module log cleanup result could not be verified".into(),
            ));
        }
        Ok(())
    }

    fn restore_if_absent(&self, private_name: &CString, source_name: &CString) {
        let result = unsafe {
            // SAFETY: both names are validated basenames under the owned directory descriptor.
            libc::linkat(
                self.file.as_raw_fd(),
                private_name.as_ptr(),
                self.file.as_raw_fd(),
                source_name.as_ptr(),
                0,
            )
        };
        if result == 0 {
            unsafe {
                // SAFETY: the successful link created a second link to the private entry.
                libc::unlinkat(self.file.as_raw_fd(), private_name.as_ptr(), 0);
            }
        }
    }

    fn exists(&self, name: &CString) -> Result<bool, ServiceError> {
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        let result = unsafe {
            // SAFETY: `stat` points to writable storage and the descriptor/name remain valid.
            libc::fstatat(
                self.file.as_raw_fd(),
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result == 0 {
            return Ok(true);
        }
        if io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
            Ok(false)
        } else {
            Err(ServiceError::InvalidOperation(
                "Module log cleanup result could not be verified".into(),
            ))
        }
    }
}

#[cfg(target_os = "linux")]
fn rename_noreplace(
    directory_fd: std::os::fd::RawFd,
    source: *const libc::c_char,
    destination: *const libc::c_char,
) -> Result<(), ServiceError> {
    let result = unsafe {
        // SAFETY: both basenames are validated and the directory descriptor is owned by the
        // caller. Calling the Linux syscall directly preserves `RENAME_NOREPLACE` on both
        // glibc and musl targets, where libc does not expose the same wrapper API.
        libc::syscall(
            libc::SYS_renameat2,
            directory_fd,
            source,
            directory_fd,
            destination,
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(ServiceError::InvalidOperation("Module log cleanup transaction could not start".into()))
    }
}

#[cfg(target_os = "macos")]
fn rename_noreplace(
    directory_fd: std::os::fd::RawFd,
    source: *const libc::c_char,
    destination: *const libc::c_char,
) -> Result<(), ServiceError> {
    let result = unsafe {
        // SAFETY: both basenames are validated and the directory descriptor is owned by the caller.
        libc::renameatx_np(directory_fd, source, directory_fd, destination, libc::RENAME_EXCL)
    };
    if result == 0 {
        Ok(())
    } else {
        Err(ServiceError::InvalidOperation("Module log cleanup transaction could not start".into()))
    }
}

#[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
fn rename_noreplace(
    _: std::os::fd::RawFd,
    _: *const libc::c_char,
    _: *const libc::c_char,
) -> Result<(), ServiceError> {
    Err(ServiceError::InvalidOperation(
        "Secure module log cleanup is unsupported on this target".into(),
    ))
}

impl SecureFile {
    pub fn signature(&self) -> Result<FileSignature, ServiceError> {
        self.file
            .metadata()
            .map(|metadata| signature(&metadata))
            .map_err(io_error("stat module log"))
    }

    pub fn read_all(&self) -> Result<Vec<u8>, ServiceError> {
        let mut file = self.file.try_clone().map_err(io_error("clone module log"))?;
        file.seek(SeekFrom::Start(0)).map_err(io_error("seek module log"))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(io_error("read module log"))?;
        Ok(bytes)
    }

    pub fn read_range(&self, start: u64, length: u64) -> Result<Vec<u8>, ServiceError> {
        let mut file = self.file.try_clone().map_err(io_error("clone module log"))?;
        file.seek(SeekFrom::Start(start)).map_err(io_error("seek module log"))?;
        let mut bytes = Vec::new();
        file.take(length).read_to_end(&mut bytes).map_err(io_error("read module log"))?;
        Ok(bytes)
    }
}

fn component(name: &str) -> Result<CString, ServiceError> {
    if name.is_empty() || name == "." || name == ".." || name.contains('/') || name.contains('\\') {
        return Err(ServiceError::InvalidOperation("Invalid module log file name".into()));
    }
    CString::new(name)
        .map_err(|_| ServiceError::InvalidOperation("Invalid module log file name".into()))
}

fn io_error(action: &'static str) -> impl FnOnce(io::Error) -> ServiceError {
    move |error| {
        tracing::warn!(%error, action, "Module log filesystem operation failed");
        ServiceError::InvalidOperation(format!("Module log operation failed: {action}"))
    }
}
