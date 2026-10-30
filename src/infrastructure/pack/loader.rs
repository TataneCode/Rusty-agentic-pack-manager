use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::application::error::ApplicationError;
use crate::application::pack::traits::PackRepository;
use crate::domain::pack::entity::{Pack, PackItem};
use crate::domain::pack::value_objects::{
    PackItemKind, PackName, PackVersion, TargetCli, TargetCliMap,
};

/// Raw pack.toml shape (single source of truth for the manifest format:
/// the loader reads it, the pack writer serializes it).
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Manifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    pub cli: Vec<String>,
    #[serde(default, rename = "items")]
    pub items: Vec<ManifestItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ManifestItem {
    pub kind: String,
    pub source: String,
    #[serde(rename = "destinations")]
    pub destinations: BTreeMap<String, String>,
}

impl Manifest {
    /// Raw manifest of a pack entity, for serialization by the pack writer.
    pub(crate) fn from_pack(pack: &Pack) -> Manifest {
        Manifest {
            name: pack.name.as_str().to_string(),
            version: pack.version.as_str().to_string(),
            description: pack.description.clone(),
            cli: pack.clis.iter().map(|c| c.as_str().to_string()).collect(),
            items: pack
                .items
                .iter()
                .map(|item| ManifestItem {
                    kind: item.kind.as_str().to_string(),
                    source: item.source.to_string_lossy().replace('\\', "/"),
                    destinations: item
                        .destinations
                        .iter()
                        .map(|(cli, dest)| (cli.as_str().to_string(), dest.to_string()))
                        .collect(),
                })
                .collect(),
        }
    }
}

/// Loads packs from the pack library: an ordered list of roots (user
/// library first, then the system locations). The first root containing a
/// pack wins: a user pack overrides a system pack of the same name.
#[derive(Debug)]
pub struct FsPackRepository {
    roots: Vec<PathBuf>,
}

impl FsPackRepository {
    pub fn new(roots: Vec<PathBuf>) -> Result<FsPackRepository, ApplicationError> {
        let first = roots
            .first()
            .ok_or_else(|| ApplicationError::NotADirectory("(no pack library root)".to_string()))?;
        if !first.is_dir() {
            return Err(ApplicationError::NotADirectory(
                first.to_string_lossy().into_owned(),
            ));
        }
        Ok(FsPackRepository { roots })
    }

    /// First root (in read order) containing a pack directory named `name`.
    fn pack_dir(&self, name: &str) -> Option<PathBuf> {
        self.roots
            .iter()
            .map(|root| root.join(name))
            .find(|dir| dir.join("pack.toml").is_file())
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
        let mut seen = std::collections::HashSet::new();
        for root in &self.roots {
            let entries = std::fs::read_dir(root).map_err(|source| ApplicationError::Io {
                path: root.to_string_lossy().into_owned(),
                source,
            })?;
            for entry in entries {
                let entry = entry.map_err(|source| ApplicationError::Io {
                    path: root.to_string_lossy().into_owned(),
                    source,
                })?;
                let path = entry.path();
                // Malformed pack dirs are skipped (rule: a directory without
                // pack.toml is ignored in listings, not an error).
                if path.is_dir() && path.join("pack.toml").is_file() {
                    let pack = self.load_pack(&path)?;
                    // First root wins: a user pack overrides a system pack
                    // of the same name.
                    if seen.insert(pack.name.as_str().to_string()) {
                        packs.push(pack);
                    }
                }
            }
        }
        packs.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
        Ok(packs)
    }

