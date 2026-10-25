//! Integration: a real pack from disk installs into a temp target repo.
//! No TUI, no Docker-style daemon: pure filesystem, wired exactly like
//! main.rs.

use std::fs;
use std::path::Path;

use rusty_agentic_pack_manager::application::explorer::service::ExplorerService;
use rusty_agentic_pack_manager::application::pack::service::PackService;
use rusty_agentic_pack_manager::application::pack::traits::PackRepository;
use rusty_agentic_pack_manager::domain::pack::value_objects::TargetCli;
use rusty_agentic_pack_manager::infrastructure::filesystem::explorer::{
    FsExplorer, FsInstallDestination,
};
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

    let repo = FsPackRepository::new(packs_dir.clone()).unwrap();
    let destination = FsInstallDestination::new(packs_dir.clone());
    let service = PackService::new(Box::new(repo), Box::new(destination));

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
    let repo = FsPackRepository::new(tmp.path().join("packs")).unwrap();
    let err = repo.find("ghost").unwrap_err();
    assert!(matches!(
        err,
        rusty_agentic_pack_manager::application::error::ApplicationError::PackNotFound(_)
    ));
}
