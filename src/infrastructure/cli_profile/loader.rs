use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::application::cli_profile::traits::CliProfileRepository;
use crate::application::error::ApplicationError;
use crate::domain::cli_profile::profile::{CliProfile, CliProfileRegistry, DestinationTemplates};
use crate::domain::cli_profile::value_objects::CliId;

/// Raw `clis/<name>.toml` shape.
#[derive(Debug, Deserialize)]
struct RawProfile {
    cli: RawCli,
    #[serde(default)]
    defaults: RawDefaults,
}

#[derive(Debug, Deserialize)]
struct RawCli {
    name: String,
    #[serde(default)]
    label: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawDefaults {
    #[serde(default)]
    skill: Option<String>,
    #[serde(default)]
    skill_dir: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    agent_dir: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    command_dir: Option<String>,
}

impl RawProfile {
    fn into_profile(self, path: &Path) -> Result<CliProfile, ApplicationError> {
        let id = CliId::new(&self.cli.name).map_err(|err| ApplicationError::InvalidManifest {
            path: path.to_string_lossy().into_owned(),
            message: err.to_string(),
        })?;
        let missing = |field: &str| ApplicationError::InvalidManifest {
            path: path.to_string_lossy().into_owned(),
            message: format!("CLI profile is missing defaults.{field}"),
        };
        let label = if self.cli.label.is_empty() {
            id.as_str().to_string()
        } else {
            self.cli.label
        };
        let templates = DestinationTemplates {
            skill: self.defaults.skill.ok_or_else(|| missing("skill"))?,
            skill_dir: self.defaults.skill_dir,
            agent: self.defaults.agent.ok_or_else(|| missing("agent"))?,
            agent_dir: self.defaults.agent_dir,
            command: self.defaults.command.ok_or_else(|| missing("command"))?,
            command_dir: self.defaults.command_dir,
        };
        Ok(CliProfile::new(id, label, templates)?)
    }
}

/// Loads CLI profiles from an ordered list of roots (user library first,
/// then shipped/system locations). The first root containing a profile
/// named `<id>.toml` wins: a user profile overrides a shipped one of the
/// same name. Malformed profile files are skipped, like malformed pack
/// dirs.
#[derive(Debug, Clone)]
pub struct FsCliProfileRepository {
    roots: Vec<PathBuf>,
}

impl FsCliProfileRepository {
    pub fn new(roots: Vec<PathBuf>) -> FsCliProfileRepository {
        FsCliProfileRepository { roots }
    }

    fn load_file(&self, path: &Path) -> Result<CliProfile, ApplicationError> {
        let raw = std::fs::read_to_string(path).map_err(|source| ApplicationError::Io {
            path: path.to_string_lossy().into_owned(),
            source,
        })?;
        let parsed: RawProfile =
            toml::from_str(&raw).map_err(|err| ApplicationError::InvalidManifest {
                path: path.to_string_lossy().into_owned(),
                message: err.to_string(),
            })?;
        parsed.into_profile(path)
    }

    fn load_all(&self) -> Vec<CliProfile> {
        let mut profiles: Vec<CliProfile> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for root in &self.roots {
            let Ok(entries) = std::fs::read_dir(root) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "toml") {
                    if let Ok(profile) = self.load_file(&path) {
                        // First root wins: a user profile overrides a
                        // shipped profile of the same name.
                        if seen.insert(profile.id.as_str().to_string()) {
                            profiles.push(profile);
                        }
                    }
                }
            }
        }
        profiles.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        profiles
    }
}

impl CliProfileRepository for FsCliProfileRepository {
    fn list(&self) -> Result<Vec<CliProfile>, ApplicationError> {
        Ok(self.load_all())
    }

    fn find(&self, id: &CliId) -> Result<CliProfile, ApplicationError> {
        for root in &self.roots {
            let path = root.join(format!("{}.toml", id.as_str()));
            if path.is_file() {
                return self.load_file(&path);
            }
        }
        Err(ApplicationError::Domain(
            crate::domain::pack::error::DomainError::UnknownCliProfile(id.as_str().to_string()),
        ))
    }

