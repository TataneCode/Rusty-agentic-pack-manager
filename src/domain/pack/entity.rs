use std::path::PathBuf;

use crate::domain::pack::error::DomainError;
use crate::domain::pack::value_objects::{
    PackItemKind, PackName, PackVersion, TargetCli, TargetCliMap,
};

/// One agentic item of a pack: a source path inside the pack directory and
/// its per-CLI destinations (relative to the user-chosen target root).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackItem {
    pub kind: PackItemKind,
    /// Path inside the pack directory (never absolute).
    pub source: PathBuf,
    pub destinations: TargetCliMap,
}

impl PackItem {
    pub fn new(
        kind: PackItemKind,
        source: PathBuf,
        destinations: TargetCliMap,
    ) -> Result<PackItem, DomainError> {
        if source.as_os_str().is_empty() {
            return Err(DomainError::EmptyItemSource);
        }
        Ok(PackItem {
            kind,
            source,
            destinations,
        })
    }

    pub fn destination_for(&self, cli: TargetCli) -> Option<String> {
        self.destinations.get(cli).map(|s| s.to_string())
    }
}

/// A pack: coherent set of skills/agents/commands installable for one or
/// several target CLIs. Identity = name + version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pack {
    pub name: PackName,
    pub version: PackVersion,
    pub description: String,
    pub clis: Vec<TargetCli>,
    pub items: Vec<PackItem>,
}

impl Pack {
    pub fn new(
        name: PackName,
        version: PackVersion,
        description: String,
        clis: Vec<TargetCli>,
        items: Vec<PackItem>,
    ) -> Result<Pack, DomainError> {
        if clis.is_empty() {
            return Err(DomainError::EmptyCliList);
        }
        Ok(Pack {
            name,
            version,
            description,
            clis,
            items,
        })
    }

    pub fn supports(&self, cli: TargetCli) -> bool {
        self.clis.contains(&cli)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(kind: PackItemKind, source: &str, dest: &str) -> PackItem {
        PackItem::new(
            kind,
            PathBuf::from(source),
            TargetCliMap::from_pairs(&[(TargetCli::Vibe, dest)]),
        )
        .unwrap()
    }

    #[test]
    fn pack_requires_at_least_one_cli() {
        let err = Pack::new(
            PackName::new("tdd").unwrap(),
            PackVersion::new("1.0.0").unwrap(),
            String::new(),
            vec![],
            vec![],
        )
        .expect_err("empty cli list must be rejected");
        assert!(matches!(err, DomainError::EmptyCliList));
    }

    #[test]
    fn pack_supports_and_item_destination() {
        let pack = Pack::new(
            PackName::new("tdd").unwrap(),
            PackVersion::new("1.0.0").unwrap(),
            "desc".to_string(),
            vec![TargetCli::Vibe],
            vec![item(
                PackItemKind::Skill,
                "skills/tdd",
                ".agents/skills/tdd/",
            )],
        )
        .unwrap();
        assert!(pack.supports(TargetCli::Vibe));
        assert!(!pack.supports(TargetCli::Gemini));
        assert_eq!(
            pack.items[0].destination_for(TargetCli::Vibe).unwrap(),
            ".agents/skills/tdd/"
        );
        assert_eq!(pack.items[0].destination_for(TargetCli::Gemini), None);
    }

    #[test]
    fn pack_item_rejects_empty_source() {
        let err = PackItem::new(PackItemKind::Skill, PathBuf::new(), TargetCliMap::default())
            .expect_err("empty source must be rejected");
        assert!(matches!(err, DomainError::EmptyItemSource));
    }
}
