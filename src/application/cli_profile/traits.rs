use mockall::automock;

use crate::application::error::ApplicationError;
use crate::domain::cli_profile::profile::{CliProfile, CliProfileRegistry};
use crate::domain::cli_profile::value_objects::CliId;

/// Read access to the CLI profile registry (`clis/*.toml`, user library
/// first, then shipped/system locations). Decides whether a `CliId` is
/// usable and provides the destination templates used when a pack item has
/// no explicit destination for a CLI (E02).
#[automock]
pub trait CliProfileRepository: Send {
    /// List every loaded profile.
    fn list(&self) -> Result<Vec<CliProfile>, ApplicationError>;
    /// Load one profile by id; `UnknownCliProfile` when absent.
    fn find(&self, id: &CliId) -> Result<CliProfile, ApplicationError>;
    /// The whole registry (for validating pack manifests against it).
    fn registry(&self) -> Result<CliProfileRegistry, ApplicationError>;
}