    fn find(&self, name: &str) -> Result<Pack, ApplicationError> {
        let dir = self
            .pack_dir(name)
            .ok_or_else(|| ApplicationError::PackNotFound(name.to_string()))?;
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

    fn repo_with(roots: Vec<PathBuf>) -> FsPackRepository {
        FsPackRepository::new(roots).unwrap()
    }

    #[test]
    fn list_and_find_pack() {
        let tmp = TempDir::new().unwrap();
        write_pack(tmp.path(), "tdd", TDD_MANIFEST);

        let repo = repo_with(vec![tmp.path().to_path_buf()]);
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
        let repo = repo_with(vec![tmp.path().to_path_buf()]);
        let err = repo.find("nope").unwrap_err();
        assert!(matches!(err, ApplicationError::PackNotFound(_)));
    }

    #[test]
    fn invalid_manifest_is_reported() {
        let tmp = TempDir::new().unwrap();
        write_pack(tmp.path(), "bad", "name = 42");
        let repo = repo_with(vec![tmp.path().to_path_buf()]);
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
        let repo = repo_with(vec![tmp.path().to_path_buf()]);
        assert_eq!(repo.list().unwrap().len(), 1);
    }

    #[test]
    fn missing_first_root_is_rejected() {
        let err = FsPackRepository::new(vec![PathBuf::from("/definitely/not/here")]).unwrap_err();
        assert!(matches!(err, ApplicationError::NotADirectory(_)));
        assert!(FsPackRepository::new(vec![]).is_err());
    }

    #[test]
    fn list_unions_roots_and_dedupes_by_name_first_root_wins() {
        let user = TempDir::new().unwrap();
        let system = TempDir::new().unwrap();
        write_pack(user.path(), "tdd", TDD_MANIFEST);
        // Same pack name in the system root, plus a system-only pack.
        let other_manifest = TDD_MANIFEST
            .replace("tdd", "review")
            .replace("TDD pack", "Review");
        write_pack(system.path(), "tdd", &other_manifest);
        write_pack(system.path(), "review", &other_manifest);

        let repo = repo_with(vec![user.path().to_path_buf(), system.path().to_path_buf()]);
        let packs = repo.list().unwrap();
        assert_eq!(packs.len(), 2);
        let tdd = packs.iter().find(|p| p.name.as_str() == "tdd").unwrap();
        // The user copy wins: its description is the original one.
        assert_eq!(tdd.description, "TDD pack");
    }

    #[test]
    fn find_prefers_the_first_root() {
        let user = TempDir::new().unwrap();
        let system = TempDir::new().unwrap();
        write_pack(system.path(), "tdd", TDD_MANIFEST);
        let user_manifest = TDD_MANIFEST.replace("TDD pack", "User override");
        write_pack(user.path(), "tdd", &user_manifest);

        let repo = repo_with(vec![user.path().to_path_buf(), system.path().to_path_buf()]);
        let pack = repo.find("tdd").unwrap();
        assert_eq!(pack.description, "User override");
    }

    #[test]
    fn find_falls_back_to_later_roots() {
        let user = TempDir::new().unwrap();
        let system = TempDir::new().unwrap();
        write_pack(system.path(), "tdd", TDD_MANIFEST);

        let repo = repo_with(vec![user.path().to_path_buf(), system.path().to_path_buf()]);
        assert!(repo.find("tdd").is_ok());
    }

    #[test]
    fn manifest_from_pack_round_trips_through_toml() {
        let tmp = TempDir::new().unwrap();
        write_pack(tmp.path(), "tdd", TDD_MANIFEST);
        let repo = repo_with(vec![tmp.path().to_path_buf()]);
        let pack = repo.find("tdd").unwrap();

        let raw = toml::to_string_pretty(&Manifest::from_pack(&pack)).unwrap();
        let parsed: Manifest = toml::from_str(&raw).unwrap();
        assert_eq!(parsed.name, "tdd");
        assert_eq!(parsed.version, "1.0.0");
        assert_eq!(parsed.cli, vec!["vibe".to_string()]);
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].kind, "skill");
        assert_eq!(parsed.items[0].source, "skills/tdd/SKILL.md");
        assert_eq!(
            parsed.items[0].destinations.get("gemini").unwrap(),
            ".gemini/commands/tdd/"
        );
    }
}