    fn registry(&self) -> Result<CliProfileRegistry, ApplicationError> {
        Ok(CliProfileRegistry::new(self.load_all()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const CLAUDE: &str = r#"
[cli]
name = "claude"
label = "Claude Code"
[defaults]
skill   = ".claude/skills/{name}/SKILL.md"
skill_dir = ".claude/skills/{name}/"
agent   = ".claude/agents/{name}.md"
command = ".claude/commands/{name}.md"
"#;

    const VIBE: &str = r#"
[cli]
name = "vibe"
[defaults]
skill   = ".agents/skills/{name}/SKILL.md"
agent   = ".vibe/agents/{name}.toml"
command = ".vibe/commands/{name}.toml"
"#;

    fn write_profile(root: &Path, file: &str, content: &str) {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join(file), content).unwrap();
    }

    #[test]
    fn loads_profiles_from_all_roots_sorted_by_id() {
        let user = TempDir::new().unwrap();
        let ship = TempDir::new().unwrap();
        write_profile(user.path(), "claude.toml", CLAUDE);
        write_profile(ship.path(), "vibe.toml", VIBE);

        let repo =
            FsCliProfileRepository::new(vec![user.path().to_path_buf(), ship.path().to_path_buf()]);
        let profiles = repo.list().unwrap();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].id.as_str(), "claude");
        assert_eq!(profiles[0].label, "Claude Code");
        assert_eq!(profiles[1].id.as_str(), "vibe");
        // label defaults to the id when omitted
        assert_eq!(profiles[1].label, "vibe");
    }

    #[test]
    fn user_profile_overrides_shipped_profile_of_same_name() {
        let user = TempDir::new().unwrap();
        let ship = TempDir::new().unwrap();
        write_profile(ship.path(), "claude.toml", CLAUDE);
        let user_claude = CLAUDE.replace("Claude Code", "My Claude");
        write_profile(user.path(), "claude.toml", &user_claude);

        let repo =
            FsCliProfileRepository::new(vec![user.path().to_path_buf(), ship.path().to_path_buf()]);
        let profile = repo.find(&CliId::new("claude").unwrap()).unwrap();
        assert_eq!(profile.label, "My Claude");
        assert_eq!(repo.list().unwrap().len(), 1);
    }

    #[test]
    fn find_falls_back_to_later_roots() {
        let user = TempDir::new().unwrap();
        let ship = TempDir::new().unwrap();
        write_profile(ship.path(), "vibe.toml", VIBE);
        let repo =
            FsCliProfileRepository::new(vec![user.path().to_path_buf(), ship.path().to_path_buf()]);
        assert_eq!(
            repo.find(&CliId::new("vibe").unwrap()).unwrap().id.as_str(),
            "vibe"
        );
    }

    #[test]
    fn find_unknown_profile_is_an_unknown_cli_profile_error() {
        let tmp = TempDir::new().unwrap();
        let repo = FsCliProfileRepository::new(vec![tmp.path().to_path_buf()]);
        let err = repo.find(&CliId::new("nope").unwrap()).unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(crate::domain::pack::error::DomainError::UnknownCliProfile(
                _
            ))
        ));
    }

    #[test]
    fn malformed_profiles_are_skipped() {
        let tmp = TempDir::new().unwrap();
        write_profile(tmp.path(), "broken.toml", "not = [toml");
        write_profile(tmp.path(), "vibe.toml", VIBE);
        write_profile(
            tmp.path(),
            "incomplete.toml",
            r#"
[cli]
name = "incomplete"
[defaults]
skill = ".x/{name}"
"#,
        );
        let repo = FsCliProfileRepository::new(vec![tmp.path().to_path_buf()]);
        let profiles = repo.list().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id.as_str(), "vibe");
    }

    #[test]
    fn missing_roots_and_non_toml_files_are_ignored() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("README.md"), "docs").unwrap();
        let repo = FsCliProfileRepository::new(vec![
            tmp.path().join("does-not-exist"),
            tmp.path().to_path_buf(),
        ]);
        assert!(repo.list().unwrap().is_empty());
        let registry = repo.registry().unwrap();
        assert!(registry.is_empty());
    }

    #[test]
    fn registry_validates_raw_ids_against_loaded_profiles() {
        let tmp = TempDir::new().unwrap();
        write_profile(tmp.path(), "vibe.toml", VIBE);
        let repo = FsCliProfileRepository::new(vec![tmp.path().to_path_buf()]);
        let registry = repo.registry().unwrap();
        assert_eq!(registry.len(), 1);
        assert!(registry.validate("vibe").is_ok());
        assert!(registry.validate("claude").is_err());
    }
}
