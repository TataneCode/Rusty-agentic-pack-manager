use std::path::Path;

use mockall::automock;

use crate::application::error::ApplicationError;
use crate::domain::pack::entity::Pack;
use crate::domain::pack::installed::InstalledPack;

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
    /// Recursively copy `source` (relative to the pack dir) to
    /// `destination` (absolute, inside the chosen target root). Fails if
    /// the destination already exists (no silent overwrite).
    fn copy(
        &self,
        pack_name: &str,
        source: &Path,
        destination: &Path,
    ) -> Result<(), ApplicationError>;
    /// Check existence of a destination path (for collision warnings).
    fn exists(&self, destination: &Path) -> bool;
    /// Remove an installed path (file or directory tree). A missing path
    /// is tolerated: uninstalling is idempotent (the user may have
    /// deleted assets by hand since the install).
    fn remove(&self, path: &Path) -> Result<(), ApplicationError>;
}

/// Read/write access to the installation index (`packs_index.toml`) at the
/// root of a target repository.
#[automock]
pub trait InstallIndexRepository: Send {
    /// Load every installation recorded for `target_root`. A missing index
    /// file means no installation, not an error.
    fn load(&self, target_root: &str) -> Result<Vec<InstalledPack>, ApplicationError>;
    /// Record (upsert by pack name + CLI) one installation in the index.
    fn record(&self, target_root: &str, entry: &InstalledPack) -> Result<(), ApplicationError>;
    /// Remove the entry matching (name, cli) from the index.
    fn remove(
        &self,
        target_root: &str,
        name: &str,
        cli: crate::domain::pack::value_objects::TargetCli,
    ) -> Result<(), ApplicationError>;
}
