use std::path::Path;

use crate::application::error::ApplicationError;
use crate::application::library::traits::LibraryInitializer;

/// std::fs adapter for the library initializer port.
pub struct FsLibraryInitializer;

impl LibraryInitializer for FsLibraryInitializer {
    fn ensure_user_root(&self, root: &Path) -> Result<(), ApplicationError> {
        std::fs::create_dir_all(root).map_err(|source| ApplicationError::Io {
            path: root.to_string_lossy().into_owned(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_user_root_creates_missing_directories_idempotently() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path().join("nested").join("packs");
        let initializer = FsLibraryInitializer;
        initializer.ensure_user_root(&root).unwrap();
        assert!(root.is_dir());
        // Idempotent: a second call on an existing directory succeeds.
        initializer.ensure_user_root(&root).unwrap();
    }
}
