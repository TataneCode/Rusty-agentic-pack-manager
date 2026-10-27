use std::path::PathBuf;

use crate::domain::pack::entity::Pack;
use crate::domain::pack::installed::InstalledPack;
use crate::domain::pack::value_objects::TargetCli;

/// UI-facing summary of one pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackSummaryDto {
    pub name: String,
    pub version: String,
    pub description: String,
    pub clis: Vec<TargetCli>,
    pub item_count: usize,
}

impl PackSummaryDto {
    pub fn from_pack(pack: &Pack) -> PackSummaryDto {
        PackSummaryDto {
            name: pack.name.as_str().to_string(),
            version: pack.version.as_str().to_string(),
            description: pack.description.clone(),
            clis: pack.clis.clone(),
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
    pub cli: TargetCli,
    pub paths: Vec<PathBuf>,
}

impl InstalledPackDto {
    pub fn from_installed(installed: &InstalledPack) -> InstalledPackDto {
        InstalledPackDto {
            name: installed.name.as_str().to_string(),
            version: installed.version.as_str().to_string(),
            cli: installed.cli,
            paths: installed.paths.clone(),
        }
    }
}

/// Result of a successful uninstall.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallResultDto {
    pub pack_name: String,
    pub cli: TargetCli,
    pub removed_count: usize,
}
