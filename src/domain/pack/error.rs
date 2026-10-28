use std::fmt;
use std::path::PathBuf;

use crate::domain::pack::entity::{Pack, PackItem};
use crate::domain::pack::installed::INDEX_FILENAME;
use crate::domain::pack::value_objects::TargetCli;

#[derive(Debug, PartialEq, Eq)]
pub enum DomainError {
    EmptyName,
    InvalidName(String),
    InvalidVersion(String),
    EmptyCliList,
    UnknownCli(String),
    UnknownItemKind(String),
    EmptyItemSource,
    EmptyItemDestination,
    ItemDestinationOutsideTarget(String),
    InstallCollision(String),
    ReservedDestination(String),
    EmptyInstallPaths,
    InstallPathOutsideTarget(String),
    DuplicateItemSource(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::EmptyName => write!(f, "pack name must not be empty"),
            DomainError::InvalidName(n) => write!(f, "invalid pack name: {n}"),
            DomainError::InvalidVersion(v) => write!(f, "invalid pack version: {v}"),
            DomainError::EmptyCliList => write!(f, "pack must target at least one CLI"),
            DomainError::UnknownCli(c) => write!(f, "unknown target CLI: {c}"),
            DomainError::UnknownItemKind(k) => write!(f, "unknown pack item kind: {k}"),
            DomainError::EmptyItemSource => write!(f, "pack item source must not be empty"),
            DomainError::EmptyItemDestination => {
                write!(f, "pack item destination must not be empty")
            }
            DomainError::ItemDestinationOutsideTarget(d) => {
                write!(f, "pack item destination escapes target root: {d}")
            }
            DomainError::InstallCollision(d) => {
                write!(f, "installation collision on destination: {d}")
            }
            DomainError::ReservedDestination(d) => {
                write!(f, "reserved destination ({INDEX_FILENAME}): {d}")
            }
            DomainError::EmptyInstallPaths => {
                write!(f, "an installed pack must record at least one path")
            }
            DomainError::InstallPathOutsideTarget(p) => {
                write!(f, "installed path escapes target root: {p}")
            }
            DomainError::DuplicateItemSource(s) => {
                write!(f, "duplicate pack item source: {s}")
            }
        }
    }
}

impl std::error::Error for DomainError {}

/// A single planned copy: source file inside the pack, destination inside the target repo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedCopy {
    pub source: PathBuf,
    pub destination: PathBuf,
}

/// Compute the installation plan of a pack for one target CLI inside `target_root`.
///
/// Domain rule: each item destination is resolved against the target root and
/// must stay inside it; a destination already claimed by another item is a
/// collision and must be rejected before any filesystem write.
pub fn plan_install(
    pack: &Pack,
    cli: TargetCli,
    target_root: &std::path::Path,
) -> Result<Vec<PlannedCopy>, DomainError> {
    let mut planned = Vec::new();
    let mut claimed: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    for item in &pack.items {
        let destination = resolve_destination(item, cli, target_root)?;
        if !claimed.insert(destination.clone()) {
            return Err(DomainError::InstallCollision(
                destination.to_string_lossy().into_owned(),
            ));
        }
        planned.push(PlannedCopy {
            source: item.source.clone(),
            destination,
        });
    }

    Ok(planned)
}

fn resolve_destination(
    item: &PackItem,
    cli: TargetCli,
    target_root: &std::path::Path,
) -> Result<PathBuf, DomainError> {
    if item.source.as_os_str().is_empty() {
        return Err(DomainError::EmptyItemSource);
    }

    let destination = item
        .destination_for(cli)
        .ok_or(DomainError::EmptyItemDestination)?;

    let resolved = normalize_in_target(&destination, target_root)?;
    // The install index at the target root is managed by the application,
    // never by a pack item.
    if resolved.file_name().is_some_and(|f| f == INDEX_FILENAME)
        && resolved.parent().is_some_and(|p| p == target_root)
    {
        return Err(DomainError::ReservedDestination(destination.to_string()));
    }
    Ok(resolved)
}

