use crate::domain::pack::entity::Pack;
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
