use std::collections::HashSet;
use std::path::PathBuf;

use crate::application::error::ApplicationError;
use crate::application::pack::traits::PackRepository;
use crate::application::pack_create::dto::{
    CreateItemDto, CreatePackDto, CreatePackResultDto, PackSourceDto,
};
use crate::application::pack_create::traits::PackWriter;
use crate::domain::cli_profile::profile::CliProfileRegistry;
use crate::domain::cli_profile::value_objects::CliId;
use crate::domain::pack::entity::{Pack, PackItem};
use crate::domain::pack::error::DomainError;
use crate::domain::pack::value_objects::{PackItemKind, PackName, PackVersion, TargetCliMap};

/// Application service for `rapm pack create`: validates the input, builds
/// the domain `Pack` (canonical `<kind>s/<name>` sources, one destination
/// per declared CLI), then delegates the filesystem write to the port.
/// Every check runs before the write: a validation error leaves no pack
/// directory behind.
pub struct PackCreationService {
    repository: Box<dyn PackRepository>,
    writer: Box<dyn PackWriter>,
    profiles: CliProfileRegistry,
}

impl PackCreationService {
    pub fn new(repository: Box<dyn PackRepository>, writer: Box<dyn PackWriter>) -> Self {
        PackCreationService {
            repository,
            writer,
            profiles: CliProfileRegistry::default(),
        }
    }

    /// CLI profiles validating `--cli` and destination keys (E02): a CLI
    /// with a loaded profile is accepted even if unknown to the v1 enum.
    pub fn with_profiles(mut self, profiles: CliProfileRegistry) -> Self {
        self.profiles = profiles;
        self
    }

    pub fn create(&self, dto: &CreatePackDto) -> Result<CreatePackResultDto, ApplicationError> {
        let name = PackName::new(&dto.name)?;
        let version = PackVersion::new(&dto.version)?;

        if !dto.overwrite && self.repository.find(name.as_str()).is_ok() {
            return Err(ApplicationError::PackAlreadyExists(
                name.as_str().to_string(),
            ));
        }

        let declared_clis = parse_clis(&dto.clis, &self.profiles)?;
        let (items, sources) = build_items(&dto.items, &self.profiles)?;

        // CLI list: explicit --cli, else the union of the items' destinations.
        let clis = if declared_clis.is_empty() {
            let mut union: Vec<CliId> = Vec::new();
            for item in &items {
                for (cli, _) in item.destinations.iter() {
                    if !union.contains(cli) {
                        union.push(cli.clone());
                    }
                }
            }
            union
        } else {
            declared_clis
        };

        // Every item must have a destination for every declared CLI.
        let missing = items
            .iter()
            .any(|item| clis.iter().any(|cli| item.destinations.get(cli).is_none()));
        if missing {
            return Err(ApplicationError::Domain(DomainError::EmptyItemDestination));
        }

        let pack = Pack::new(name.clone(), version, dto.description.clone(), clis, items)?;
        // The writer replaces an existing pack directory only when it is
        // reached, which the check above has already gated on `overwrite`.
        let outcome = self.writer.write(&pack, &sources)?;
        Ok(CreatePackResultDto {
            pack_name: pack.name.as_str().to_string(),
            pack_dir: outcome.pack_dir,
            copied_count: outcome.copied_count,
        })
    }
}

fn parse_clis(
    raw: &[String],
    profiles: &CliProfileRegistry,
) -> Result<Vec<CliId>, ApplicationError> {
    let mut clis = Vec::new();
    for c in raw {
        let cli = profiles.validate(c)?;
        if !clis.contains(&cli) {
            clis.push(cli);
        }
    }
    Ok(clis)
}

