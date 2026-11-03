use std::collections::HashSet;
use std::path::PathBuf;

use crate::application::error::ApplicationError;
use crate::application::pack::traits::PackRepository;
use crate::application::pack_admin::dto::{
    AddPackDto, AddPackResultDto, PackSourceDto, RemovePackResultDto, ScannedItem,
};
use crate::application::pack_admin::traits::{BaseDirScanner, PackWriter};
use crate::domain::cli_profile::profile::CliProfileRegistry;
use crate::domain::cli_profile::value_objects::CliId;
use crate::domain::pack::entity::{Pack, PackItem};
use crate::domain::pack::error::DomainError;
use crate::domain::pack::value_objects::{PackName, PackVersion, TargetCliMap};

/// Application service for the pack library administration (E13):
/// `add_from_dir` imports a base directory as a new pack (the `pack.toml`
/// manifest is generated, with one destination per loaded CLI profile),
/// `remove` deletes a pack from the writable library root. Every check
/// runs before the write: a validation error leaves no pack behind.
pub struct PackAdminService {
    repository: Box<dyn PackRepository>,
    writer: Box<dyn PackWriter>,
    scanner: Box<dyn BaseDirScanner>,
    profiles: CliProfileRegistry,
}

impl PackAdminService {
    pub fn new(
        repository: Box<dyn PackRepository>,
        writer: Box<dyn PackWriter>,
        scanner: Box<dyn BaseDirScanner>,
    ) -> PackAdminService {
        PackAdminService {
            repository,
            writer,
            scanner,
            profiles: CliProfileRegistry::default(),
        }
    }

    /// CLI profiles used to generate the destinations of the imported
    /// items: one destination per profile that has a template for the
    /// item's kind.
    pub fn with_profiles(mut self, profiles: CliProfileRegistry) -> Self {
        self.profiles = profiles;
        self
    }

    /// Import `dto.dir` as a pack of the library: scan its kind
    /// directories, build the domain `Pack` (canonical `<kind>s/<name>`
    /// sources, generated destinations), then write it through the port.
    pub fn add_from_dir(&self, dto: &AddPackDto) -> Result<AddPackResultDto, ApplicationError> {
        let name = match &dto.name {
            Some(raw) => PackName::new(raw)?,
            None => {
                let dir_name = dto
                    .dir
                    .file_name()
                    .ok_or(DomainError::EmptyName)?
                    .to_string_lossy()
                    .into_owned();
                PackName::new(&dir_name)?
            }
        };

        if !dto.overwrite && self.repository.find(name.as_str()).is_ok() {
            return Err(ApplicationError::PackAlreadyExists(
                name.as_str().to_string(),
            ));
        }

        let scanned = self.scanner.scan(&dto.dir)?;
        if scanned.is_empty() {
            return Err(ApplicationError::NoPackContent(
                dto.dir.to_string_lossy().into_owned(),
            ));
        }

        let (items, sources) = build_items(&scanned, &self.profiles)?;

        // The pack's CLI list is the union of the generated destinations,
        // in registry order (deterministic).
        let mut clis: Vec<CliId> = Vec::new();
        for (cli, _) in self.profiles.iter() {
            if items
                .iter()
                .any(|item| item.destinations.get(cli).is_some())
            {
                clis.push(cli.clone());
            }
        }

        let item_count = items.len();
        let pack = Pack::new(
            name.clone(),
            PackVersion::new("1.0.0")?,
            String::new(),
            clis,
            items,
        )?;
        let outcome = self.writer.write(&pack, &sources)?;
        Ok(AddPackResultDto {
            pack_name: pack.name.as_str().to_string(),
            pack_dir: outcome.pack_dir,
            item_count,
        })
    }

    /// Remove the pack `name` from the writable library root. The name is
    /// validated first: only the user library is ever touched, never a
    /// system root.
    pub fn remove(&self, name: &str) -> Result<RemovePackResultDto, ApplicationError> {
        let name = PackName::new(name)?;
        let pack_dir = self.writer.remove(name.as_str())?;
        Ok(RemovePackResultDto {
            pack_name: name.as_str().to_string(),
            pack_dir,
        })
    }
}

