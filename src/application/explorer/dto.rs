use std::path::PathBuf;

/// One entry of a host directory listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNodeDto {
    pub name: String,
    pub is_dir: bool,
}

/// A directory listing: the current path and its sorted entries
/// (directories first, then alphabetical).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryListingDto {
    pub path: PathBuf,
    pub entries: Vec<FileNodeDto>,
}
