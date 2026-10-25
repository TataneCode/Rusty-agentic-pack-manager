use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::application::error::ApplicationError;
use crate::application::pack::traits::PackRepository;
use crate::domain::pack::entity::{Pack, PackItem};
use crate::domain::pack::value_objects::{
    PackItemKind, PackName, PackVersion, TargetCli, TargetCliMap,
};

/// Raw pack.toml shape.
#[derive(Debug, Deserialize)]
struct Manifest {
    name: String,
    version: String,
    #[serde(default)]
    description: String,
    cli: Vec<String>,
    #[serde(default, rename = "items")]
    items: Vec<ManifestItem>,
}

#[derive(Debug, Deserialize)]
struct ManifestItem {
    kind: String,
    source: String,
    #[serde(rename = "destinations")]
    destinations: BTreeMap<String, String>,
}

/// Loads packs from a directory of pack directories, each with pack.toml.
#[derive(Debug)]
pub struct FsPackRepository {
    packs_dir: PathBuf,
}

impl FsPackRepository {
    pub fn new(packs_dir: PathBuf) -> Result<FsPackRepository, ApplicationError> {
        if !packs_dir.is_dir() {
            return Err(ApplicationError::NotADirectory(
                packs_dir.to_string_lossy().into_owned(),
            ));
        }
        Ok(FsPackRepository { packs_dir })
    }

    pub fn packs_dir(&self) -> &Path {
        &self.packs_dir
    }

    fn pack_dir(&self, name: &str) -> PathBuf {
        self.packs_dir.join(name)
    }

    fn load_pack(&self, dir: &Path) -> Result<Pack, ApplicationError> {
        let manifest_path = dir.join("pack.toml");
        let raw =
            std::fs::read_to_string(&manifest_path).map_err(|source| ApplicationError::Io {
                path: manifest_path.to_string_lossy().into_owned(),
                source,
            })?;
        let manifest: Manifest =
            toml::from_str(&raw).map_err(|err| ApplicationError::InvalidManifest {
                path: manifest_path.to_string_lossy().into_owned(),
                message: err.to_string(),
            })?;

        let name = PackName::new(&manifest.name)?;
        let version = PackVersion::new(&manifest.version)?;
        let clis = manifest
            .cli
            .iter()
            .map(|c| TargetCli::parse(c))
            .collect::<Result<Vec<TargetCli>, _>>()?;

        let mut items = Vec::new();
        for raw_item in &manifest.items {
            let kind = PackItemKind::parse(&raw_item.kind).map_err(|_| {
                ApplicationError::InvalidManifest {
                    path: manifest_path.to_string_lossy().into_owned(),
                    message: format!("unknown item kind: {}", raw_item.kind),
                }
            })?;
            let mut destinations = TargetCliMap::default();
            for (cli_raw, dest) in &raw_item.destinations {
                let cli = TargetCli::parse(cli_raw)?;
                destinations.insert(cli, dest.clone());
            }
            items.push(PackItem::new(
                kind,
                PathBuf::from(&raw_item.source),
                destinations,
            )?);
        }

        Ok(Pack::new(name, version, manifest.description, clis, items)?)
    }
}

impl PackRepository for FsPackRepository {
    fn list(&self) -> Result<Vec<Pack>, ApplicationError> {
        let mut packs = Vec::new();
        let entries =
            std::fs::read_dir(&self.packs_dir).map_err(|source| ApplicationError::Io {
                path: self.packs_dir.to_string_lossy().into_owned(),
                source,
            })?;
        for entry in entries {
            let entry = entry.map_err(|source| ApplicationError::Io {
                path: self.packs_dir.to_string_lossy().into_owned(),
                source,
            })?;
            let path = entry.path();
            if path.is_dir() && path.join("pack.toml").is_file() {
                packs.push(self.load_pack(&path)?);
            }
        }
        packs.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
        Ok(packs)
    }

    fn find(&self, name: &str) -> Result<Pack, ApplicationError> {
        let dir = self.pack_dir(name);
        if !dir.join("pack.toml").is_file() {
            return Err(ApplicationError::PackNotFound(name.to_string()));
        }
        self.load_pack(&dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    const TDD_MANIFEST: &str = r#"
name = "tdd"
version = "1.0.0"
description = "TDD pack"
cli = ["vibe"]

[[items]]
kind = "skill"
source = "skills/tdd/SKILL.md"

[items.destinations]
vibe = ".agents/skills/tdd/"
gemini = ".gemini/commands/tdd/"
"#;

    fn write_pack(dir: &Path, name: &str, manifest: &str) {
        let pack_dir = dir.join(name);
        fs::create_dir_all(&pack_dir).unwrap();
        fs::write(pack_dir.join("pack.toml"), manifest).unwrap();
    }

    #[test]
    fn list_and_find_pack() {
        let tmp = TempDir::new().unwrap();
        write_pack(tmp.path(), "tdd", TDD_MANIFEST);

        let repo = FsPackRepository::new(tmp.path().to_path_buf()).unwrap();
        let packs = repo.list().unwrap();
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].name.as_str(), "tdd");
        assert_eq!(packs[0].items.len(), 1);

        let pack = repo.find("tdd").unwrap();
        assert_eq!(
            pack.items[0].destination_for(TargetCli::Gemini).unwrap(),
            ".gemini/commands/tdd/"
        );
    }

    #[test]
    fn find_missing_pack() {
        let tmp = TempDir::new().unwrap();
        let repo = FsPackRepository::new(tmp.path().to_path_buf()).unwrap();
        let err = repo.find("nope").unwrap_err();
        assert!(matches!(err, ApplicationError::PackNotFound(_)));
    }

    #[test]
    fn invalid_manifest_is_reported() {
        let tmp = TempDir::new().unwrap();
        write_pack(tmp.path(), "bad", "name = 42");
        let repo = FsPackRepository::new(tmp.path().to_path_buf()).unwrap();
        assert!(matches!(
            repo.list().unwrap_err(),
            ApplicationError::InvalidManifest { .. }
        ));
    }

    #[test]
    fn non_pack_dirs_are_ignored() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir_all(tmp.path().join("not-a-pack")).unwrap();
        write_pack(tmp.path(), "tdd", TDD_MANIFEST);
        let repo = FsPackRepository::new(tmp.path().to_path_buf()).unwrap();
        assert_eq!(repo.list().unwrap().len(), 1);
    }

    #[test]
    fn missing_packs_dir_is_rejected() {
        let err = FsPackRepository::new(PathBuf::from("/definitely/not/here")).unwrap_err();
        assert!(matches!(err, ApplicationError::NotADirectory(_)));
    }
}
