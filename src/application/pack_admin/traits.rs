use std::path::{Path, PathBuf};

use mockall::automock;

use crate::application::error::ApplicationError;
use crate::application::pack_admin::dto::{PackSourceDto, PackWriteOutcomeDto, ScannedItem};
use crate::domain::pack::entity::Pack;

/// Write access to the pack library (add and remove packs).
#[automock]
pub trait PackWriter: Send {
    /// Persist a new pack: copy every source into the pack directory and
    /// write the `pack.toml` manifest. `sources` pairs each host path with
    /// its canonical path inside the pack directory. When a pack with the
    /// same name already exists, it is replaced (the caller has gated this
    /// on the user's `overwrite` flag).
    fn write(
        &self,
        pack: &Pack,
        sources: &[PackSourceDto],
    ) -> Result<PackWriteOutcomeDto, ApplicationError>;

    /// Remove the pack `name` from the writable library root and return
    /// the removed directory. Fails with `PackNotFound` when the root
    /// holds no such pack — system roots are read-only and never touched.
    fn remove(&self, name: &str) -> Result<PathBuf, ApplicationError>;
}

/// Scans a base directory for pack content (E13): each entry of its
/// `agents/`, `skills/`, `rules/`, `commands/` subdirectories is one item.
#[automock]
pub trait BaseDirScanner: Send {
    /// One entry per item found under the kind directories of `base`,
    /// sorted by (kind, name). Empty when no kind directory exists.
    fn scan(&self, base: &Path) -> Result<Vec<ScannedItem>, ApplicationError>;
}
