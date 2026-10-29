use std::path::PathBuf;

/// One item to import into a new pack, as given on the CLI: a host path
/// (file or directory) plus its per-CLI destinations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateItemDto {
    /// Raw kind name ("skill", "agent", "command"); parsed by the service.
    pub kind: String,
    /// Path on the host (file or directory), relative to cwd or absolute.
    pub source: PathBuf,
    /// (cli name, destination) pairs; cli names are parsed by the service.
    pub destinations: Vec<(String, String)>,
}

/// Input of `PackCreationService::create`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePackDto {
    pub name: String,
    pub version: String,
    pub description: String,
    /// Raw CLI names. Empty = deduce the union of the items' destinations.
    pub clis: Vec<String>,
    pub items: Vec<CreateItemDto>,
    pub overwrite: bool,
}

/// One file/directory to copy into the pack: host path -> canonical path
/// inside the pack directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackSourceDto {
    pub host_path: PathBuf,
    pub pack_source: PathBuf,
}

/// What the writer reports after a successful write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackWriteOutcomeDto {
    pub pack_dir: PathBuf,
    pub copied_count: usize,
}

/// Result of a successful pack creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatePackResultDto {
    pub pack_name: String,
    pub pack_dir: PathBuf,
    pub copied_count: usize,
}
