use crate::application::cli_profile::traits::CliProfileRepository;
use crate::application::error::ApplicationError;
use crate::application::pack_view::traits::PackContentRepository;
use crate::domain::cli_profile::value_objects::CliId;
use crate::domain::pack::entity::Pack;

/// Resolves pack items against the CLI profiles: an item without an
/// explicit destination for a CLI gets the profile's default destination
/// (E02). Explicit manifest destinations always win (retrocompatible with
/// the existing packs).
pub struct CliProfileService {
    profiles: Box<dyn CliProfileRepository>,
    content: Box<dyn PackContentRepository>,
}

impl CliProfileService {
    pub fn new(
        profiles: Box<dyn CliProfileRepository>,
        content: Box<dyn PackContentRepository>,
    ) -> CliProfileService {
        CliProfileService { profiles, content }
    }

    /// Return a copy of `pack` where every item has a destination for `cli`:
    /// explicit destinations are kept, missing ones are computed from the
    /// CLI profile. Fails with `UnknownCliProfile` when no profile exists
    /// for `cli`, or `MissingProfileTemplate` when the profile has no
    /// template for an item (e.g. a directory source without a `_dir`
    /// template).
    pub fn resolve_pack(&self, pack: &Pack, cli: &CliId) -> Result<Pack, ApplicationError> {
        let profile = self.profiles.find(cli)?;
        let pack_name = pack.name.as_str();
        let mut resolved = pack.clone();
        for item in &mut resolved.items {
            if item.destinations.get(cli).is_some() {
                continue;
            }
            let source_is_dir = self.content.source_is_dir(pack_name, &item.source)?;
            let destination = profile.destination(item, source_is_dir)?;
            item.destinations.insert(cli.clone(), destination);
        }
        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::cli_profile::traits::MockCliProfileRepository;
    use crate::application::pack_view::traits::MockPackContentRepository;
    use crate::domain::cli_profile::profile::{CliProfile, DestinationTemplates};
    use crate::domain::pack::entity::PackItem;
    use crate::domain::pack::error::DomainError;
    use crate::domain::pack::value_objects::{PackItemKind, PackName, PackVersion, TargetCliMap};
    use std::path::PathBuf;

    fn templates() -> DestinationTemplates {
        DestinationTemplates {
            skill: ".claude/skills/{name}/SKILL.md".to_string(),
            skill_dir: Some(".claude/skills/{name}/".to_string()),
            agent: ".claude/agents/{name}.md".to_string(),
            agent_dir: None,
            rule: None,
            rule_dir: None,
            command: ".claude/commands/{name}.md".to_string(),
            command_dir: None,
        }
    }

    fn claude_profile() -> CliProfile {
        CliProfile::new(
            CliId::new("claude").unwrap(),
            "Claude Code".to_string(),
            templates(),
        )
        .unwrap()
    }

    fn pack_with(items: Vec<PackItem>) -> Pack {
        Pack::new(
            PackName::new("demo").unwrap(),
            PackVersion::new("1.0.0").unwrap(),
            String::new(),
            vec![CliId::new("claude").unwrap()],
            items,
        )
        .unwrap()
    }

    fn file_item(source: &str, explicit: Option<(&str, &str)>) -> PackItem {
        let mut destinations = TargetCliMap::default();
        if let Some((cli, dest)) = explicit {
            destinations.insert(CliId::new(cli).unwrap(), dest.to_string());
        }
        PackItem::new(PackItemKind::Skill, PathBuf::from(source), destinations).unwrap()
    }

    #[test]
    fn resolve_computes_missing_destinations_from_the_profile() {
        let mut profiles = MockCliProfileRepository::new();
        profiles
            .expect_find()
            .withf(|id| id.as_str() == "claude")
            .returning(|_| Ok(claude_profile()));
        let mut content = MockPackContentRepository::new();
        content.expect_source_is_dir().returning(|_, _| Ok(false));

        let service = CliProfileService::new(Box::new(profiles), Box::new(content));
        let pack = pack_with(vec![file_item("skills/tdd/SKILL.md", None)]);
        let resolved = service
            .resolve_pack(&pack, &CliId::new("claude").unwrap())
            .unwrap();
        assert_eq!(
            resolved.items[0]
                .destination_for(&CliId::new("claude").unwrap())
                .unwrap(),
            ".claude/skills/tdd/SKILL.md"
        );
    }

    #[test]
    fn resolve_keeps_explicit_destinations_and_uses_dir_template_for_dir_sources() {
        let mut profiles = MockCliProfileRepository::new();
        profiles.expect_find().returning(|_| Ok(claude_profile()));
        let mut content = MockPackContentRepository::new();
        content
            .expect_source_is_dir()
            .withf(|_, source| source == std::path::Path::new("skills/tdd"))
            .returning(|_, _| Ok(true));

        let service = CliProfileService::new(Box::new(profiles), Box::new(content));
        let pack = pack_with(vec![
            file_item("skills/tdd", None),
            file_item(
                "skills/other/SKILL.md",
                Some(("claude", ".custom/other/SKILL.md")),
            ),
        ]);
        let claude = CliId::new("claude").unwrap();
        let resolved = service.resolve_pack(&pack, &claude).unwrap();
        assert_eq!(
            resolved.items[0].destination_for(&claude).unwrap(),
            ".claude/skills/tdd/"
        );
        assert_eq!(
            resolved.items[1].destination_for(&claude).unwrap(),
            ".custom/other/SKILL.md"
        );
    }

    #[test]
    fn resolve_fails_when_no_profile_exists_for_the_cli() {
        let mut profiles = MockCliProfileRepository::new();
        profiles.expect_find().returning(|id| {
            Err(ApplicationError::Domain(DomainError::UnknownCliProfile(
                id.as_str().to_string(),
            )))
        });
        let content = MockPackContentRepository::new();

        let service = CliProfileService::new(Box::new(profiles), Box::new(content));
        let pack = pack_with(vec![file_item("skills/tdd/SKILL.md", None)]);
        let err = service
            .resolve_pack(&pack, &CliId::new("unknown").unwrap())
            .unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::UnknownCliProfile(_))
        ));
    }

    #[test]
    fn resolve_fails_when_the_profile_lacks_a_dir_template() {
        let mut profiles = MockCliProfileRepository::new();
        profiles.expect_find().returning(|_| Ok(claude_profile()));
        let mut content = MockPackContentRepository::new();
        content.expect_source_is_dir().returning(|_, _| Ok(true));

        let service = CliProfileService::new(Box::new(profiles), Box::new(content));
        // agent source is a directory but the profile has no agent_dir template
        let mut item = file_item("agents/debugger", None);
        item.kind = PackItemKind::Agent;
        let pack = pack_with(vec![item]);
        let err = service
            .resolve_pack(&pack, &CliId::new("claude").unwrap())
            .unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::MissingProfileTemplate(_))
        ));
    }
}
