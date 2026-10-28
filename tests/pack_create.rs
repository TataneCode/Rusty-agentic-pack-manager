//! Integration: `rapm pack create` builds a pack in the user library from
//! an arbitrary list of files/directories, and the pack is then loadable
//! and installable through the regular pack pipeline. Wired exactly like
//! main.rs: real adapters, tempdir library, no TUI.

use std::fs;
use std::path::{Path, PathBuf};

use rusty_agentic_pack_manager::application::error::ApplicationError;
use rusty_agentic_pack_manager::application::pack::service::PackService;
use rusty_agentic_pack_manager::application::pack::traits::PackRepository;
use rusty_agentic_pack_manager::application::pack_create::dto::{CreateItemDto, CreatePackDto};
use rusty_agentic_pack_manager::application::pack_create::service::PackCreationService;
use rusty_agentic_pack_manager::domain::pack::value_objects::TargetCli;
use rusty_agentic_pack_manager::infrastructure::filesystem::explorer::FsInstallDestination;
use rusty_agentic_pack_manager::infrastructure::pack::index::FsInstallIndexRepository;
use rusty_agentic_pack_manager::infrastructure::pack::loader::FsPackRepository;
use rusty_agentic_pack_manager::infrastructure::pack::writer::FsPackWriter;
use rusty_agentic_pack_manager::presentation::library::PackLibrary;
use tempfile::TempDir;

/// The CLI wiring of main.rs: a library rooted at a user root (first) and
/// a system root (fallback), a creation service writing into the user root.
fn wired(tmp: &TempDir) -> (PackLibrary, PackCreationService, PackService) {
    let user_root = tmp.path().join("user-packs");
    let system_root = tmp.path().join("system-packs");
    fs::create_dir_all(&user_root).unwrap();
    fs::create_dir_all(&system_root).unwrap();
    let library = PackLibrary::new(vec![user_root.clone(), system_root], user_root.clone());
    library.ensure_user_root().unwrap();
    let repository = FsPackRepository::new(library.roots.clone()).unwrap();
    let writer = FsPackWriter::new(user_root.clone());
    let creation = PackCreationService::new(
        Box::new(FsPackRepository::new(library.roots.clone()).unwrap()),
        Box::new(writer),
    );
    let install = PackService::new(
        Box::new(repository),
        Box::new(FsInstallDestination::new(library.roots.clone())),
        Box::new(FsInstallIndexRepository),
    );
    (library, creation, install)
}

/// A host directory with an arbitrary layout: a skill directory (with a
/// nested script), an agent file and a command file.
fn host_tree(tmp: &TempDir) -> PathBuf {
    let host = tmp.path().join("host");
    fs::create_dir_all(host.join("my-stuff/tdd/scripts")).unwrap();
    fs::write(host.join("my-stuff/tdd/SKILL.md"), "# TDD").unwrap();
    fs::write(host.join("my-stuff/tdd/scripts/run.sh"), "#!/bin/sh").unwrap();
    fs::write(host.join("reviewer.toml"), "name = 'reviewer'").unwrap();
    fs::write(host.join("review.toml"), "description = 'review'").unwrap();
    host
}

fn skill_dto(host: &Path) -> CreateItemDto {
    CreateItemDto {
        kind: "skill".to_string(),
        source: host.join("my-stuff/tdd"),
        destinations: vec![
            ("vibe".to_string(), ".agents/skills/tdd/".to_string()),
            ("gemini".to_string(), ".gemini/skills/tdd/".to_string()),
        ],
    }
}

fn command_dto(host: &Path) -> CreateItemDto {
    CreateItemDto {
        kind: "command".to_string(),
        source: host.join("review.toml"),
        destinations: vec![
            ("vibe".to_string(), ".vibe/commands/review.toml".to_string()),
            (
                "gemini".to_string(),
                ".gemini/commands/review.toml".to_string(),
            ),
        ],
    }
}

fn agent_dto(host: &Path) -> CreateItemDto {
    CreateItemDto {
        kind: "agent".to_string(),
        source: host.join("reviewer.toml"),
        destinations: vec![
            ("vibe".to_string(), ".vibe/agents/reviewer.toml".to_string()),
            (
                "gemini".to_string(),
                ".gemini/agents/reviewer.toml".to_string(),
            ),
        ],
    }
}

fn dto(items: Vec<CreateItemDto>) -> CreatePackDto {
    CreatePackDto {
        name: "my-pack".to_string(),
        version: "1.0.0".to_string(),
        description: "Created by the CLI".to_string(),
        clis: vec![],
        items,
        overwrite: false,
    }
}

