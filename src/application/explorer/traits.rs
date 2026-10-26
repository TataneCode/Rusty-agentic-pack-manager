use mockall::automock;

use crate::application::error::ApplicationError;
use crate::application::explorer::dto::DirectoryListingDto;

/// Read/write access to the host filesystem for the explorer panel.
#[automock]
pub trait ExplorerRepository: Send {
    /// List `path` as a directory. Errors if it is not a directory.
    fn list(&self, path: &str) -> Result<DirectoryListingDto, ApplicationError>;
    /// Create a directory (non-recursive parents must exist) inside `parent`.
    fn create_dir(&self, parent: &str, name: &str) -> Result<(), ApplicationError>;
    /// Rename `from` into `to`, both inside `parent`.
    fn rename(&self, parent: &str, from: &str, to: &str) -> Result<(), ApplicationError>;
    /// Recursively remove `name` inside `parent`.
    fn remove(&self, parent: &str, name: &str) -> Result<(), ApplicationError>;
}
