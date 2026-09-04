use std::path::Path;

use super::{FileSignature, ServiceError};

pub struct SecureDirectory;
pub struct SecureFile;

pub fn open_directory(_: &Path) -> Result<Option<SecureDirectory>, ServiceError> {
    Err(unsupported())
}

impl SecureDirectory {
    pub fn open_file(&self, _: &str) -> Result<SecureFile, ServiceError> {
        Err(unsupported())
    }

    pub fn unlink_if_unchanged(
        &self,
        _: &str,
        _: FileSignature,
        _: &str,
    ) -> Result<(), ServiceError> {
        Err(unsupported())
    }
}

impl SecureFile {
    pub fn signature(&self) -> Result<FileSignature, ServiceError> {
        Err(unsupported())
    }

    pub fn read_all(&self) -> Result<Vec<u8>, ServiceError> {
        Err(unsupported())
    }

    pub fn read_range(&self, _: u64, _: u64) -> Result<Vec<u8>, ServiceError> {
        Err(unsupported())
    }
}

fn unsupported() -> ServiceError {
    ServiceError::InvalidOperation(
        "Secure module log file operations are unsupported on this target".into(),
    )
}
