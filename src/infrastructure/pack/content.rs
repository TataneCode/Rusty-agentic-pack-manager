use std::path::PathBuf;

use crate::application::error::ApplicationError;
use crate::application::pack_view::traits::{PackContentRepository, PackFileDto};

/// Text extensions the viewer can display.
const TEXT_EXTENSIONS: [&str; 9] = ["md", "js", "toml", "txt", "json", "yaml", "yml", "sh", "py"];

/// Reads pack file contents from the pack library (viewer backing).
/// Roots are in read order (user library first): the first root containing
/// the pack wins.
pub struct FsPackContentRepository {
    roots: Vec<PathBuf>,
}

impl FsPackContentRepository {
    pub fn new(roots: Vec<PathBuf>) -> Result<FsPackContentRepository, ApplicationError> {
        let first = roots
            .first()
            .ok_or_else(|| ApplicationError::NotADirectory("(no pack library root)".to_string()))?;
        if !first.is_dir() {
            return Err(ApplicationError::NotADirectory(
                first.to_string_lossy().into_owned(),
            ));
        }
        Ok(FsPackContentRepository { roots })
    }

    fn pack_dir(&self, pack_name: &str) -> PathBuf {
        self.roots
            .iter()
            .map(|root| root.join(pack_name))
            .find(|dir| dir.is_dir())
            .unwrap_or_else(|| self.roots[0].join(pack_name))
    }

    fn collect_files(&self, dir: &std::path::Path, base: &std::path::Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                self.collect_files(&path, base, out);
            } else {
                out.push(rel);
            }
        }
    }

    fn is_text(path: &str) -> bool {
        path.rsplit('.')
            .next()
            .is_some_and(|ext| TEXT_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
    }
}

impl PackContentRepository for FsPackContentRepository {
    fn list_files(&self, pack_name: &str) -> Result<Vec<PackFileDto>, ApplicationError> {
        let pack_dir = self.pack_dir(pack_name);
        if !pack_dir.is_dir() {
            return Err(ApplicationError::PackNotFound(pack_name.to_string()));
        }
        let mut paths = Vec::new();
        self.collect_files(&pack_dir, &pack_dir, &mut paths);
        // pack.toml is included: the viewer opens on the manifest.
        Ok(paths
            .into_iter()
            .map(|path| PackFileDto {
                is_text: Self::is_text(&path),
                path,
            })
            .collect())
    }

    fn read_file(&self, pack_name: &str, path: &str) -> Result<String, ApplicationError> {
        let full = self.pack_dir(pack_name).join(path);
        if !full.is_file() {
            return Err(ApplicationError::FileNotFound(path.to_string()));
        }
        std::fs::read_to_string(&full).map_err(|source| ApplicationError::Io {
            path: full.to_string_lossy().into_owned(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn lists_and_reads_text_files_including_manifest() {
        let tmp = TempDir::new().unwrap();
        let pack = tmp.path().join("tdd");
        fs::create_dir_all(pack.join("skills/tdd")).unwrap();
        fs::write(pack.join("pack.toml"), "name = 'tdd'").unwrap();
        fs::write(pack.join("skills/tdd/SKILL.md"), "# TDD").unwrap();
        fs::write(pack.join("logo.png"), "binary").unwrap();

        let repo = FsPackContentRepository::new(vec![tmp.path().to_path_buf()]).unwrap();
        let files = repo.list_files("tdd").unwrap();
        assert_eq!(files.len(), 3);
        let manifest = files.iter().find(|f| f.path == "pack.toml").unwrap();
        assert!(manifest.is_text);
        let skill = files
            .iter()
            .find(|f| f.path == "skills/tdd/SKILL.md")
            .unwrap();
        assert!(skill.is_text);
        let logo = files.iter().find(|f| f.path == "logo.png").unwrap();
        assert!(!logo.is_text);

        assert_eq!(
            repo.read_file("tdd", "skills/tdd/SKILL.md").unwrap(),
            "# TDD"
        );
    }

    #[test]
    fn python_scripts_are_text() {
        let tmp = TempDir::new().unwrap();
        let pack = tmp.path().join("vibe-wise");
        fs::create_dir_all(pack.join("skills/vibe-wise-reset/scripts")).unwrap();
        fs::write(
            pack.join("skills/vibe-wise-reset/scripts/reset.py"),
            "print(1)",
        )
        .unwrap();

        let repo = FsPackContentRepository::new(vec![tmp.path().to_path_buf()]).unwrap();
        let files = repo.list_files("vibe-wise").unwrap();
        let script = files
            .iter()
            .find(|f| f.path == "skills/vibe-wise-reset/scripts/reset.py")
            .unwrap();
        assert!(script.is_text);
        assert_eq!(
            repo.read_file("vibe-wise", "skills/vibe-wise-reset/scripts/reset.py")
                .unwrap(),
            "print(1)"
        );
    }

    #[test]
    fn read_missing_file_and_pack() {
        let tmp = TempDir::new().unwrap();
        let repo = FsPackContentRepository::new(vec![tmp.path().to_path_buf()]).unwrap();
        assert!(matches!(
            repo.list_files("nope").unwrap_err(),
            ApplicationError::PackNotFound(_)
        ));
        let err = repo.read_file("nope", "x.md").unwrap_err();
        assert!(matches!(err, ApplicationError::FileNotFound(_)));
    }

    #[test]
    fn pack_dir_prefers_the_first_root() {
        let user = TempDir::new().unwrap();
        let system = TempDir::new().unwrap();
        fs::create_dir_all(user.path().join("tdd")).unwrap();
        fs::write(user.path().join("tdd/USER.md"), "user").unwrap();
        fs::create_dir_all(system.path().join("tdd")).unwrap();
        fs::write(system.path().join("tdd/SYSTEM.md"), "system").unwrap();

        let repo = FsPackContentRepository::new(vec![
            user.path().to_path_buf(),
            system.path().to_path_buf(),
        ])
        .unwrap();
        let files = repo.list_files("tdd").unwrap();
        assert!(files.iter().any(|f| f.path == "USER.md"));
        assert!(!files.iter().any(|f| f.path == "SYSTEM.md"));
    }
}
