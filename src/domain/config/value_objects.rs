use std::fmt;
use std::path::{Path, PathBuf};

use crate::domain::pack::error::DomainError;

/// Absolute, non-empty library directory, as read from the config file
/// (`packs_dir`). `~` is rejected: no shell expansion happens inside a
/// config file. A library root must not depend on the cwd.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LibraryPath(PathBuf);

impl LibraryPath {
    pub fn new(path: PathBuf) -> Result<LibraryPath, DomainError> {
        if path.as_os_str().is_empty() {
            return Err(DomainError::InvalidLibraryPath("(empty)".to_string()));
        }
        if !path.is_absolute() {
            return Err(DomainError::InvalidLibraryPath(
                path.to_string_lossy().into_owned(),
            ));
        }
        Ok(LibraryPath(path))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

impl fmt::Display for LibraryPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_string_lossy())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_path_accepts_absolute_paths() {
        let path = LibraryPath::new(PathBuf::from("/home/u/packs")).unwrap();
        assert_eq!(path.as_path(), Path::new("/home/u/packs"));
        assert_eq!(path.to_string(), "/home/u/packs");
    }

    #[test]
    fn library_path_rejects_relative_and_empty_paths() {
        assert!(matches!(
            LibraryPath::new(PathBuf::from("relative/packs")).unwrap_err(),
            DomainError::InvalidLibraryPath(_)
        ));
        assert!(matches!(
            LibraryPath::new(PathBuf::from("~/packs")).unwrap_err(),
            DomainError::InvalidLibraryPath(_)
        ));
        assert!(matches!(
            LibraryPath::new(PathBuf::new()).unwrap_err(),
            DomainError::InvalidLibraryPath(_)
        ));
    }
}
