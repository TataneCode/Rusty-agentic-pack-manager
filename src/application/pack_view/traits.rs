use mockall::automock;

use crate::application::error::ApplicationError;

/// A file inside a pack, viewable by the viewer screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackFileDto {
    /// Path relative to the pack directory.
    pub path: String,
    pub is_text: bool,
}

/// Read access to one pack's files, for the viewer.
#[automock]
pub trait PackContentRepository: Send {
    /// List the files of pack `pack_name`, sorted, excluding pack.toml.
    fn list_files(&self, pack_name: &str) -> Result<Vec<PackFileDto>, ApplicationError>;
    /// Read one file's content (UTF-8 text) of pack `pack_name`.
    fn read_file(&self, pack_name: &str, path: &str) -> Result<String, ApplicationError>;
    /// Whether `path` (relative to the pack dir) is a directory. Used to
    /// pick the `_dir` destination template of a CLI profile (E02).
    fn source_is_dir(
        &self,
        pack_name: &str,
        path: &std::path::Path,
    ) -> Result<bool, ApplicationError>;
}
