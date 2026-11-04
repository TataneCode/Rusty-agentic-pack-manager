use std::path::PathBuf;

use crate::domain::pack::entity::Pack;
use crate::domain::pack::installed::InstalledPack;

/// UI-facing summary of one pack. CLI identifiers are plain strings
/// (`CliId::as_str()`): DTOs never expose domain types (E01).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackSummaryDto {
    pub name: String,
    pub version: String,
    pub description: String,
    pub clis: Vec<String>,
    pub item_count: usize,
}

impl PackSummaryDto {
    pub fn from_pack(pack: &Pack) -> PackSummaryDto {
        PackSummaryDto {
            name: pack.name.as_str().to_string(),
            version: pack.version.as_str().to_string(),
            description: pack.description.clone(),
            clis: pack
                .clis
                .iter()
                .map(|cli| cli.as_str().to_string())
                .collect(),
            item_count: pack.items.len(),
        }
    }
}

/// Result of a successful installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallResultDto {
    pub pack_name: String,
    pub target_root: String,
    pub copied_count: usize,
}

/// UI-facing view of one recorded installation (packs_index.toml entry).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPackDto {
    pub name: String,
    pub version: String,
    pub cli: String,
    pub paths: Vec<PathBuf>,
}

impl InstalledPackDto {
    pub fn from_installed(installed: &InstalledPack) -> InstalledPackDto {
        InstalledPackDto {
            name: installed.name.as_str().to_string(),
            version: installed.version.as_str().to_string(),
            cli: installed.cli.as_str().to_string(),
            paths: installed.paths.clone(),
        }
    }
}

/// Result of a successful uninstall.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallResultDto {
    pub pack_name: String,
    pub cli: String,
    pub removed_count: usize,
}

/// Status of one planned copy in an install plan (E04 dry-run).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanEntryStatus {
    /// Destination is free: the install would copy here.
    Ok,
    /// Destination already exists on disk and is recorded in the install
    /// index (owned by an installed pack).
    Collision,
    /// Destination already exists on disk but is not tracked by the index
    /// (a user file).
    Untracked,
}

impl PlanEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlanEntryStatus::Ok => "ok",
            PlanEntryStatus::Collision => "collision",
            PlanEntryStatus::Untracked => "hors index",
        }
    }
}

/// One entry of an install plan: a source file inside the pack mapped to
/// its destination (relative to the target root), with its status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanEntryDto {
    pub source: String,
    pub destination: String,
    pub status: PlanEntryStatus,
}

/// Install plan of one pack for one CLI inside a target root, computable
/// without any write (E04): the dry-run result shared by the CLI and the TUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlanDto {
    pub pack_name: String,
    pub cli: String,
    pub target_root: String,
    pub entries: Vec<PlanEntryDto>,
}
