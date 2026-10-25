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
}

/// The pack source files live in the packs dir; destinations are absolute.
/// We need both sides, so the installer adapter is separate from the
/// explorer one but shares the same std::fs operations.
pub struct FsInstallDestination {
    packs_dir: std::path::PathBuf,
}

impl FsInstallDestination {
    pub fn new(packs_dir: std::path::PathBuf) -> FsInstallDestination {
        FsInstallDestination { packs_dir }
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
            self.packs_dir.join(pack_name).join(source)
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
    fn create_dir_creates_folder() {
        let tmp = TempDir::new().unwrap();
        FsExplorer
            .create_dir(tmp.path().to_str().unwrap(), "new")
            .unwrap();
        assert!(tmp.path().join("new").is_dir());
    }

    #[test]
    fn install_destination_copies_file_and_tree() {
        let tmp = TempDir::new().unwrap();
        let pack_dir = tmp.path().join("packs/tdd");
        fs::create_dir_all(pack_dir.join("skills/tdd")).unwrap();
        fs::write(pack_dir.join("skills/tdd/SKILL.md"), "# TDD").unwrap();

        let target = tmp.path().join("repo");
        fs::create_dir_all(&target).unwrap();

        let dest = FsInstallDestination::new(tmp.path().join("packs"));
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
}
