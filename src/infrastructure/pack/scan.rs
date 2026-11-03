use std::path::{Path, PathBuf};

use crate::application::error::ApplicationError;
use crate::application::pack_admin::dto::ScannedItem;
use crate::application::pack_admin::traits::BaseDirScanner;
use crate::domain::pack::value_objects::PackItemKind;

/// Scans a base directory for pack content (E13): each entry of its
/// `agents/`, `skills/`, `rules/`, `commands/` subdirectories becomes one
/// pack item (file or directory, directories copied recursively later).
/// Anything else at the base root is ignored. Entries are sorted by
/// (kind, name) so the generated manifest is deterministic.
pub struct FsBaseDirScanner;

impl BaseDirScanner for FsBaseDirScanner {
    fn scan(&self, base: &Path) -> Result<Vec<ScannedItem>, ApplicationError> {
        if !base.is_dir() {
            return Err(ApplicationError::NotADirectory(
                base.to_string_lossy().into_owned(),
            ));
        }
        let mut items = Vec::new();
        for kind in PackItemKind::all() {
            let kind_dir = base.join(kind.plural_dir());
            let Ok(entries) = std::fs::read_dir(&kind_dir) else {
                continue;
            };
            let mut entries: Vec<_> = entries.flatten().collect();
            entries.sort_by_key(|entry| entry.file_name());
            for entry in entries {
                let host_path = entry.path();
                items.push(ScannedItem {
                    kind: *kind,
                    source_is_dir: host_path.is_dir(),
                    pack_source: PathBuf::from(kind.plural_dir()).join(entry.file_name()),
                    host_path,
                });
            }
        }
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// A base directory with one entry per kind plus an unrelated dir.
    fn base_tree(tmp: &TempDir) -> PathBuf {
        let base = tmp.path().join("my-pack");
        fs::create_dir_all(base.join("skills/tdd")).unwrap();
        fs::write(base.join("skills/tdd/SKILL.md"), "# TDD").unwrap();
        fs::create_dir_all(base.join("agents")).unwrap();
        fs::write(base.join("agents/debugger.toml"), "agent").unwrap();
        fs::create_dir_all(base.join("rules")).unwrap();
        fs::write(base.join("rules/no-panic.md"), "rule").unwrap();
        fs::create_dir_all(base.join("commands")).unwrap();
        fs::write(base.join("commands/review.toml"), "cmd").unwrap();
        fs::create_dir_all(base.join("docs")).unwrap();
        fs::write(base.join("docs/ignored.md"), "not an item").unwrap();
        base
    }

    #[test]
    fn scan_collects_one_item_per_entry_of_the_kind_directories() {
        let tmp = TempDir::new().unwrap();
        let base = base_tree(&tmp);
        let items = FsBaseDirScanner.scan(&base).unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].kind, PackItemKind::Skill);
        assert_eq!(items[0].pack_source, PathBuf::from("skills/tdd"));
        assert!(items[0].source_is_dir);
        assert_eq!(items[0].host_path, base.join("skills/tdd"));
        assert_eq!(items[1].kind, PackItemKind::Agent);
        assert_eq!(items[1].pack_source, PathBuf::from("agents/debugger.toml"));
        assert!(!items[1].source_is_dir);
        assert_eq!(items[2].kind, PackItemKind::Rule);
        assert_eq!(items[2].pack_source, PathBuf::from("rules/no-panic.md"));
        assert_eq!(items[3].kind, PackItemKind::Command);
        assert_eq!(items[3].pack_source, PathBuf::from("commands/review.toml"));
    }

    #[test]
    fn scan_ignores_directories_that_are_not_kind_directories() {
        let tmp = TempDir::new().unwrap();
        let base = base_tree(&tmp);
        let items = FsBaseDirScanner.scan(&base).unwrap();
        assert!(!items
            .iter()
            .any(|item| item.pack_source.starts_with("docs")));
    }

    #[test]
    fn scan_of_an_empty_base_is_empty() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join("empty");
        fs::create_dir_all(&base).unwrap();
        assert!(FsBaseDirScanner.scan(&base).unwrap().is_empty());
    }

    #[test]
    fn scan_of_a_missing_base_is_not_a_directory() {
        let tmp = TempDir::new().unwrap();
        let err = FsBaseDirScanner
            .scan(&tmp.path().join("missing"))
            .unwrap_err();
        assert!(matches!(err, ApplicationError::NotADirectory(_)));
    }
}
