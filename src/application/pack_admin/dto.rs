use std::path::PathBuf;

use crate::domain::pack::value_objects::PackItemKind;

/// Input of `PackAdminService::add_from_dir` (E13): a base directory whose
/// `agents/`, `skills/`, `rules/`, `commands/` subdirectories become the
/// pack's items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddPackDto {
    /// Base directory of the pack.
    pub dir: PathBuf,
    /// Explicit pack name; defaults to the directory's file name.
    pub name: Option<String>,
    pub overwrite: bool,
}

/// Result of a successful `add_from_dir`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddPackResultDto {
    pub pack_name: String,
    pub pack_dir: PathBuf,
    pub item_count: usize,
    /// Non-fatal issues noticed at creation, e.g. an item for which no
    /// loaded CLI profile provides a template (it gets no destination and
    /// could not be installed). Printed as warnings by the CLI.
    pub warnings: Vec<String>,
}

/// Result of a successful `remove`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovePackResultDto {
    pub pack_name: String,
    pub pack_dir: PathBuf,
}

/// One entry found in the base directory: a host path plus its canonical
/// path inside the pack directory (`<kind>s/<name>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedItem {
    pub kind: PackItemKind,
    pub host_path: PathBuf,
    pub pack_source: PathBuf,
    pub source_is_dir: bool,
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
