use std::path::PathBuf;

/// The user configuration as loaded from the config file. Every key is
/// optional: an absent config file means an empty config.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppConfigDto {
    /// Writable pack library root (absolute), when configured. Overrides
    /// the XDG default location.
    pub packs_dir: Option<PathBuf>,
    /// Default target CLI for installs (`CliId::as_str()`), when configured.
    /// Used when the pack declares it; otherwise the pack's first declared
    /// CLI wins.
    pub default_cli: Option<String>,
}