/// Parse the item DTOs into domain items plus the copy plan (host path ->
/// canonical source inside the pack). Rejects duplicate canonical sources
/// (they would collide under `<kind>s/<name>`).
fn build_items(
    dtos: &[CreateItemDto],
    profiles: &CliProfileRegistry,
) -> Result<(Vec<PackItem>, Vec<PackSourceDto>), ApplicationError> {
    let mut items = Vec::new();
    let mut sources = Vec::new();
    let mut claimed: HashSet<PathBuf> = HashSet::new();
    for dto in dtos {
        let kind = PackItemKind::parse(&dto.kind)?;
        let file_name = dto.source.file_name().ok_or(DomainError::EmptyItemSource)?;
        let pack_source = PathBuf::from(kind.plural_dir()).join(file_name);
        if !claimed.insert(pack_source.clone()) {
            return Err(ApplicationError::Domain(DomainError::DuplicateItemSource(
                pack_source.to_string_lossy().into_owned(),
            )));
        }
        let mut destinations = TargetCliMap::default();
        for (cli_raw, dest) in &dto.destinations {
            let cli = profiles.validate(cli_raw)?;
            // Fail fast on destinations that could never install (absolute,
            // escaping, or root-only paths): the same rule the domain
            // enforces at install time.
            crate::domain::pack::error::validate_item_destination(dest)?;
            destinations.insert(cli, dest.clone());
        }
        items.push(PackItem::new(kind, pack_source.clone(), destinations)?);
        sources.push(PackSourceDto {
            host_path: dto.source.clone(),
            pack_source,
        });
    }
    Ok((items, sources))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use crate::application::pack::traits::MockPackRepository;
    use crate::application::pack_create::dto::PackWriteOutcomeDto;
    use crate::application::pack_create::traits::MockPackWriter;

    fn item_dto(kind: &str, source: &str, destinations: &[(&str, &str)]) -> CreateItemDto {
        CreateItemDto {
            kind: kind.to_string(),
            source: PathBuf::from(source),
            destinations: destinations
                .iter()
                .map(|(c, d)| (c.to_string(), d.to_string()))
                .collect(),
        }
    }

    fn dto(items: Vec<CreateItemDto>) -> CreatePackDto {
        CreatePackDto {
            name: "my-pack".to_string(),
            version: "1.0.0".to_string(),
            description: "desc".to_string(),
            clis: vec![],
            items,
            overwrite: false,
        }
    }

    fn sample_dto() -> CreatePackDto {
        dto(vec![item_dto(
            "skill",
            "./my-skills/tdd",
            &[
                ("vibe", ".agents/skills/tdd/"),
                ("gemini", ".gemini/skills/tdd/"),
            ],
        )])
    }

    fn existing_pack(name: &str) -> Pack {
        Pack::new(
            PackName::new(name).unwrap(),
            PackVersion::new("1.0.0").unwrap(),
            String::new(),
            vec![CliId::new("vibe").unwrap()],
            vec![],
        )
        .unwrap()
    }

    /// A repository where `find("my-pack")` answers according to `exists`.
    fn test_registry(ids: &[&str]) -> crate::domain::cli_profile::profile::CliProfileRegistry {
        use crate::domain::cli_profile::profile::{
            CliProfile, CliProfileRegistry, DestinationTemplates,
        };
        let profile = |id: &str| {
            CliProfile::new(
                CliId::new(id).unwrap(),
                id.to_string(),
                DestinationTemplates {
                    skill: ".agents/skills/{name}/SKILL.md".to_string(),
                    skill_dir: Some(".agents/skills/{name}/".to_string()),
                    agent: ".agents/{name}.toml".to_string(),
                    agent_dir: None,
                    command: ".agents/{name}.toml".to_string(),
                    command_dir: None,
                },
            )
            .unwrap()
        };
        CliProfileRegistry::new(ids.iter().map(|id| profile(id)).collect())
    }

    fn service_with_registry(
        exists: bool,
        registry: crate::domain::cli_profile::profile::CliProfileRegistry,
        writer: MockPackWriter,
    ) -> PackCreationService {
        PackCreationService::new(Box::new(repo_with(exists)), Box::new(writer))
            .with_profiles(registry)
    }

    fn repo_with(exists: bool) -> MockPackRepository {
        let mut repo = MockPackRepository::new();
        repo.expect_find().times(0..).returning(move |name| {
            if exists {
                Ok(existing_pack(name))
            } else {
                Err(ApplicationError::PackNotFound(name.to_string()))
            }
        });
        repo
    }

    fn ok_outcome() -> PackWriteOutcomeDto {
        PackWriteOutcomeDto {
            pack_dir: PathBuf::from("/lib/my-pack"),
            copied_count: 1,
        }
    }

    #[test]
    fn create_builds_canonical_sources_and_deduces_clis() {
        let mut writer = MockPackWriter::new();
        writer
            .expect_write()
            .times(1)
            .withf(|pack, sources| {
                pack.name.as_str() == "my-pack"
                    && pack.clis == vec![CliId::new("gemini").unwrap(), CliId::new("vibe").unwrap()]
                    && pack.items.len() == 1
                    && pack.items[0].kind == PackItemKind::Skill
                    && pack.items[0].source == Path::new("skills/tdd")
                    && sources.len() == 1
                    && sources[0].host_path == Path::new("./my-skills/tdd")
                    && sources[0].pack_source == Path::new("skills/tdd")
            })
            .returning(|_, _| Ok(ok_outcome()));

        let service = service_with_registry(false, test_registry(&["vibe", "gemini"]), writer);
        let result = service.create(&sample_dto()).unwrap();
        assert_eq!(result.pack_name, "my-pack");
        assert_eq!(result.copied_count, 1);
        assert_eq!(result.pack_dir, PathBuf::from("/lib/my-pack"));
    }

    #[test]
    fn create_rejects_existing_pack_without_overwrite() {
        let mut writer = MockPackWriter::new();
        writer.expect_write().never();
        let service = service_with_registry(true, test_registry(&["vibe", "gemini"]), writer);
        let err = service.create(&sample_dto()).unwrap_err();
        assert!(matches!(err, ApplicationError::PackAlreadyExists(_)));
    }

    #[test]
    fn create_with_overwrite_writes_anyway() {
        let mut writer = MockPackWriter::new();
        writer
            .expect_write()
            .times(1)
            .returning(|_, _| Ok(ok_outcome()));
        let mut dto = sample_dto();
        dto.overwrite = true;
        let service = service_with_registry(true, test_registry(&["vibe", "gemini"]), writer);
        assert!(service.create(&dto).is_ok());
    }

    #[test]
    fn create_rejects_item_missing_a_destination_for_a_declared_cli() {
        let mut writer = MockPackWriter::new();
        writer.expect_write().never();
        let mut dto = dto(vec![item_dto(
            "skill",
            "./tdd",
            &[("vibe", ".agents/skills/tdd/")],
        )]);
        dto.clis = vec!["vibe".to_string(), "gemini".to_string()];
        let service = service_with_registry(false, test_registry(&["vibe", "gemini"]), writer);
        let err = service.create(&dto).unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::EmptyItemDestination)
        ));
    }

    #[test]
    fn create_rejects_duplicate_canonical_sources() {
        let mut writer = MockPackWriter::new();
        writer.expect_write().never();
        // Same file name, same kind -> same canonical source `skills/tdd`.
        let mut dto = dto(vec![
            item_dto("skill", "./a/tdd", &[("vibe", ".agents/skills/tdd/")]),
            item_dto("skill", "./b/tdd", &[("vibe", ".agents/skills/tdd/")]),
        ]);
        dto.clis = vec!["vibe".to_string()];
        let service = service_with_registry(false, test_registry(&["vibe", "gemini"]), writer);
        let err = service.create(&dto).unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::DuplicateItemSource(_))
        ));
    }

    #[test]
    fn create_allows_same_name_across_kinds() {
        let mut writer = MockPackWriter::new();
        writer
            .expect_write()
            .times(1)
            .withf(|_pack, sources| {
                sources[0].pack_source == Path::new("skills/tdd")
                    && sources[1].pack_source == Path::new("commands/tdd")
            })
            .returning(|_, _| Ok(ok_outcome()));
        let dto = dto(vec![
            item_dto("skill", "./tdd", &[("vibe", ".agents/skills/tdd/")]),
            item_dto("command", "./tdd", &[("vibe", ".vibe/commands/tdd.toml")]),
        ]);
        let service = service_with_registry(false, test_registry(&["vibe", "gemini"]), writer);
        assert!(service.create(&dto).is_ok());
    }

    #[test]
    fn create_rejects_invalid_name_version_kind_and_cli() {
        let mut writer = MockPackWriter::new();
        writer.expect_write().never();
        let service = service_with_registry(false, test_registry(&["vibe", "gemini"]), writer);

        let mut bad_name = sample_dto();
        bad_name.name = "Invalid Name".to_string();
        assert!(service.create(&bad_name).is_err());

        let mut bad_version = sample_dto();
        bad_version.version = "1.0".to_string();
        assert!(service.create(&bad_version).is_err());

        let mut bad_kind = sample_dto();
        bad_kind.items[0].kind = "macro".to_string();
        assert!(service.create(&bad_kind).is_err());

        let mut bad_cli = sample_dto();
        bad_cli.clis = vec!["copilot".to_string()];
        let err = service.create(&bad_cli).unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::UnknownCliProfile(_))
        ));
    }

    #[test]
    fn create_accepts_a_cli_known_only_through_a_profile() {
        let mut writer = MockPackWriter::new();
        writer
            .expect_write()
            .times(1)
            .withf(|pack, _| pack.clis == vec![CliId::new("claude").unwrap()])
            .returning(|_, _| Ok(ok_outcome()));
        let mut dto = sample_dto();
        dto.clis = vec!["claude".to_string()];
        dto.items[0].destinations = vec![(
            "claude".to_string(),
            ".claude/skills/tdd/SKILL.md".to_string(),
        )];
        let service = service_with_registry(false, test_registry(&["claude"]), writer);
        assert!(service.create(&dto).is_ok());
    }

    #[test]
    fn create_rejects_empty_destination_and_source_without_file_name() {
        let mut writer = MockPackWriter::new();
        writer.expect_write().never();
        let service = service_with_registry(false, test_registry(&["vibe", "gemini"]), writer);

        let mut empty_dest = dto(vec![item_dto("skill", "./tdd", &[("vibe", "  ")])]);
        empty_dest.clis = vec!["vibe".to_string()];
        assert!(service.create(&empty_dest).is_err());

        let mut no_name = dto(vec![item_dto("skill", "/", &[("vibe", ".agents/x/")])]);
        no_name.clis = vec!["vibe".to_string()];
        assert!(service.create(&no_name).is_err());
    }

    #[test]
    fn create_rejects_uninstallable_destinations() {
        let mut writer = MockPackWriter::new();
        writer.expect_write().never();
        let service = service_with_registry(false, test_registry(&["vibe", "gemini"]), writer);

        for bad_dest in ["/etc/passwd", "../../outside", ".", "./"] {
            let mut dto = dto(vec![item_dto("skill", "./tdd", &[("vibe", bad_dest)])]);
            dto.clis = vec!["vibe".to_string()];
            let err = service.create(&dto).unwrap_err();
            assert!(
                matches!(
                    err,
                    ApplicationError::Domain(
                        DomainError::ItemDestinationOutsideTarget(_)
                            | DomainError::EmptyItemDestination
                    )
                ),
                "destination {bad_dest:?} must be rejected at creation"
            );
        }
    }
}
