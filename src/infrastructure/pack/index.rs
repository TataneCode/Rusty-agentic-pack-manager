use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::application::error::ApplicationError;
use crate::application::pack::traits::InstallIndexRepository;
use crate::domain::pack::installed::{InstalledPack, INDEX_FILENAME};
use crate::domain::pack::value_objects::{PackName, PackVersion, TargetCli};

/// Raw packs_index.toml shape: one entry per installed (pack, CLI).
#[derive(Debug, Default, Serialize, Deserialize)]
struct IndexFile {
    #[serde(default)]
    packs: Vec<IndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexEntry {
    name: String,
    version: String,
    cli: String,
    paths: Vec<PathBuf>,
}

/// Reads and writes the installation index (`packs_index.toml`) at the root
/// of a target repository.
pub struct FsInstallIndexRepository;

impl FsInstallIndexRepository {
    fn index_path(target_root: &str) -> PathBuf {
        Path::new(target_root).join(INDEX_FILENAME)
    }

    fn parse(path: &Path, raw: &str) -> Result<IndexFile, ApplicationError> {
        toml::from_str(raw).map_err(|err| ApplicationError::InvalidManifest {
            path: path.to_string_lossy().into_owned(),
            message: err.to_string(),
        })
    }

    fn load_file(&self, target_root: &str) -> Result<Vec<InstalledPack>, ApplicationError> {
        let path = Self::index_path(target_root);
        if !path.is_file() {
            return Ok(Vec::new());
        }
        let raw = std::fs::read_to_string(&path).map_err(|source| ApplicationError::Io {
            path: path.to_string_lossy().into_owned(),
            source,
        })?;
        let index = Self::parse(&path, &raw)?;

        let mut entries = Vec::new();
        for raw_entry in &index.packs {
            let name = PackName::new(&raw_entry.name)?;
            let version = PackVersion::new(&raw_entry.version)?;
            let cli = TargetCli::parse(&raw_entry.cli)?;
            entries.push(InstalledPack::new(
                name,
                version,
                cli,
                raw_entry.paths.clone(),
            )?);
        }
        Ok(entries)
    }

    fn save_file(
        &self,
        target_root: &str,
        entries: &[InstalledPack],
    ) -> Result<(), ApplicationError> {
        let path = Self::index_path(target_root);
        let index = IndexFile {
            packs: entries
                .iter()
                .map(|entry| IndexEntry {
                    name: entry.name.as_str().to_string(),
                    version: entry.version.as_str().to_string(),
                    cli: entry.cli.as_str().to_string(),
                    paths: entry.paths.clone(),
                })
                .collect(),
        };
        let raw =
            toml::to_string_pretty(&index).map_err(|err| ApplicationError::InvalidManifest {
                path: path.to_string_lossy().into_owned(),
                message: err.to_string(),
            })?;
        std::fs::write(&path, raw).map_err(|source| ApplicationError::Io {
            path: path.to_string_lossy().into_owned(),
            source,
        })
    }
}

impl InstallIndexRepository for FsInstallIndexRepository {
    fn load(&self, target_root: &str) -> Result<Vec<InstalledPack>, ApplicationError> {
        self.load_file(target_root)
    }

    fn record(&self, target_root: &str, entry: &InstalledPack) -> Result<(), ApplicationError> {
        let mut entries = self.load_file(target_root)?;
        // Upsert by (name, cli): a reinstall updates the recorded paths.
        entries.retain(|e| !e.matches(entry.name.as_str(), entry.cli));
        entries.push(entry.clone());
        entries.sort_by(|a, b| {
            a.name
                .as_str()
                .cmp(b.name.as_str())
                .then_with(|| a.cli.cmp(&b.cli))
        });
        self.save_file(target_root, &entries)
    }

    fn remove(
        &self,
        target_root: &str,
        name: &str,
        cli: TargetCli,
    ) -> Result<(), ApplicationError> {
        let mut entries = self.load_file(target_root)?;
        entries.retain(|e| !e.matches(name, cli));
        self.save_file(target_root, &entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn entry(name: &str, cli: TargetCli, paths: &[&str]) -> InstalledPack {
        InstalledPack::new(
            PackName::new(name).unwrap(),
            PackVersion::new("1.0.0").unwrap(),
            cli,
            paths.iter().map(PathBuf::from).collect(),
        )
        .unwrap()
    }

    #[test]
    fn missing_index_loads_as_empty() {
        let tmp = TempDir::new().unwrap();
        let repo = FsInstallIndexRepository;
        let entries = repo.load(tmp.path().to_str().unwrap()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn record_creates_index_and_upserts_on_reinstall() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_str().unwrap();
        let repo = FsInstallIndexRepository;

        repo.record(
            root,
            &entry("tdd", TargetCli::Vibe, &[".agents/skills/tdd"]),
        )
        .unwrap();
        repo.record(
            root,
            &entry("tdd", TargetCli::Gemini, &[".gemini/skills/tdd"]),
        )
        .unwrap();
        repo.record(
            root,
            &entry("tdd", TargetCli::Vibe, &[".agents/skills/tdd-v2"]),
        )
        .unwrap();

        let entries = repo.load(root).unwrap();
        assert_eq!(entries.len(), 2);
        let vibe = entries
            .iter()
            .find(|e| e.matches("tdd", TargetCli::Vibe))
            .unwrap();
        assert_eq!(vibe.paths, vec![PathBuf::from(".agents/skills/tdd-v2")]);

        let raw = fs::read_to_string(tmp.path().join(INDEX_FILENAME)).unwrap();
        assert!(raw.contains("name = \"tdd\""));
        assert!(raw.contains("cli = \"gemini\""));
    }

    #[test]
    fn remove_drops_only_the_matching_entry() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_str().unwrap();
        let repo = FsInstallIndexRepository;

        repo.record(
            root,
            &entry("tdd", TargetCli::Vibe, &[".agents/skills/tdd"]),
        )
        .unwrap();
        repo.record(
            root,
            &entry("review", TargetCli::Vibe, &[".agents/skills/review"]),
        )
        .unwrap();
        repo.remove(root, "tdd", TargetCli::Vibe).unwrap();

        let entries = repo.load(root).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].matches("review", TargetCli::Vibe));
    }

    #[test]
    fn corrupt_index_is_an_invalid_manifest_error() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join(INDEX_FILENAME), "not = [toml").unwrap();
        let repo = FsInstallIndexRepository;
        let err = repo.load(tmp.path().to_str().unwrap()).unwrap_err();
        assert!(matches!(err, ApplicationError::InvalidManifest { .. }));
    }

    #[test]
    fn entry_with_escaping_path_is_rejected() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(INDEX_FILENAME),
            r#"
[[packs]]
name = "evil"
version = "1.0.0"
cli = "vibe"
paths = ["../outside"]
"#,
        )
        .unwrap();
        let repo = FsInstallIndexRepository;
        let err = repo.load(tmp.path().to_str().unwrap()).unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(
                crate::domain::pack::error::DomainError::InstallPathOutsideTarget(_)
            )
        ));
    }
}
