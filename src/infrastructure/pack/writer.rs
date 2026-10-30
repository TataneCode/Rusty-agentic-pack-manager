use std::path::{Path, PathBuf};

use crate::application::error::ApplicationError;
use crate::application::pack_create::dto::{PackSourceDto, PackWriteOutcomeDto};
use crate::application::pack_create::traits::PackWriter;
use crate::domain::pack::entity::Pack;
use crate::infrastructure::pack::loader::Manifest;

fn io_err(path: &Path, source: std::io::Error) -> ApplicationError {
    ApplicationError::Io {
        path: path.to_string_lossy().into_owned(),
        source,
    }
}

/// Writes new packs into the user pack library (the writable root of the
/// library). Strategy: validate every source, stage the whole pack in a
/// sibling `.staging-<name>-<pid>` directory, then swap — the existing pack
/// (if any) is renamed aside, the staging dir is renamed into place, and
/// the old directory is removed. A failure leaves no partial pack behind.
pub struct FsPackWriter {
    /// Writable library root (user library).
    user_root: PathBuf,
}

impl FsPackWriter {
    pub fn new(user_root: PathBuf) -> FsPackWriter {
        FsPackWriter { user_root }
    }

    fn stage_dir(&self, name: &str) -> PathBuf {
        self.user_root
            .join(format!(".staging-{name}-{}", std::process::id()))
    }

    fn validate_sources(&self, sources: &[PackSourceDto]) -> Result<(), ApplicationError> {
        for source in sources {
            if !source.host_path.exists() {
                return Err(ApplicationError::FileNotFound(
                    source.host_path.to_string_lossy().into_owned(),
                ));
            }
        }
        Ok(())
    }

    fn stage(&self, pack: &Pack, sources: &[PackSourceDto]) -> Result<PathBuf, ApplicationError> {
        let staging = self.stage_dir(pack.name.as_str());
        // `create_dir_all` also creates the user root itself.
        std::fs::create_dir_all(&staging).map_err(|e| io_err(&staging, e))?;
        for source in sources {
            let destination = staging.join(&source.pack_source);
            if source.host_path.is_dir() {
                copy_dir(&source.host_path, &destination)?;
            } else {
                if let Some(parent) = destination.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;
                }
                std::fs::copy(&source.host_path, &destination)
                    .map_err(|e| io_err(&destination, e))?;
            }
        }
        let manifest = toml::to_string_pretty(&Manifest::from_pack(pack)).map_err(|err| {
            ApplicationError::InvalidManifest {
                path: staging.join("pack.toml").to_string_lossy().into_owned(),
                message: err.to_string(),
            }
        })?;
        std::fs::write(staging.join("pack.toml"), manifest)
            .map_err(|e| io_err(&staging.join("pack.toml"), e))?;
        Ok(staging)
    }
}

impl PackWriter for FsPackWriter {
    fn write(
        &self,
        pack: &Pack,
        sources: &[PackSourceDto],
    ) -> Result<PackWriteOutcomeDto, ApplicationError> {
        self.validate_sources(sources)?;

        let final_dir = self.user_root.join(pack.name.as_str());
        let staging = self.stage_dir(pack.name.as_str());
        // A previous run may have left a staging dir behind: start clean.
        if staging.exists() {
            std::fs::remove_dir_all(&staging).map_err(|e| io_err(&staging, e))?;
        }

        let result = self.stage(pack, sources);
        let staging = match result {
            Ok(staging) => staging,
            Err(err) => {
                let _ = std::fs::remove_dir_all(&staging);
                return Err(err);
            }
        };

        // Swap: move any existing pack aside, rename staging into place,
        // then remove the old directory.
        let backup =
            self.user_root
                .join(format!("{}.old-{}", pack.name.as_str(), std::process::id()));
        if final_dir.exists() {
            std::fs::rename(&final_dir, &backup).map_err(|e| {
                let _ = std::fs::remove_dir_all(&staging);
                io_err(&final_dir, e)
            })?;
        }
        if let Err(err) = std::fs::rename(&staging, &final_dir) {
            // Roll back the aside rename so the old pack stays in place.
            if backup.exists() {
                let _ = std::fs::rename(&backup, &final_dir);
            }
            let _ = std::fs::remove_dir_all(&staging);
            return Err(io_err(&final_dir, err));
        }
        if backup.exists() {
            // Best effort: a failure here leaves a `<name>.old-<pid>`
            // directory behind (documented accepted limit).
            let _ = std::fs::remove_dir_all(&backup);
        }

        Ok(PackWriteOutcomeDto {
            pack_dir: final_dir,
            copied_count: sources.len(),
        })
    }
}

