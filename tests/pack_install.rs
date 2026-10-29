//! Integration: a real pack from disk installs into a temp target repo.
//! No TUI, no Docker-style daemon: pure filesystem, wired exactly like
//! main.rs.

use std::fs;
use std::path::{Path, PathBuf};

use rusty_agentic_pack_manager::application::explorer::service::ExplorerService;
use rusty_agentic_pack_manager::application::pack::service::PackService;
use rusty_agentic_pack_manager::application::pack::traits::PackRepository;
use rusty_agentic_pack_manager::domain::pack::value_objects::TargetCli;
use rusty_agentic_pack_manager::infrastructure::filesystem::explorer::{
    FsExplorer, FsInstallDestination,
};
use rusty_agentic_pack_manager::infrastructure::pack::index::FsInstallIndexRepository;
use rusty_agentic_pack_manager::infrastructure::pack::loader::FsPackRepository;
use tempfile::TempDir;

fn make_pack(packs_dir: &Path) {
    let pack_dir = packs_dir.join("tdd");
    fs::create_dir_all(pack_dir.join("skills/tdd")).unwrap();
    fs::create_dir_all(pack_dir.join("agents")).unwrap();
    fs::write(
        pack_dir.join("skills/tdd/SKILL.md"),
        "# TDD skill\nWrite the test first.",
    )
    .unwrap();
    fs::write(
        pack_dir.join("agents/tdd-orchestrator.toml"),
        "name = 'tdd'",
    )
    .unwrap();
    fs::write(
        pack_dir.join("pack.toml"),
        r#"
name = "tdd"
version = "1.0.0"
description = "TDD pack"
cli = ["vibe"]

[[items]]
kind = "skill"
source = "skills/tdd"
[items.destinations]
vibe = ".agents/skills/tdd"

[[items]]
kind = "agent"
source = "agents/tdd-orchestrator.toml"
[items.destinations]
vibe = ".vibe/agents/tdd-orchestrator.toml"
"#,
    )
    .unwrap();
}

#[test]
fn real_pack_installs_into_target_repo() {
    let tmp = TempDir::new().unwrap();
    let packs_dir = tmp.path().join("packs");
    fs::create_dir_all(&packs_dir).unwrap();
    make_pack(&packs_dir);

    let repo = FsPackRepository::new(vec![packs_dir.clone()]).unwrap();
    let destination = FsInstallDestination::new(vec![packs_dir.clone()]);
    let service = PackService::new(
        Box::new(repo),
        Box::new(destination),
        Box::new(FsInstallIndexRepository),
    );

    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();

    let result = service
        .install("tdd", TargetCli::Vibe, target.to_str().unwrap(), false)
        .expect("install must succeed");
    assert_eq!(result.copied_count, 2);

    assert!(target.join(".agents/skills/tdd/SKILL.md").is_file());
    assert!(target.join(".vibe/agents/tdd-orchestrator.toml").is_file());

    // Re-install without overwrite fails on collision.
    let err = service
        .install("tdd", TargetCli::Vibe, target.to_str().unwrap(), false)
        .expect_err("collision must be detected");
    assert!(err.to_string().contains("already exists"));

    // With overwrite it succeeds.
    service
        .install("tdd", TargetCli::Vibe, target.to_str().unwrap(), true)
        .expect("overwrite install must succeed");
}

#[test]
fn install_records_the_index_and_uninstall_removes_everything() {
    let tmp = TempDir::new().unwrap();
    let packs_dir = tmp.path().join("packs");
    fs::create_dir_all(&packs_dir).unwrap();
    make_pack(&packs_dir);

    let repo = FsPackRepository::new(vec![packs_dir.clone()]).unwrap();
    let destination = FsInstallDestination::new(vec![packs_dir.clone()]);
    let service = PackService::new(
        Box::new(repo),
        Box::new(destination),
        Box::new(FsInstallIndexRepository),
    );

    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();
    let root = target.to_str().unwrap();

    // No index before the first install.
    assert!(service.installed_in(root).unwrap().is_empty());

    service
        .install("tdd", TargetCli::Vibe, root, false)
        .expect("install must succeed");

    // The index records the (pack, cli) entry with the relative paths.
    let installed = service.installed_in(root).unwrap();
    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0].name, "tdd");
    assert_eq!(installed[0].cli, TargetCli::Vibe);
    assert_eq!(
        installed[0].paths,
        vec![
            PathBuf::from(".agents/skills/tdd"),
            PathBuf::from(".vibe/agents/tdd-orchestrator.toml")
        ]
    );
    let raw = fs::read_to_string(target.join("packs_index.toml")).unwrap();
    assert!(raw.contains("name = \"tdd\""));
    assert!(raw.contains("cli = \"vibe\""));

    // Reinstall with overwrite: still a single entry (upsert).
    service
        .install("tdd", TargetCli::Vibe, root, true)
        .expect("overwrite install must succeed");
    assert_eq!(service.installed_in(root).unwrap().len(), 1);

    // Uninstall removes every listed asset and drops the index entry.
    let result = service
        .uninstall("tdd", TargetCli::Vibe, root)
        .expect("uninstall must succeed");
    assert_eq!(result.removed_count, 2);
    assert!(!target.join(".agents/skills/tdd").exists());
    assert!(!target.join(".vibe/agents/tdd-orchestrator.toml").exists());
    assert!(service.installed_in(root).unwrap().is_empty());

    // Uninstalling again is reported, not a crash.
    let err = service
        .uninstall("tdd", TargetCli::Vibe, root)
        .expect_err("second uninstall must be reported");
    assert!(err.to_string().contains("no install"));
}

