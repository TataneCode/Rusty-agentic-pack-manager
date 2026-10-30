use std::path::Path;

use crate::application::error::ApplicationError;
use crate::application::explorer::dto::{DirectoryListingDto, FileNodeDto};
use crate::application::explorer::traits::ExplorerRepository;
use crate::application::pack::traits::InstallDestination;

/// std::fs adapter for the host explorer panel and pack installation.
pub struct FsExplorer;

fn io_err(path: &str, source: std::io::Error) -> ApplicationError {
    ApplicationError::Io {
        path: path.to_string(),
        source,
    }
}

impl ExplorerRepository for FsExplorer {
    fn list(&self, path: &str) -> Result<DirectoryListingDto, ApplicationError> {
        let dir = Path::new(path);
        if !dir.is_dir() {
            return Err(ApplicationError::NotADirectory(path.to_string()));
        }
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(dir).map_err(|e| io_err(path, e))? {
            let entry = entry.map_err(|e| io_err(path, e))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            entries.push(FileNodeDto {
                name,
                is_dir: entry.path().is_dir(),
            });
        }
        Ok(DirectoryListingDto {
            path: dir.to_path_buf(),
            entries,
        })
    }

    fn create_dir(&self, parent: &str, name: &str) -> Result<(), ApplicationError> {
        let target = Path::new(parent).join(name);
        std::fs::create_dir(&target).map_err(|e| io_err(&target.to_string_lossy(), e))
    }

    fn rename(&self, parent: &str, from: &str, to: &str) -> Result<(), ApplicationError> {
        let source = Path::new(parent).join(from);
        let target = Path::new(parent).join(to);
        std::fs::rename(&source, &target).map_err(|e| {
            io_err(
                &format!(
                    "{} -> {}",
                    source.to_string_lossy(),
                    target.to_string_lossy()
                ),
                e,
            )
        })
    }

    fn remove(&self, parent: &str, name: &str) -> Result<(), ApplicationError> {
        let target = Path::new(parent).join(name);
        if target.is_dir() {
            std::fs::remove_dir_all(&target).map_err(|e| io_err(&target.to_string_lossy(), e))
        } else {
            std::fs::remove_file(&target).map_err(|e| io_err(&target.to_string_lossy(), e))
        }
    }
}

/// The pack source files live in the pack library; destinations are
/// absolute. We need both sides, so the installer adapter is separate from
/// the explorer one but shares the same std::fs operations. Roots are in
/// read order (user library first): the first root containing the pack
/// provides the sources.
pub struct FsInstallDestination {
    roots: Vec<std::path::PathBuf>,
}

impl FsInstallDestination {
    pub fn new(roots: Vec<std::path::PathBuf>) -> FsInstallDestination {
        FsInstallDestination { roots }
    }

    fn pack_dir(&self, pack_name: &str) -> std::path::PathBuf {
        self.roots
            .iter()
            .map(|root| root.join(pack_name))
            .find(|dir| dir.is_dir())
            .unwrap_or_else(|| self.roots[0].join(pack_name))
    }
}

impl InstallDestination for FsInstallDestination {
    fn copy(
        &self,
        pack_name: &str,
        source: &Path,
        destination: &Path,
    ) -> Result<(), ApplicationError> {
        let source = if source.is_absolute() {
            source.to_path_buf()
        } else {
            self.pack_dir(pack_name).join(source)
        };
        let source = source.as_path();
        if source.is_dir() {
            copy_dir(source, destination)
        } else {
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| io_err(&parent.to_string_lossy(), e))?;
            }
            std::fs::copy(source, destination)
                .map_err(|e| io_err(&destination.to_string_lossy(), e))?;
            Ok(())
        }
    }

    fn exists(&self, destination: &Path) -> bool {
        destination.exists()
    }

    fn remove(&self, path: &Path) -> Result<(), ApplicationError> {
        if !path.exists() {
            return Ok(());
        }
        if path.is_dir() {
            std::fs::remove_dir_all(path).map_err(|e| io_err(&path.to_string_lossy(), e))
        } else {
            std::fs::remove_file(path).map_err(|e| io_err(&path.to_string_lossy(), e))
        }
    }
}