fn copy_dir(source: &Path, destination: &Path) -> Result<(), ApplicationError> {
    std::fs::create_dir_all(destination).map_err(|e| io_err(destination, e))?;
    for entry in std::fs::read_dir(source).map_err(|e| io_err(source, e))? {
        let entry = entry.map_err(|e| io_err(source, e))?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).map_err(|e| io_err(&to, e))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::pack::entity::PackItem;
    use crate::domain::pack::value_objects::{
        PackItemKind, PackName, PackVersion, TargetCli, TargetCliMap,
    };
    use std::fs;
    use tempfile::TempDir;

    fn sample_pack() -> Pack {
        Pack::new(
            PackName::new("my-pack").unwrap(),
            PackVersion::new("1.0.0").unwrap(),
            "A test pack".to_string(),
            vec![TargetCli::Vibe, TargetCli::Gemini],
            vec![
                PackItem::new(
                    PackItemKind::Skill,
                    PathBuf::from("skills/tdd"),
                    TargetCliMap::from_pairs(&[
                        (TargetCli::Vibe, ".agents/skills/tdd/"),
                        (TargetCli::Gemini, ".gemini/skills/tdd/"),
                    ]),
                )
                .unwrap(),
                PackItem::new(
                    PackItemKind::Command,
                    PathBuf::from("commands/review.toml"),
                    TargetCliMap::from_pairs(&[
                        (TargetCli::Vibe, ".vibe/commands/review.toml"),
                        (TargetCli::Gemini, ".gemini/commands/review.toml"),
                    ]),
                )
                .unwrap(),
            ],
        )
        .unwrap()
    }

    fn sample_sources(host: &Path) -> Vec<PackSourceDto> {
        vec![
            PackSourceDto {
                host_path: host.join("tdd"),
                pack_source: PathBuf::from("skills/tdd"),
            },
            PackSourceDto {
                host_path: host.join("review.toml"),
                pack_source: PathBuf::from("commands/review.toml"),
            },
        ]
    }

    /// A host directory holding one skill directory and one command file.
    fn host_tree(tmp: &TempDir) -> PathBuf {
        let host = tmp.path().join("host");
        fs::create_dir_all(host.join("tdd/scripts")).unwrap();
        fs::write(host.join("tdd/SKILL.md"), "# TDD").unwrap();
        fs::write(host.join("tdd/scripts/run.sh"), "#!/bin/sh").unwrap();
        fs::write(host.join("review.toml"), "description = 'review'").unwrap();
        host
    }

    #[test]
    fn write_copies_sources_and_manifest_into_the_user_root() {
        let tmp = TempDir::new().unwrap();
        let user_root = tmp.path().join("packs");
        fs::create_dir_all(&user_root).unwrap();
        let host = host_tree(&tmp);

        let writer = FsPackWriter::new(user_root.clone());
        let outcome = writer
            .write(&sample_pack(), &sample_sources(&host))
            .unwrap();

        assert_eq!(outcome.copied_count, 2);
        let pack_dir = user_root.join("my-pack");
        assert_eq!(outcome.pack_dir, pack_dir);
        assert!(pack_dir.join("skills/tdd/SKILL.md").is_file());
        assert!(pack_dir.join("skills/tdd/scripts/run.sh").is_file());
        assert!(pack_dir.join("commands/review.toml").is_file());

        let raw = fs::read_to_string(pack_dir.join("pack.toml")).unwrap();
        let manifest: Manifest = toml::from_str(&raw).unwrap();
        assert_eq!(manifest.name, "my-pack");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.description, "A test pack");
        assert_eq!(manifest.cli, vec!["vibe".to_string(), "gemini".to_string()]);
        assert_eq!(manifest.items.len(), 2);
        assert_eq!(manifest.items[0].kind, "skill");
        assert_eq!(manifest.items[0].source, "skills/tdd");
        assert_eq!(
            manifest.items[0].destinations.get("vibe").unwrap(),
            ".agents/skills/tdd/"
        );
        // No staging or backup directory is left behind.
        let leftovers: Vec<_> = fs::read_dir(&user_root)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(leftovers, vec![std::ffi::OsString::from("my-pack")]);
    }

    #[test]
    fn write_rejects_a_missing_source_without_writing_anything() {
        let tmp = TempDir::new().unwrap();
        let user_root = tmp.path().join("packs");
        fs::create_dir_all(&user_root).unwrap();

        let writer = FsPackWriter::new(user_root.clone());
        let err = writer
            .write(
                &sample_pack(),
                &[PackSourceDto {
                    host_path: tmp.path().join("does-not-exist"),
                    pack_source: PathBuf::from("skills/ghost"),
                }],
            )
            .unwrap_err();
        assert!(matches!(err, ApplicationError::FileNotFound(_)));
        // Nothing was written: no pack dir, no staging dir.
        assert_eq!(fs::read_dir(&user_root).unwrap().count(), 0);
    }

    #[test]
    fn write_replaces_an_existing_pack() {
        let tmp = TempDir::new().unwrap();
        let user_root = tmp.path().join("packs");
        let old = user_root.join("my-pack");
        fs::create_dir_all(&old).unwrap();
        fs::write(old.join("stale.txt"), "old").unwrap();
        let host = host_tree(&tmp);

        let writer = FsPackWriter::new(user_root.clone());
        writer
            .write(&sample_pack(), &sample_sources(&host))
            .unwrap();

        let pack_dir = user_root.join("my-pack");
        assert!(pack_dir.join("pack.toml").is_file());
        assert!(!pack_dir.join("stale.txt").exists());
        // No `.old` backup is left behind.
        let leftovers: Vec<_> = fs::read_dir(&user_root)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(leftovers, vec![std::ffi::OsString::from("my-pack")]);
    }

    #[test]
    fn write_creates_the_user_root_when_missing() {
        let tmp = TempDir::new().unwrap();
        let user_root = tmp.path().join("packs");
        // The user root does not exist yet: staging creates it (the DI
        // point also creates it upfront via `PackLibrary::ensure_user_root`).
        let host = host_tree(&tmp);
        let writer = FsPackWriter::new(user_root.clone());
        writer
            .write(&sample_pack(), &sample_sources(&host))
            .unwrap();
        assert!(user_root.join("my-pack/pack.toml").is_file());
    }
}
