use std::path::Path;

use mockall::automock;

use crate::application::error::ApplicationError;

/// Port for preparing the pack library on disk (E01: the presentation no
/// longer does direct I/O; `main.rs` wires the infrastructure adapter).
#[automock]
pub trait LibraryInitializer: Send {
    /// Ensure the user library root exists (and is a directory). Idempotent.
    fn ensure_user_root(&self, root: &Path) -> Result<(), ApplicationError>;
}
