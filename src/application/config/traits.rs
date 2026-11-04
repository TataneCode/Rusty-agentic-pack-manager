use mockall::automock;

use crate::application::config::dto::AppConfigDto;
use crate::application::error::ApplicationError;

/// Read access to the user configuration file.
#[automock]
pub trait ConfigRepository: Send {
    /// Load the user configuration. A missing config file is an empty
    /// config, not an error; a malformed file is an error naming the path.
    fn load(&self) -> Result<AppConfigDto, ApplicationError>;
}
