use std::collections::BTreeMap;
use std::path::Path;

use crate::domain::cli_profile::value_objects::CliId;
use crate::domain::pack::entity::PackItem;
use crate::domain::pack::error::{validate_item_destination, DomainError};
use crate::domain::pack::value_objects::PackItemKind;

/// Destination templates of one target CLI, one per item kind. The base
/// template applies to file sources; the optional `_dir` template applies to
/// directory sources (a copied tree needs a directory destination, not a
/// file path). Placeholders: `{name}` = item name (see `item_name`),
/// `{file}` = source file name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestinationTemplates {
    pub skill: String,
    pub skill_dir: Option<String>,
    pub agent: String,
    pub agent_dir: Option<String>,
    pub command: String,
    pub command_dir: Option<String>,
}

impl DestinationTemplates {
    fn for_kind(&self, kind: PackItemKind, source_is_dir: bool) -> Result<&str, DomainError> {
        let (base, dir) = match kind {
            PackItemKind::Skill => (&self.skill, &self.skill_dir),
            PackItemKind::Agent => (&self.agent, &self.agent_dir),
            PackItemKind::Command => (&self.command, &self.command_dir),
        };
        if source_is_dir {
            dir.as_deref().ok_or_else(|| {
                DomainError::MissingProfileTemplate(format!("{}_dir", kind.as_str()))
            })
        } else {
            Ok(base)
        }
    }
}

/// One loadable CLI profile: how items of a pack map to destinations inside
/// a target repository for this CLI (E02). Loaded from `clis/<id>.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliProfile {
    pub id: CliId,
    pub label: String,
    pub templates: DestinationTemplates,
}

impl CliProfile {
    pub fn new(
        id: CliId,
        label: String,
        templates: DestinationTemplates,
    ) -> Result<CliProfile, DomainError> {
        for template in templates.all() {
            validate_template(template)?;
        }
        Ok(CliProfile {
            id,
            label,
            templates,
        })
    }

    /// Compute the destination of one pack item for this CLI. Explicit
    /// manifest destinations are handled by the caller (application layer);
    /// this is the profile-based default. The result is validated with the
    /// same rule as explicit destinations (`validate_item_destination`).
    pub fn destination(&self, item: &PackItem, source_is_dir: bool) -> Result<String, DomainError> {
        let template = self.templates.for_kind(item.kind, source_is_dir)?;
        let name = item_name(item.kind, &item.source, source_is_dir);
        let file = file_name(&item.source).unwrap_or_else(|| name.clone());
        let rendered = template.replace("{name}", &name).replace("{file}", &file);
        validate_item_destination(&rendered)
            .map_err(|_| DomainError::InvalidProfileTemplate(rendered.clone()))?;
        Ok(rendered)
    }
}

impl DestinationTemplates {
    fn all(&self) -> impl Iterator<Item = &str> {
        [&self.skill, &self.agent, &self.command]
            .into_iter()
            .map(|s| s.as_str())
            .chain(
                [&self.skill_dir, &self.agent_dir, &self.command_dir]
                    .into_iter()
                    .flatten()
                    .map(|s| s.as_str()),
            )
    }
}

/// Validate a raw template by rendering it with a sample item: the rendered
/// destination must pass the same rule as an explicit destination.
fn validate_template(template: &str) -> Result<(), DomainError> {
    let rendered = template.replace("{name}", "x").replace("{file}", "x.md");
    validate_item_destination(&rendered)
        .map_err(|_| DomainError::InvalidProfileTemplate(template.to_string()))
}

/// Item name used for the `{name}` placeholder: a directory source keeps its
/// own name; a file source uses its parent directory name, unless that
/// parent is the canonical pack kind directory (`skills/`, `agents/`,
/// `commands/`) — in that case the file stem is used (e.g.
/// `commands/tdd.toml` -> "tdd", `skills/tdd/SKILL.md` -> "tdd").
pub fn item_name(kind: PackItemKind, source: &Path, source_is_dir: bool) -> String {
    if source_is_dir {
        if let Some(name) = source.file_name() {
            return name.to_string_lossy().into_owned();
        }
    } else if let Some(parent) = source.parent() {
        if let Some(parent_name) = parent.file_name() {
            let parent_name = parent_name.to_string_lossy();
            if parent_name != kind.plural_dir() {
                return parent_name.into_owned();
            }
        }
    }
    source
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| source.to_string_lossy().into_owned())
}

/// Source file name (for the `{file}` placeholder).
pub fn file_name(source: &Path) -> Option<String> {
    source.file_name().map(|s| s.to_string_lossy().into_owned())
}

/// The loaded CLI profiles, keyed by id. Decides whether a `CliId` is usable
/// (replaces the v1 closed `CliId` enum).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CliProfileRegistry {
    profiles: BTreeMap<CliId, CliProfile>,
}