#[test]
fn create_builds_a_pack_that_the_library_loads_and_installs() {
    let tmp = TempDir::new().unwrap();
    let (library, creation, install) = wired(&tmp);
    let host = host_tree(&tmp);

    let result = creation
        .create(&dto(vec![
            skill_dto(&host),
            command_dto(&host),
            agent_dto(&host),
        ]))
        .expect("creation must succeed");
    assert_eq!(result.pack_name, "my-pack");
    assert_eq!(result.copied_count, 3);

    // Canonical layout inside the user library.
    let pack_dir = library.user_root.join("my-pack");
    assert!(pack_dir.join("skills/tdd/SKILL.md").is_file());
    assert!(pack_dir.join("skills/tdd/scripts/run.sh").is_file());
    assert!(pack_dir.join("commands/review.toml").is_file());
    assert!(pack_dir.join("agents/reviewer.toml").is_file());
    let manifest = fs::read_to_string(pack_dir.join("pack.toml")).unwrap();
    assert!(manifest.contains("name = \"my-pack\""));

    // The pack is listed by the library...
    let repo = FsPackRepository::new(library.roots.clone()).unwrap();
    let packs = repo.list().unwrap();
    assert_eq!(packs.len(), 1);
    assert_eq!(packs[0].name.as_str(), "my-pack");
    assert_eq!(packs[0].items.len(), 3);
    // ...with the CLIs deduced from the items' destinations.
    assert_eq!(packs[0].clis, vec![TargetCli::Vibe, TargetCli::Gemini]);

    // ...and installable like any library pack.
    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();
    let installed = install
        .install("my-pack", TargetCli::Vibe, target.to_str().unwrap(), false)
        .expect("install must succeed");
    assert_eq!(installed.copied_count, 3);
    assert!(target.join(".agents/skills/tdd/SKILL.md").is_file());
    assert!(target.join(".vibe/commands/review.toml").is_file());
    assert!(target.join(".vibe/agents/reviewer.toml").is_file());
}

#[test]
fn create_refuses_to_overwrite_without_force() {
    let tmp = TempDir::new().unwrap();
    let (library, creation, _) = wired(&tmp);
    let host = host_tree(&tmp);

    creation
        .create(&dto(vec![skill_dto(&host)]))
        .expect("first creation must succeed");
    let err = creation.create(&dto(vec![skill_dto(&host)])).unwrap_err();
    assert!(matches!(err, ApplicationError::PackAlreadyExists(_)));

    // --force replaces the pack.
    let mut forced = dto(vec![skill_dto(&host)]);
    forced.overwrite = true;
    creation.create(&forced).expect("overwrite must succeed");
    let repo = FsPackRepository::new(library.roots.clone()).unwrap();
    assert_eq!(repo.list().unwrap().len(), 1);
}

#[test]
fn create_rejects_a_missing_source_without_writing() {
    let tmp = TempDir::new().unwrap();
    let (library, creation, _) = wired(&tmp);
    let host = host_tree(&tmp);

    let mut bad = dto(vec![skill_dto(&host)]);
    bad.items.push(CreateItemDto {
        kind: "command".to_string(),
        source: host.join("ghost"),
        destinations: vec![
            ("vibe".to_string(), ".vibe/commands/ghost.toml".to_string()),
            (
                "gemini".to_string(),
                ".gemini/commands/ghost.toml".to_string(),
            ),
        ],
    });
    // Distinct kind: the duplicate-source check must not fire first.
    let err = creation.create(&bad).unwrap_err();
    assert!(matches!(err, ApplicationError::FileNotFound(_)));

    let repo = FsPackRepository::new(library.roots.clone()).unwrap();
    assert!(repo.list().unwrap().is_empty());
    // No staging directory is left behind.
    let leftovers: Vec<_> = fs::read_dir(&library.user_root)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert!(leftovers.is_empty());
}

#[test]
fn user_pack_overrides_a_system_pack_of_the_same_name() {
    let tmp = TempDir::new().unwrap();
    let (library, creation, _) = wired(&tmp);
    let host = host_tree(&tmp);

    // A system pack named my-pack already exists.
    let system_pack = library.roots[1].join("my-pack");
    fs::create_dir_all(&system_pack).unwrap();
    fs::write(
        system_pack.join("pack.toml"),
        "name = \"my-pack\"\nversion = \"0.1.0\"\ncli = [\"vibe\"]\n",
    )
    .unwrap();

    // The library sees the system version first.
    let repo = FsPackRepository::new(library.roots.clone()).unwrap();
    assert_eq!(repo.find("my-pack").unwrap().version.as_str(), "0.1.0");

    // Creating a user pack of the same name shadows it (overwrite via the
    // user root; the system copy is untouched).
    let mut forced = dto(vec![skill_dto(&host)]);
    forced.version = "2.0.0".to_string();
    forced.overwrite = true;
    creation.create(&forced).unwrap();

    let repo = FsPackRepository::new(library.roots.clone()).unwrap();
    assert_eq!(repo.find("my-pack").unwrap().version.as_str(), "2.0.0");
    // The system pack directory is untouched.
    assert!(system_pack.join("pack.toml").is_file());
}

#[test]
fn create_requires_a_destination_for_every_declared_cli() {
    let tmp = TempDir::new().unwrap();
    let (_, creation, _) = wired(&tmp);
    let host = host_tree(&tmp);

    let mut create = dto(vec![skill_dto(&host)]);
    create.clis = vec!["vibe".to_string(), "gemini".to_string(), "vibe".to_string()];
    // The skill has both destinations: creation succeeds and the CLI list
    // is deduplicated.
    creation.create(&create).expect("creation must succeed");

    // A different pack name: the existence check must not fire first.
    let mut partial = dto(vec![CreateItemDto {
        kind: "skill".to_string(),
        source: host.join("my-stuff/tdd"),
        destinations: vec![("vibe".to_string(), ".agents/skills/tdd/".to_string())],
    }]);
    partial.name = "partial-pack".to_string();
    partial.clis = vec!["vibe".to_string(), "gemini".to_string()];
    let err = creation.create(&partial).unwrap_err();
    assert!(matches!(
        err,
        ApplicationError::Domain(
            rusty_agentic_pack_manager::domain::pack::error::DomainError::EmptyItemDestination
        )
    ));
}
