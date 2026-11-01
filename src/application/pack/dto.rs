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

/// One item of a pack, as plain strings for display (`rapm show`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackItemDto {
    pub kind: String,
    pub source: String,
    pub destinations: Vec<(String, String)>,
}

/// Full manifest summary of one pack, for `rapm show` (E03). CLI
/// identifiers and destinations are plain strings: DTOs never expose
/// domain types (E01).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackDetailDto {
    pub name: String,
    pub version: String,
    pub description: String,
    pub clis: Vec<String>,
    pub items: Vec<PackItemDto>,
}

impl PackDetailDto {
    pub fn from_pack(pack: &Pack) -> PackDetailDto {
        PackDetailDto {
            name: pack.name.as_str().to_string(),
            version: pack.version.as_str().to_string(),
            description: pack.description.clone(),
            clis: pack
                .clis
                .iter()
                .map(|cli| cli.as_str().to_string())
                .collect(),
            items: pack
                .items
                .iter()
                .map(|item| PackItemDto {
                    kind: item.kind.as_str().to_string(),
                    source: item.source.to_string_lossy().into_owned(),
                    destinations: item
                        .destinations
                        .iter()
                        .map(|(cli, dest)| (cli.as_str().to_string(), dest.to_string()))
                        .collect(),
                })
                .collect(),
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
