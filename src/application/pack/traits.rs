use std::path::Path;

use mockall::automock;

use crate::application::error::ApplicationError;
use crate::domain::pack::entity::Pack;
use crate::domain::pack::value_objects::TargetCli;

/// Read access to the pack library (directories with pack.toml manifests).
#[automock]
pub trait PackRepository: Send {
    /// List every available pack, sorted by name.
    fn list(&self) -> Result<Vec<Pack>, ApplicationError>;
    /// Load one pack by name.
    fn find(&self, name: &str) -> Result<Pack, ApplicationError>;
}

/// Write access to the target filesystem for installing a planned copy.
#[automock]
pub trait InstallDestination: Send {
    /// Recursively copy `source` (inside the pack dir) to `destination`
    /// (absolute, inside the chosen target root). Fails if the destination
    /// already exists (no silent overwrite).
    fn copy(&self, source: &Path, destination: &Path) -> Result<(), ApplicationError>;
    /// Check existence of a destination path (for collision warnings).
    fn exists(&self, destination: &Path) -> bool;
}

/// Which CLI an install targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallRequest {
    pub pack_name: String,
    pub cli: TargetCli,
    pub target_root: String,
    pub overwrite: bool,
}