fn copy_dir(source: &Path, destination: &Path) -> Result<(), ApplicationError> {
    std::fs::create_dir_all(destination).map_err(|e| io_err(&destination.to_string_lossy(), e))?;
    for entry in std::fs::read_dir(source).map_err(|e| io_err(&source.to_string_lossy(), e))? {
        let entry = entry.map_err(|e| io_err(&source.to_string_lossy(), e))?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).map_err(|e| io_err(&to.to_string_lossy(), e))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn list_directory_contents() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("dir")).unwrap();
        fs::write(tmp.path().join("file.txt"), "hi").unwrap();
        let listing = FsExplorer.list(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(listing.entries.len(), 2);
        assert!(listing.entries.iter().any(|e| e.name == "dir" && e.is_dir));
        assert!(listing
            .entries
            .iter()
            .any(|e| e.name == "file.txt" && !e.is_dir));
    }

    #[test]
    fn list_rejects_non_directory() {
        let err = FsExplorer.list("/definitely/not/here").unwrap_err();
        assert!(matches!(err, ApplicationError::NotADirectory(_)));
    }

    #[test]
    fn rename_moves_entry() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("old.txt"), "hi").unwrap();
        FsExplorer
            .rename(tmp.path().to_str().unwrap(), "old.txt", "new.txt")
            .unwrap();
        assert!(tmp.path().join("new.txt").is_file());
        assert!(!tmp.path().join("old.txt").exists());
    }

    #[test]
    fn remove_deletes_file_and_tree() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("file.txt"), "hi").unwrap();
        fs::create_dir_all(tmp.path().join("tree/inner")).unwrap();
        fs::write(tmp.path().join("tree/inner/x.md"), "#").unwrap();

        FsExplorer
            .remove(tmp.path().to_str().unwrap(), "file.txt")
            .unwrap();
        assert!(!tmp.path().join("file.txt").exists());

        FsExplorer
            .remove(tmp.path().to_str().unwrap(), "tree")
            .unwrap();
        assert!(!tmp.path().join("tree").exists());
    }

    #[test]
    fn create_dir_creates_folder() {
        let tmp = TempDir::new().unwrap();
        FsExplorer
            .create_dir(tmp.path().to_str().unwrap(), "new")
            .unwrap();
        assert!(tmp.path().join("new").is_dir());
    }

    #[test]
    fn install_destination_remove_deletes_file_tree_and_tolerates_missing() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("repo/.agents/skills/tdd")).unwrap();
        fs::write(tmp.path().join("repo/.agents/skills/tdd/SKILL.md"), "#").unwrap();
        fs::write(tmp.path().join("repo/file.txt"), "x").unwrap();

        let dest = FsInstallDestination::new(vec![tmp.path().join("packs")]);
        dest.remove(&tmp.path().join("repo/.agents/skills/tdd"))
            .unwrap();
        assert!(!tmp.path().join("repo/.agents/skills/tdd").exists());
        dest.remove(&tmp.path().join("repo/file.txt")).unwrap();
        assert!(!tmp.path().join("repo/file.txt").exists());
        // Idempotent: a missing path is not an error.
        dest.remove(&tmp.path().join("repo/file.txt")).unwrap();
    }

    #[test]
    fn install_destination_copies_file_and_tree() {
        let tmp = TempDir::new().unwrap();
        let pack_dir = tmp.path().join("packs/tdd");
        fs::create_dir_all(pack_dir.join("skills/tdd")).unwrap();
        fs::write(pack_dir.join("skills/tdd/SKILL.md"), "# TDD").unwrap();

        let target = tmp.path().join("repo");
        fs::create_dir_all(&target).unwrap();

        let dest = FsInstallDestination::new(vec![tmp.path().join("packs")]);
        dest.copy(
            "tdd",
            Path::new("skills/tdd/SKILL.md"),
            &target.join(".agents/skills/tdd/SKILL.md"),
        )
        .unwrap();
        assert!(target.join(".agents/skills/tdd/SKILL.md").is_file());

        dest.copy(
            "tdd",
            Path::new("skills/tdd"),
            &target.join(".agents/skills/tdd-tree"),
        )
        .unwrap();
        assert!(target.join(".agents/skills/tdd-tree/SKILL.md").is_file());
    }

    #[test]
    fn copy_resolves_sources_from_the_first_root_containing_the_pack() {
        let tmp = TempDir::new().unwrap();
        let user_packs = tmp.path().join("user-packs");
        let system_packs = tmp.path().join("system-packs");
        fs::create_dir_all(system_packs.join("tdd/skills/tdd")).unwrap();
        fs::write(system_packs.join("tdd/skills/tdd/SKILL.md"), "# system").unwrap();

        let target = tmp.path().join("repo");
        fs::create_dir_all(&target).unwrap();
        let dest = FsInstallDestination::new(vec![user_packs, system_packs]);
        dest.copy(
            "tdd",
            Path::new("skills/tdd/SKILL.md"),
            &target.join(".agents/skills/tdd/SKILL.md"),
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(target.join(".agents/skills/tdd/SKILL.md")).unwrap(),
            "# system"
        );
    }
}
