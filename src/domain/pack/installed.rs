use std::path::{Component, Path, PathBuf};

use crate::domain::pack::error::DomainError;
use crate::domain::pack::value_objects::{PackName, PackVersion, TargetCli};

/// Name of the installation index file written at the root of a target
/// repository. Reserved: no pack item may install to this path.
pub const INDEX_FILENAME: &str = "packs_index.toml";

/// One recorded installation of a pack in a target repository: identity,
/// target CLI and the paths (relative to the target root) actually copied
/// during the install. The paths are the source of truth for uninstalling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPack {
    pub name: PackName,
    pub version: PackVersion,
    pub cli: TargetCli,
    pub paths: Vec<PathBuf>,
}

impl InstalledPack {
    pub fn new(
        name: PackName,
        version: PackVersion,
        cli: TargetCli,
        paths: Vec<PathBuf>,
    ) -> Result<InstalledPack, DomainError> {
        if paths.is_empty() {
            return Err(DomainError::EmptyInstallPaths);
        }
        for path in &paths {
            if !is_safe_relative(path) {
                return Err(DomainError::InstallPathOutsideTarget(
                    path.to_string_lossy().into_owned(),
                ));
            }
        }
        Ok(InstalledPack {
            name,
            version,
            cli,
            paths,
        })
    }

    /// Index key: one entry per (pack name, target CLI).
    pub fn matches(&self, name: &str, cli: TargetCli) -> bool {
        self.name.as_str() == name && self.cli == cli
    }
}

/// A recorded path must be relative, non-empty and stay inside the target
/// root: no absolute path, no `..`, no bare `.`.
fn is_safe_relative(path: &Path) -> bool {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return false;
    }
    let mut has_normal = false;
    for component in path.components() {
        match component {
            Component::Normal(_) => has_normal = true,
            Component::CurDir => {}
            _ => return false,
        }
    }
    has_normal
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(paths: Vec<PathBuf>) -> Result<InstalledPack, DomainError> {
        InstalledPack::new(
            PackName::new("tdd").unwrap(),
            PackVersion::new("1.0.0").unwrap(),
            TargetCli::Vibe,
            paths,
        )
    }

    #[test]
    fn new_accepts_relative_paths() {
        let installed = entry(vec![
            PathBuf::from(".agents/skills/tdd"),
            PathBuf::from(".vibe/agents/tdd-orchestrator.toml"),
        ])
        .unwrap();
        assert_eq!(installed.paths.len(), 2);
        assert!(installed.matches("tdd", TargetCli::Vibe));
        assert!(!installed.matches("tdd", TargetCli::Gemini));
        assert!(!installed.matches("review", TargetCli::Vibe));
    }

    #[test]
    fn new_rejects_empty_paths() {
        let err = entry(vec![]).expect_err("empty paths must be rejected");
        assert!(matches!(err, DomainError::EmptyInstallPaths));
    }

    #[test]
    fn new_rejects_absolute_path() {
        let err =
            entry(vec![PathBuf::from("/etc/passwd")]).expect_err("absolute path must be rejected");
        assert!(matches!(err, DomainError::InstallPathOutsideTarget(_)));
    }

    #[test]
    fn new_rejects_escaping_path() {
        let err =
            entry(vec![PathBuf::from("../outside")]).expect_err("escaping path must be rejected");
        assert!(matches!(err, DomainError::InstallPathOutsideTarget(_)));
    }

    #[test]
    fn new_rejects_bare_dot_and_empty_path() {
        assert!(entry(vec![PathBuf::from(".")]).is_err());
        assert!(entry(vec![PathBuf::new()]).is_err());
    }
}