/// Build the domain items plus the copy plan from the scanned entries.
/// Destinations are generated per loaded profile via its templates;
/// profiles lacking a template for an item's kind are skipped for that
/// item. Rejects duplicate canonical sources (they would collide under
/// `<kind>s/<name>`).
fn build_items(
    scanned: &[ScannedItem],
    profiles: &CliProfileRegistry,
) -> Result<(Vec<PackItem>, Vec<PackSourceDto>), ApplicationError> {
    let mut items = Vec::new();
    let mut sources = Vec::new();
    let mut claimed: HashSet<PathBuf> = HashSet::new();
    for entry in scanned {
        if !claimed.insert(entry.pack_source.clone()) {
            return Err(ApplicationError::Domain(DomainError::DuplicateItemSource(
                entry.pack_source.to_string_lossy().into_owned(),
            )));
        }
        let probe = PackItem::new(
            entry.kind,
            entry.pack_source.clone(),
            TargetCliMap::default(),
        )?;
        let mut destinations = TargetCliMap::default();
        for (cli, profile) in profiles.iter() {
            if let Ok(destination) = profile.destination(&probe, entry.source_is_dir) {
                destinations.insert(cli.clone(), destination);
            }
        }
        items.push(PackItem::new(
            entry.kind,
            entry.pack_source.clone(),
            destinations,
        )?);
        sources.push(PackSourceDto {
            host_path: entry.host_path.clone(),
            pack_source: entry.pack_source.clone(),
        });
    }
    Ok((items, sources))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::pack::traits::MockPackRepository;
    use crate::application::pack_admin::dto::PackWriteOutcomeDto;
    use crate::application::pack_admin::traits::{MockBaseDirScanner, MockPackWriter};
    use crate::domain::cli_profile::profile::{CliProfile, DestinationTemplates};

    fn test_registry(ids: &[&str]) -> CliProfileRegistry {
        let profile = |id: &str| {
            CliProfile::new(
                CliId::new(id).unwrap(),
                id.to_string(),
                DestinationTemplates {
                    skill: ".agents/skills/{name}/SKILL.md".to_string(),
                    skill_dir: Some(".agents/skills/{name}/".to_string()),
                    agent: ".agents/{name}.toml".to_string(),
                    agent_dir: None,
                    rule: Some(".rules/{name}.md".to_string()),
                    rule_dir: None,
                    command: ".agents/{name}.toml".to_string(),
                    command_dir: None,
                },
            )
            .unwrap()
        };
        CliProfileRegistry::new(ids.iter().map(|id| profile(id)).collect())
    }

    fn scanned() -> Vec<ScannedItem> {
        vec![
            ScannedItem {
                kind: crate::domain::pack::value_objects::PackItemKind::Skill,
                host_path: PathBuf::from("/base/skills/tdd"),
                pack_source: PathBuf::from("skills/tdd"),
                source_is_dir: true,
            },
            ScannedItem {
                kind: crate::domain::pack::value_objects::PackItemKind::Rule,
                host_path: PathBuf::from("/base/rules/no-panic.md"),
                pack_source: PathBuf::from("rules/no-panic.md"),
                source_is_dir: false,
            },
        ]
    }

    fn repo_with(exists: bool) -> MockPackRepository {
        let mut repo = MockPackRepository::new();
        repo.expect_find().times(0..).returning(move |name| {
            if exists {
                Ok(Pack::new(
                    PackName::new(name).unwrap(),
                    PackVersion::new("1.0.0").unwrap(),
                    String::new(),
                    vec![CliId::new("vibe").unwrap()],
                    vec![],
                )
                .unwrap())
            } else {
                Err(ApplicationError::PackNotFound(name.to_string()))
            }
        });
        repo
    }

    fn scanner_with(items: Vec<ScannedItem>) -> MockBaseDirScanner {
        let mut scanner = MockBaseDirScanner::new();
        scanner.expect_scan().returning(move |_| Ok(items.clone()));
        scanner
    }

    fn ok_outcome() -> PackWriteOutcomeDto {
        PackWriteOutcomeDto {
            pack_dir: PathBuf::from("/lib/my-pack"),
            copied_count: 2,
        }
    }

    fn service(
        exists: bool,
        writer: MockPackWriter,
        scanner: MockBaseDirScanner,
    ) -> PackAdminService {
        PackAdminService::new(
            Box::new(repo_with(exists)),
            Box::new(writer),
            Box::new(scanner),
        )
        .with_profiles(test_registry(&["vibe", "gemini"]))
    }

    #[test]
    fn add_from_dir_scans_generates_destinations_and_writes() {
        let mut writer = MockPackWriter::new();
        writer
            .expect_write()
            .times(1)
            .withf(|pack, sources| {
                pack.name.as_str() == "my-pack"
                    && pack.version.as_str() == "1.0.0"
                    && pack.clis == vec![CliId::new("gemini").unwrap(), CliId::new("vibe").unwrap()]
                    && pack.items.len() == 2
                    && pack.items[0].kind == crate::domain::pack::value_objects::PackItemKind::Skill
                    && pack.items[0].source == std::path::Path::new("skills/tdd")
                    && pack.items[0]
                        .destination_for(&CliId::new("vibe").unwrap())
                        .as_deref()
                        == Some(".agents/skills/tdd/")
                    && pack.items[1].kind == crate::domain::pack::value_objects::PackItemKind::Rule
                    && pack.items[1]
                        .destination_for(&CliId::new("gemini").unwrap())
                        .as_deref()
                        == Some(".rules/no-panic.md")
                    && sources.len() == 2
                    && sources[0].host_path == std::path::Path::new("/base/skills/tdd")
                    && sources[0].pack_source == std::path::Path::new("skills/tdd")
            })
            .returning(|_, _| Ok(ok_outcome()));
        let dto = AddPackDto {
            dir: PathBuf::from("/base"),
            name: Some("my-pack".to_string()),
            overwrite: false,
        };
        let service = service(false, writer, scanner_with(scanned()));
        let result = service.add_from_dir(&dto).unwrap();
        assert_eq!(result.pack_name, "my-pack");
        assert_eq!(result.item_count, 2);
        assert_eq!(result.pack_dir, PathBuf::from("/lib/my-pack"));
    }

    #[test]
    fn add_from_dir_defaults_the_name_to_the_directory_name() {
        let mut writer = MockPackWriter::new();
        writer
            .expect_write()
            .times(1)
            .withf(|pack, _| pack.name.as_str() == "base")
            .returning(|_, _| Ok(ok_outcome()));
        let dto = AddPackDto {
            dir: PathBuf::from("/somewhere/base"),
            name: None,
            overwrite: false,
        };
        let service = service(false, writer, scanner_with(scanned()));
        let result = service.add_from_dir(&dto).unwrap();
        assert_eq!(result.pack_name, "base");
    }

    #[test]
    fn add_from_dir_rejects_an_existing_pack_without_overwrite() {
        let mut writer = MockPackWriter::new();
        writer.expect_write().never();
        let dto = AddPackDto {
            dir: PathBuf::from("/base"),
            name: Some("my-pack".to_string()),
            overwrite: false,
        };
        let service = service(true, writer, scanner_with(scanned()));
        let err = service.add_from_dir(&dto).unwrap_err();
        assert!(matches!(err, ApplicationError::PackAlreadyExists(_)));
    }

    #[test]
    fn add_from_dir_with_overwrite_writes_anyway() {
        let mut writer = MockPackWriter::new();
        writer
            .expect_write()
            .times(1)
            .returning(|_, _| Ok(ok_outcome()));
        let dto = AddPackDto {
            dir: PathBuf::from("/base"),
            name: Some("my-pack".to_string()),
            overwrite: true,
        };
        let service = service(true, writer, scanner_with(scanned()));
        assert!(service.add_from_dir(&dto).is_ok());
    }

    #[test]
    fn add_from_dir_rejects_an_empty_base_directory_before_scanning_results_are_used() {
        let mut writer = MockPackWriter::new();
        writer.expect_write().never();
        let dto = AddPackDto {
            dir: PathBuf::from("/base"),
            name: Some("my-pack".to_string()),
            overwrite: false,
        };
        let service = service(false, writer, scanner_with(vec![]));
        let err = service.add_from_dir(&dto).unwrap_err();
        assert!(matches!(err, ApplicationError::NoPackContent(_)));
    }

    #[test]
    fn add_from_dir_rejects_duplicate_canonical_sources() {
        let mut writer = MockPackWriter::new();
        writer.expect_write().never();
        let dup = ScannedItem {
            kind: crate::domain::pack::value_objects::PackItemKind::Skill,
            host_path: PathBuf::from("/base/skills/tdd-copy"),
            pack_source: PathBuf::from("skills/tdd"),
            source_is_dir: true,
        };
        let mut items = scanned();
        items.push(dup);
        let dto = AddPackDto {
            dir: PathBuf::from("/base"),
            name: Some("my-pack".to_string()),
            overwrite: false,
        };
        let service = service(false, writer, scanner_with(items));
        let err = service.add_from_dir(&dto).unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::DuplicateItemSource(_))
        ));
    }

    #[test]
    fn add_from_dir_skips_profiles_without_a_template_for_the_kind() {
        // Registry with a single profile that has no rule template.
        let no_rule = CliProfile::new(
            CliId::new("vibe").unwrap(),
            "vibe".to_string(),
            DestinationTemplates {
                skill: ".agents/skills/{name}/SKILL.md".to_string(),
                skill_dir: Some(".agents/skills/{name}/".to_string()),
                agent: ".agents/{name}.toml".to_string(),
                agent_dir: None,
                rule: None,
                rule_dir: None,
                command: ".agents/{name}.toml".to_string(),
                command_dir: None,
            },
        )
        .unwrap();
        let registry = CliProfileRegistry::new(vec![no_rule]);
        let mut writer = MockPackWriter::new();
        writer
            .expect_write()
            .times(1)
            .withf(|pack, _| {
                // The rule item has no destination for vibe, the skill does.
                pack.items[1].destinations.is_empty()
                    && pack.items[0]
                        .destination_for(&CliId::new("vibe").unwrap())
                        .is_some()
            })
            .returning(|_, _| Ok(ok_outcome()));
        let service = PackAdminService::new(
            Box::new(repo_with(false)),
            Box::new(writer),
            Box::new(scanner_with(scanned())),
        )
        .with_profiles(registry);
        let dto = AddPackDto {
            dir: PathBuf::from("/base"),
            name: Some("my-pack".to_string()),
            overwrite: false,
        };
        assert!(service.add_from_dir(&dto).is_ok());
    }

    #[test]
    fn remove_delegates_to_the_writer_with_a_validated_name() {
        let mut writer = MockPackWriter::new();
        writer
            .expect_remove()
            .times(1)
            .with(mockall::predicate::eq("my-pack"))
            .returning(|_| Ok(PathBuf::from("/lib/my-pack")));
        let service = PackAdminService::new(
            Box::new(repo_with(false)),
            Box::new(writer),
            Box::new(scanner_with(vec![])),
        );
        let result = service.remove("my-pack").unwrap();
        assert_eq!(result.pack_name, "my-pack");
        assert_eq!(result.pack_dir, PathBuf::from("/lib/my-pack"));
    }

    #[test]
    fn remove_rejects_an_invalid_name_without_touching_the_writer() {
        let mut writer = MockPackWriter::new();
        writer.expect_remove().never();
        let service = PackAdminService::new(
            Box::new(repo_with(false)),
            Box::new(writer),
            Box::new(scanner_with(vec![])),
        );
        assert!(service.remove("../escape").is_err());
    }
}