#[test]
fn failed_install_writes_no_index() {
    let tmp = TempDir::new().unwrap();
    let packs_dir = tmp.path().join("packs");
    fs::create_dir_all(&packs_dir).unwrap();
    make_pack(&packs_dir);

    let repo = FsPackRepository::new(vec![packs_dir.clone()]).unwrap();
    let destination = FsInstallDestination::new(vec![packs_dir.clone()]);
    let service = PackService::new(
        Box::new(repo),
        Box::new(destination),
        Box::new(FsInstallIndexRepository),
    );

    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();
    let root = target.to_str().unwrap();

    // First install, then a second one without overwrite aborts on collision.
    service
        .install("tdd", TargetCli::Vibe, root, false)
        .unwrap();
    let err = service
        .install("tdd", TargetCli::Vibe, root, false)
        .expect_err("collision must abort");
    assert!(err.to_string().contains("already exists"));

    // The aborted install changed nothing in the index.
    let installed = service.installed_in(root).unwrap();
    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0].paths.len(), 2);
}

#[test]
fn explorer_lists_created_dir() {
    let tmp = TempDir::new().unwrap();
    let explorer = ExplorerService::new(Box::new(FsExplorer));
    explorer
        .create_dir(tmp.path().to_str().unwrap(), "sub")
        .unwrap();
    let listing = explorer.list(tmp.path().to_str().unwrap()).unwrap();
    assert!(listing.entries.iter().any(|e| e.name == "sub" && e.is_dir));
}

#[test]
fn missing_pack_is_reported() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("packs")).unwrap();
    let repo = FsPackRepository::new(vec![tmp.path().join("packs")]).unwrap();
    let err = repo.find("ghost").unwrap_err();
    assert!(matches!(
        err,
        rusty_agentic_pack_manager::application::error::ApplicationError::PackNotFound(_)
    ));
}

#[test]
fn all_shipped_packs_load_and_install() {
    let tmp = TempDir::new().unwrap();
    let packs_dir = PathBuf::from("packs");
    let repo = FsPackRepository::new(vec![packs_dir.clone()]).unwrap();
    let packs = repo.list().unwrap();
    assert!(
        packs.len() >= 16,
        "expected 16 shipped packs, got {}",
        packs.len()
    );

    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();
    let destination = FsInstallDestination::new(vec![packs_dir]);
    let service = PackService::new(
        Box::new(repo),
        Box::new(destination),
        Box::new(FsInstallIndexRepository),
    );

    for cli in [TargetCli::Vibe, TargetCli::Gemini] {
        let target = tmp.path().join(format!("{cli}"));
        fs::create_dir_all(&target).unwrap();
        for pack in &packs {
            if !pack.supports(cli) {
                continue;
            }
            let result = service
                .install(pack.name.as_str(), cli, target.to_str().unwrap(), false)
                .unwrap_or_else(|e| panic!("install {}/{} failed: {e}", pack.name, cli));
            assert!(result.copied_count > 0, "{} copied nothing", pack.name);
        }
    }

    let vibe_target = tmp.path().join("vibe");
    assert!(vibe_target.join(".agents/skills/tdd/SKILL.md").is_file());
    assert!(vibe_target
        .join(".vibe/agents/tdd-orchestrator.toml")
        .is_file());
    let gemini_target = tmp.path().join("gemini");
    assert!(gemini_target.join(".gemini/skills/tdd/SKILL.md").is_file());
    assert!(gemini_target.join(".gemini/commands/tdd.toml").is_file());

    // Multi-file skills (scripts included) install as whole directories.
    assert!(vibe_target
        .join(".agents/skills/vibe-wise-learn/behavior.md")
        .is_file());
    assert!(vibe_target
        .join(".agents/skills/vibe-wise-reset/scripts/reset.py")
        .is_file());
    // The Gemini-only commands pack namespaces under vibe-wise/.
    assert!(gemini_target
        .join(".gemini/commands/vibe-wise/learn.toml")
        .is_file());
    assert!(gemini_target
        .join(".gemini/commands/vibe-wise/reset.toml")
        .is_file());
    // The vibe-wise-commands pack does not target Vibe.
    let commands_pack = service.find("vibe-wise-commands").unwrap();
    assert!(commands_pack.supports(TargetCli::Gemini));
    assert!(!commands_pack.supports(TargetCli::Vibe));
}
