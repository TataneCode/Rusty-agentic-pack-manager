use std::path::PathBuf;

use serde::Deserialize;

use crate::application::config::dto::AppConfigDto;
use crate::application::config::traits::ConfigRepository;
use crate::application::error::ApplicationError;
use crate::domain::config::value_objects::LibraryPath;

/// Raw config file shape. Every key is optional.
#[derive(Debug, Default, Deserialize)]
struct RawConfig {
    #[serde(default)]
    packs_dir: Option<String>,
    #[serde(default)]
    default_cli: Option<String>,
}

/// Reads the user configuration file (`config.toml`). Read-only: the file
/// is edited by hand, never written by the application. A missing file is
/// an empty config; a malformed file or a relative `packs_dir` is an
/// `InvalidConfig` naming the file.
pub struct FsConfigRepository {
    path: PathBuf,
}

impl FsConfigRepository {
    pub fn new(path: PathBuf) -> FsConfigRepository {
        FsConfigRepository { path }
    }

    /// The default config file location from the environment:
    /// `$XDG_CONFIG_HOME/rapm/config.toml`, else `$HOME/.config/rapm/config.toml`,
    /// else `%APPDATA%\rapm\config.toml` on Windows.
    pub fn discover() -> Option<FsConfigRepository> {
        let path = config_file_path(
            std::env::var_os("XDG_CONFIG_HOME").as_deref(),
            std::env::var_os("HOME").as_deref(),
            std::env::var_os("APPDATA").as_deref(),
        )?;
        Some(FsConfigRepository::new(path))
    }

    fn invalid_config(&self, message: impl Into<String>) -> ApplicationError {
        ApplicationError::InvalidConfig {
            path: self.path.to_string_lossy().into_owned(),
            message: message.into(),
        }
    }
}

#[cfg_attr(not(windows), allow(unused_variables))]
fn config_file_path(
    xdg_config_home: Option<&std::ffi::OsStr>,
    home: Option<&std::ffi::OsStr>,
    appdata: Option<&std::ffi::OsStr>,
) -> Option<PathBuf> {
    if let Some(xdg) = xdg_config_home {
        if !xdg.is_empty() {
            return Some(std::path::Path::new(xdg).join("rapm").join("config.toml"));
        }
    }
    #[cfg(windows)]
    if let Some(appdata) = appdata {
        if !appdata.is_empty() {
            return Some(
                std::path::Path::new(appdata)
                    .join("rapm")
                    .join("config.toml"),
            );
        }
    }
    let home = home?;
    if home.is_empty() {
        return None;
    }
    Some(
        std::path::Path::new(home)
            .join(".config")
            .join("rapm")
            .join("config.toml"),
    )
}

impl ConfigRepository for FsConfigRepository {
    fn load(&self) -> Result<AppConfigDto, ApplicationError> {
        if !self.path.is_file() {
            return Ok(AppConfigDto::default());
        }
        let raw = std::fs::read_to_string(&self.path).map_err(|source| ApplicationError::Io {
            path: self.path.to_string_lossy().into_owned(),
            source,
        })?;
        let parsed: RawConfig =
            toml::from_str(&raw).map_err(|err| self.invalid_config(err.to_string()))?;
        let packs_dir = match parsed.packs_dir {
            None => None,
            Some(raw_dir) => Some(
                LibraryPath::new(PathBuf::from(&raw_dir))
                    .map_err(|err| self.invalid_config(err.to_string()))?
                    .into_path_buf(),
            ),
        };
        Ok(AppConfigDto {
            packs_dir,
            default_cli: parsed.default_cli,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn repo_at(tmp: &TempDir) -> FsConfigRepository {
        FsConfigRepository::new(tmp.path().join("config.toml"))
    }

    #[test]
    fn missing_file_is_an_empty_config_not_an_error() {
        let tmp = TempDir::new().unwrap();
        let dto = repo_at(&tmp).load().unwrap();
        assert_eq!(dto, AppConfigDto::default());
    }

    #[test]
    fn valid_config_loads_both_keys() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("config.toml"),
            r#"
packs_dir = "/home/u/my-packs"
default_cli = "gemini"
"#,
        )
        .unwrap();
        let dto = repo_at(&tmp).load().unwrap();
        assert_eq!(dto.packs_dir, Some(PathBuf::from("/home/u/my-packs")));
        assert_eq!(dto.default_cli.as_deref(), Some("gemini"));
    }

    #[test]
    fn partial_config_loads_with_the_other_key_absent() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("config.toml"), "default_cli = \"vibe\"\n").unwrap();
        let dto = repo_at(&tmp).load().unwrap();
        assert_eq!(dto.packs_dir, None);
        assert_eq!(dto.default_cli.as_deref(), Some("vibe"));
    }

    #[test]
    fn malformed_toml_is_an_invalid_config_naming_the_file() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("config.toml"), "not = [toml").unwrap();
        let err = repo_at(&tmp).load().unwrap_err();
        match err {
            ApplicationError::InvalidConfig { path, message } => {
                assert!(path.ends_with("config.toml"));
                assert!(!message.is_empty());
            }
            other => panic!("expected InvalidConfig, got {other:?}"),
        }
    }

    #[test]
    fn relative_packs_dir_is_rejected() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("config.toml"),
            "packs_dir = \"relative/packs\"\n",
        )
        .unwrap();
        let err = repo_at(&tmp).load().unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidConfig { .. }));
    }

    #[test]
    fn config_file_path_priority_matrix() {
        let p = config_file_path(
            Some(std::ffi::OsStr::new("/xdg-cfg")),
            Some(std::ffi::OsStr::new("/home/u")),
            Some(std::ffi::OsStr::new("C:/appdata")),
        )
        .unwrap();
        assert_eq!(p, PathBuf::from("/xdg-cfg/rapm/config.toml"));
        let p = config_file_path(
            Some(std::ffi::OsStr::new("")),
            Some(std::ffi::OsStr::new("/home/u")),
            None,
        )
        .unwrap();
        assert_eq!(p, PathBuf::from("/home/u/.config/rapm/config.toml"));
        assert!(config_file_path(None, None, None).is_none());
        assert!(config_file_path(None, Some(std::ffi::OsStr::new("")), None).is_none());
    }
}
