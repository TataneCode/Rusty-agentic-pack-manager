use mockall::automock;

use crate::application::error::ApplicationError;
use crate::application::pack_create::dto::{PackSourceDto, PackWriteOutcomeDto};
use crate::domain::pack::entity::Pack;

/// Write access to the pack library for pack creation.
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
}