impl CliProfileRegistry {
    pub fn new(profiles: Vec<CliProfile>) -> CliProfileRegistry {
        CliProfileRegistry {
            profiles: profiles.into_iter().map(|p| (p.id.clone(), p)).collect(),
        }
    }

    pub fn get(&self, id: &CliId) -> Option<&CliProfile> {
        self.profiles.get(id)
    }

    /// Validate a raw CLI id against the registry: well-formed and profiled.
    pub fn validate(&self, raw: &str) -> Result<CliId, DomainError> {
        let id = CliId::new(raw)?;
        if self.profiles.contains_key(&id) {
            Ok(id)
        } else {
            Err(DomainError::UnknownCliProfile(id.to_string()))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn ids(&self) -> impl Iterator<Item = &CliId> {
        self.profiles.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&CliId, &CliProfile)> {
        self.profiles.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn templates() -> DestinationTemplates {
        DestinationTemplates {
            skill: ".agents/skills/{name}/SKILL.md".to_string(),
            skill_dir: Some(".agents/skills/{name}/".to_string()),
            agent: ".vibe/agents/{name}.toml".to_string(),
            agent_dir: None,
            command: ".vibe/commands/{name}.toml".to_string(),
            command_dir: None,
        }
    }

    fn profile() -> CliProfile {
        CliProfile::new(
            CliId::new("vibe").unwrap(),
            "Vibe CLI".to_string(),
            templates(),
        )
        .unwrap()
    }

    fn item(source: &str) -> PackItem {
        PackItem::new(
            PackItemKind::Skill,
            PathBuf::from(source),
            Default::default(),
        )
        .unwrap()
    }

    #[test]
    fn destination_renders_file_template_with_item_name() {
        let p = profile();
        let dest = p.destination(&item("skills/tdd/SKILL.md"), false).unwrap();
        assert_eq!(dest, ".agents/skills/tdd/SKILL.md");
    }

    #[test]
    fn destination_uses_parent_dir_name_not_kind_dir() {
        let p = profile();
        // commands/tdd.toml -> stem "tdd" (parent is the canonical commands dir)
        let mut cmd = item("commands/tdd.toml");
        cmd.kind = PackItemKind::Command;
        let dest = p.destination(&cmd, false).unwrap();
        assert_eq!(dest, ".vibe/commands/tdd.toml");
    }

    #[test]
    fn destination_renders_dir_template_for_directory_sources() {
        let p = profile();
        let dest = p.destination(&item("skills/tdd"), true).unwrap();
        assert_eq!(dest, ".agents/skills/tdd/");
    }

    #[test]
    fn destination_rejects_dir_source_without_dir_template() {
        let p = profile();
        let mut agent = item("agents/debugger");
        agent.kind = PackItemKind::Agent;
        let err = p.destination(&agent, true).unwrap_err();
        assert!(matches!(err, DomainError::MissingProfileTemplate(_)));
    }

    #[test]
    fn profile_rejects_template_rendering_outside_target() {
        let bad = DestinationTemplates {
            skill: "../escape/{name}".to_string(),
            ..templates()
        };
        let err =
            CliProfile::new(CliId::new("evil").unwrap(), "Evil".to_string(), bad).unwrap_err();
        assert!(matches!(err, DomainError::InvalidProfileTemplate(_)));
    }

    #[test]
    fn item_name_rules() {
        assert_eq!(
            item_name(PackItemKind::Skill, Path::new("skills/tdd"), true),
            "tdd"
        );
        assert_eq!(
            item_name(PackItemKind::Skill, Path::new("skills/tdd/SKILL.md"), false),
            "tdd"
        );
        assert_eq!(
            item_name(PackItemKind::Command, Path::new("commands/tdd.toml"), false),
            "tdd"
        );
        assert_eq!(
            item_name(PackItemKind::Agent, Path::new("agents/x.toml"), false),
            "x"
        );
    }

    #[test]
    fn registry_validates_against_loaded_profiles() {
        let registry = CliProfileRegistry::new(vec![profile()]);
        assert_eq!(registry.validate("vibe").unwrap().as_str(), "vibe");
        assert!(matches!(
            registry.validate("claude").unwrap_err(),
            DomainError::UnknownCliProfile(_)
        ));
        assert!(matches!(
            registry.validate("Not An Id").unwrap_err(),
            DomainError::InvalidCliId(_)
        ));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        assert_eq!(registry.ids().next().unwrap().as_str(), "vibe");
    }

    #[test]
    fn empty_registry_rejects_everything() {
        let registry = CliProfileRegistry::default();
        assert!(registry.is_empty());
        assert!(matches!(
            registry.validate("vibe").unwrap_err(),
            DomainError::UnknownCliProfile(_)
        ));
    }
}
