//! Runtime layout helpers for V2 local-first startup.

use std::path::{Component, Path, PathBuf};

/// Default runtime root used by local development and packaging.
pub const DEFAULT_RUNTIME_ROOT: &str = ".rustzen-admin";

/// Default public files prefix for uploaded assets.
pub const DEFAULT_FILES_PREFIX: &str = "/resources";

/// Shared runtime paths under a single root path.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RuntimeLayout {
    runtime_root: String,
    files_prefix: String,
}

impl RuntimeLayout {
    /// Creates a layout bound to a runtime root and public files prefix.
    pub fn new(runtime_root: impl Into<String>, files_prefix: impl Into<String>) -> Self {
        Self {
            runtime_root: runtime_root.into(),
            files_prefix: files_prefix.into(),
        }
    }

    /// Returns the configured runtime root as string.
    pub fn runtime_root(&self) -> &str {
        &self.runtime_root
    }

    /// Root directory path as configured (relative or absolute).
    pub fn runtime_root_dir(&self) -> PathBuf {
        PathBuf::from(&self.runtime_root)
    }

    /// Directory for packaged frontend assets.
    pub fn web_dist_dir(&self) -> PathBuf {
        self.runtime_root_dir().join("web").join("dist")
    }

    /// Directory for uploaded and runtime data.
    pub fn data_dir(&self) -> PathBuf {
        self.runtime_root_dir().join("data")
    }

    /// Directory for logs.
    pub fn log_dir(&self) -> PathBuf {
        self.runtime_root_dir().join("logs")
    }

    /// Avatar file directory.
    pub fn avatars_dir(&self) -> PathBuf {
        self.data_dir().join("avatars")
    }

    /// Upload root directory.
    pub fn uploads_dir(&self) -> PathBuf {
        self.data_dir().join("uploads")
    }

    /// Public avatar prefix for static file route.
    pub fn avatars_prefix(&self) -> String {
        format!("{}/avatars", self.files_prefix.trim_end_matches('/'))
    }

    /// Resolves a runtime path value relative to runtime root.
    pub fn resolve_runtime_path(&self, value: &str) -> PathBuf {
        if value == ":memory:" || value.starts_with("sqlite:") {
            PathBuf::from(value)
        } else {
            resolve_path_with_runtime_root(&self.runtime_root, value)
        }
    }
}

/// Resolves a path relative to a runtime root.
///
/// Relative paths use the local working directory when runtime root is relative.
pub fn resolve_path_with_runtime_root(runtime_root: &str, value: &str) -> PathBuf {
    let candidate = Path::new(value);
    let runtime_root = {
        let root = Path::new(runtime_root);
        if root.is_absolute() {
            root.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(root))
                .unwrap_or_else(|_| root.to_path_buf())
        }
    };

    let path = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        runtime_root.join(candidate)
    };

    normalize_path(&path)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::{resolve_path_with_runtime_root, DEFAULT_RUNTIME_ROOT, RuntimeLayout};
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn runtime_layout_derives_standard_directories() {
        let layout = RuntimeLayout::new(".rustzen-admin", "/resources");

        assert_eq!(layout.runtime_root_dir(), PathBuf::from(".rustzen-admin"));
        assert_eq!(layout.web_dist_dir(), PathBuf::from(".rustzen-admin/web/dist"));
        assert_eq!(layout.data_dir(), PathBuf::from(".rustzen-admin/data"));
        assert_eq!(layout.log_dir(), PathBuf::from(".rustzen-admin/logs"));
        assert_eq!(layout.uploads_dir(), PathBuf::from(".rustzen-admin/data/uploads"));
        assert_eq!(layout.avatars_dir(), PathBuf::from(".rustzen-admin/data/avatars"));
        assert_eq!(layout.avatars_prefix(), "/resources/avatars");
    }

    #[test]
    fn runtime_root_default_matches_constant() {
        assert_eq!(DEFAULT_RUNTIME_ROOT, ".rustzen-admin");
    }

    #[test]
    fn resolve_path_prefers_absolute_candidate() {
        let resolved = resolve_path_with_runtime_root(".rustzen-admin", "/tmp/data.db");

        assert_eq!(resolved, PathBuf::from("/tmp/data.db"));
        assert!(Path::new("/tmp/data.db").is_absolute());
    }
}
