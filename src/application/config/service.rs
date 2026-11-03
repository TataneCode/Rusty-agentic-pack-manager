use crate::application::config::dto::AppConfigDto;
use crate::application::config::traits::ConfigRepository;
use crate::application::error::ApplicationError;
use crate::domain::cli_profile::value_objects::CliId;

/// Application service for the user configuration: loads it through the
/// repository port and validates the raw values (`packs_dir` is validated
/// by the adapter via the `LibraryPath` invariant; `default_cli` must be a
/// well-formed CLI id — whether a profile exists for it is checked at the
/// DI point against the registry).
pub struct ConfigService {
    repository: Box<dyn ConfigRepository>,
}

impl ConfigService {
    pub fn new(repository: Box<dyn ConfigRepository>) -> ConfigService {
        ConfigService { repository }
    }

    pub fn load(&self) -> Result<AppConfigDto, ApplicationError> {
        let dto = self.repository.load()?;
        if let Some(cli) = &dto.default_cli {
            CliId::new(cli)?;
        }
        Ok(dto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::config::traits::MockConfigRepository;
    use std::path::PathBuf;

    #[test]
    fn load_passes_a_valid_config_through() {
        let mut repository = MockConfigRepository::new();
        repository.expect_load().returning(|| {
            Ok(AppConfigDto {
                packs_dir: Some(PathBuf::from("/abs/packs")),
                default_cli: Some("vibe".to_string()),
            })
        });
        let service = ConfigService::new(Box::new(repository));
        let dto = service.load().unwrap();
        assert_eq!(dto.packs_dir, Some(PathBuf::from("/abs/packs")));
        assert_eq!(dto.default_cli.as_deref(), Some("vibe"));
    }

    #[test]
    fn load_defaults_to_an_empty_config() {
        let mut repository = MockConfigRepository::new();
        repository
            .expect_load()
            .returning(|| Ok(AppConfigDto::default()));
        let service = ConfigService::new(Box::new(repository));
        let dto = service.load().unwrap();
        assert_eq!(dto, AppConfigDto::default());
    }

    #[test]
    fn load_rejects_a_malformed_default_cli() {
        let mut repository = MockConfigRepository::new();
        repository.expect_load().returning(|| {
            Ok(AppConfigDto {
                packs_dir: None,
                default_cli: Some("Not An Id".to_string()),
            })
        });
        let service = ConfigService::new(Box::new(repository));
        assert!(service.load().is_err());
    }
}