/// Join `target_root` with a relative destination, rejecting absolute or
/// upward-escaping paths (`..`).
fn normalize_in_target(
    destination: &str,
    target_root: &std::path::Path,
) -> Result<PathBuf, DomainError> {
    let rel = std::path::Path::new(destination);
    if rel.is_absolute() {
        return Err(DomainError::ItemDestinationOutsideTarget(
            destination.to_string(),
        ));
    }
    let mut resolved = target_root.to_path_buf();
    for component in rel.components() {
        match component {
            std::path::Component::Normal(part) => {
                resolved.push(part);
            }
            std::path::Component::CurDir => {}
            _ => {
                return Err(DomainError::ItemDestinationOutsideTarget(
                    destination.to_string(),
                ));
            }
        }
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::pack::value_objects::{PackName, PackVersion, TargetCliMap};

    fn sample_pack() -> Pack {
        Pack::new(
            PackName::new("tdd").unwrap(),
            PackVersion::new("1.0.0").unwrap(),
            "TDD pack".to_string(),
            vec![TargetCli::Vibe],
            vec![PackItem::new(
                crate::domain::pack::value_objects::PackItemKind::Skill,
                PathBuf::from("skills/tdd/SKILL.md"),
                TargetCliMap::from_pairs(&[(TargetCli::Vibe, ".agents/skills/tdd/")]),
            )
            .unwrap()],
        )
        .unwrap()
    }

    #[test]
    fn plan_install_resolves_destination_in_target() {
        let pack = sample_pack();
        let plan =
            plan_install(&pack, TargetCli::Vibe, std::path::Path::new("/repo")).expect("plan");
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].source, PathBuf::from("skills/tdd/SKILL.md"));
        assert_eq!(
            plan[0].destination,
            PathBuf::from("/repo/.agents/skills/tdd")
        );
    }

    #[test]
    fn plan_install_rejects_absolute_destination() {
        let mut pack = sample_pack();
        pack.items[0] = PackItem::new(
            crate::domain::pack::value_objects::PackItemKind::Skill,
            PathBuf::from("skills/tdd/SKILL.md"),
            TargetCliMap::from_pairs(&[(TargetCli::Vibe, "/etc/passwd")]),
        )
        .unwrap();
        let err = plan_install(&pack, TargetCli::Vibe, std::path::Path::new("/repo"))
            .expect_err("must reject absolute destination");
        assert!(matches!(err, DomainError::ItemDestinationOutsideTarget(_)));
    }

    #[test]
    fn plan_install_rejects_escaping_destination() {
        let mut pack = sample_pack();
        pack.items[0] = PackItem::new(
            crate::domain::pack::value_objects::PackItemKind::Skill,
            PathBuf::from("skills/tdd/SKILL.md"),
            TargetCliMap::from_pairs(&[(TargetCli::Vibe, "../outside/")]),
        )
        .unwrap();
        let err = plan_install(&pack, TargetCli::Vibe, std::path::Path::new("/repo"))
            .expect_err("must reject escaping destination");
        assert!(matches!(err, DomainError::ItemDestinationOutsideTarget(_)));
    }

    #[test]
    fn plan_install_rejects_collision() {
        let item = || {
            PackItem::new(
                crate::domain::pack::value_objects::PackItemKind::Skill,
                PathBuf::from("skills/tdd/SKILL.md"),
                TargetCliMap::from_pairs(&[(TargetCli::Vibe, ".agents/skills/tdd/SKILL.md")]),
            )
            .unwrap()
        };
        let mut pack = sample_pack();
        pack.items.push(item());
        pack.items.push(item());
        let err = plan_install(&pack, TargetCli::Vibe, std::path::Path::new("/repo"))
            .expect_err("must reject collision");
        assert!(matches!(err, DomainError::InstallCollision(_)));
    }

    #[test]
    fn plan_install_empty_destination_for_cli_is_rejected() {
        let pack = sample_pack();
        let err = plan_install(&pack, TargetCli::Gemini, std::path::Path::new("/repo"))
            .expect_err("must reject missing destination");
        assert!(matches!(err, DomainError::EmptyItemDestination));
    }

    #[test]
    fn normalize_accepts_dot_components() {
        let resolved = normalize_in_target("./a/./b", std::path::Path::new("/repo")).unwrap();
        assert_eq!(resolved, PathBuf::from("/repo/a/b"));
    }

    #[test]
    fn plan_install_rejects_index_file_destination() {
        let mut pack = sample_pack();
        pack.items[0] = PackItem::new(
            crate::domain::pack::value_objects::PackItemKind::Skill,
            PathBuf::from("skills/tdd/SKILL.md"),
            TargetCliMap::from_pairs(&[(TargetCli::Vibe, "packs_index.toml")]),
        )
        .unwrap();
        let err = plan_install(&pack, TargetCli::Vibe, std::path::Path::new("/repo"))
            .expect_err("must reject the index file destination");
        assert!(matches!(err, DomainError::ReservedDestination(_)));
    }

    #[test]
    fn plan_install_allows_index_file_name_below_root() {
        let mut pack = sample_pack();
        pack.items[0] = PackItem::new(
            crate::domain::pack::value_objects::PackItemKind::Skill,
            PathBuf::from("skills/tdd/SKILL.md"),
            TargetCliMap::from_pairs(&[(TargetCli::Vibe, ".agents/packs_index.toml")]),
        )
        .unwrap();
        let plan = plan_install(&pack, TargetCli::Vibe, std::path::Path::new("/repo"))
            .expect("only the root index file is reserved");
        assert_eq!(plan.len(), 1);
    }
}
