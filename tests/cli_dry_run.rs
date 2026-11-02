//! Integration (E04): `rapm install <pack> --dry-run` — the install plan is
//! printed without any write (no file, no `packs_index.toml`), exit 0, or
//! exit 3 when a destination already exists without `--force` (status
//! `collision` when tracked by the index, `hors index` otherwise). Driven
//! through the real binary (`CARGO_BIN_EXE_rapm`) on a tempdir, environment
//! isolated like `cli_headless.rs`. No TTY.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const VIBE_PROFILE: &str = r#"
[cli]
name = "vibe"
label = "Vibe CLI"

[defaults]
skill     = ".agents/skills/{name}/SKILL.md"
skill_dir = ".agents/skills/{name}/"
agent     = ".vibe/agents/{name}.toml"
command   = ".vibe/commands/{name}.toml"
"#;

const TDD_MANIFEST: &str = r#"
name = "tdd"
version = "1.0.0"
description = "TDD pack"
cli = ["vibe"]

[[items]]
kind = "skill"
source = "skills/tdd"
[items.destinations]
vibe = ".agents/skills/tdd"
"#;

/// A workspace tempdir acting as cwd: it holds the pack library (`packs/`)
/// and the CLI profiles (`clis/`) discovered from the cwd, plus an empty
/// user library under `xdg/`. Returns (tempdir, target repo dir).
fn workspace() -> (TempDir, std::path::PathBuf) {
    let tmp = TempDir::new().unwrap();
    let pack_dir = tmp.path().join("packs/tdd/skills/tdd");
    fs::create_dir_all(&pack_dir).unwrap();
    fs::write(pack_dir.join("SKILL.md"), "# TDD skill\n").unwrap();
    fs::write(tmp.path().join("packs/tdd/pack.toml"), TDD_MANIFEST).unwrap();
    let clis = tmp.path().join("clis");
    fs::create_dir_all(&clis).unwrap();
    fs::write(clis.join("vibe.toml"), VIBE_PROFILE).unwrap();
    fs::create_dir_all(tmp.path().join("xdg")).unwrap();
    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();
    (tmp, target)
}

fn rapm(workspace: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rapm"))
        .args(args)
        .current_dir(workspace)
        .env("HOME", workspace.join("home"))
        .env("XDG_DATA_HOME", workspace.join("xdg"))
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("failed to spawn rapm")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn dry_run_prints_the_plan_and_writes_nothing() {
    let (tmp, target) = workspace();
    let dest = target.to_str().unwrap();
    let output = rapm(tmp.path(), &["install", "tdd", "--dest", dest, "--dry-run"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("tdd"), "stdout: {out}");
    assert!(out.contains("vibe"), "stdout: {out}");
    assert!(out.contains("ok"), "stdout: {out}");
    assert!(out.contains("skills/tdd"), "stdout: {out}");
    assert!(out.contains(".agents/skills/tdd"), "stdout: {out}");

    // Acceptance criterion: no file copied, no index written.
    assert!(!target.join(".agents").exists(), "dry-run must not copy");
    assert!(
        !target.join("packs_index.toml").exists(),
        "dry-run must not write the index"
    );
}

#[test]
fn dry_run_marks_an_untracked_existing_destination_hors_index_and_exits_3() {
    let (tmp, target) = workspace();
    let dest = target.to_str().unwrap();
    // A user file already sits at the destination, tracked by no index entry.
    fs::create_dir_all(target.join(".agents/skills/tdd")).unwrap();
    fs::write(target.join(".agents/skills/tdd/SKILL.md"), "mine\n").unwrap();

    let output = rapm(tmp.path(), &["install", "tdd", "--dest", dest, "--dry-run"]);
    assert_eq!(output.status.code(), Some(3), "stdout: {}", stdout(&output));
    let out = stdout(&output);
    assert!(out.contains("hors index"), "stdout: {out}");
    assert!(
        !target.join("packs_index.toml").exists(),
        "dry-run must not write the index"
    );
    // The user file is untouched.
    assert_eq!(
        fs::read_to_string(target.join(".agents/skills/tdd/SKILL.md")).unwrap(),
        "mine\n"
    );
}

#[test]
fn dry_run_marks_an_indexed_destination_collision_and_exits_3() {
    let (tmp, target) = workspace();
    let dest = target.to_str().unwrap();
    // Real install first: the destination becomes tracked by the index.
    let output = rapm(tmp.path(), &["install", "tdd", "--dest", dest]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let output = rapm(tmp.path(), &["install", "tdd", "--dest", dest, "--dry-run"]);
    assert_eq!(output.status.code(), Some(3), "stdout: {}", stdout(&output));
    assert!(stdout(&output).contains("collision"), "stdout: {}", stdout(&output));
}

#[test]
fn dry_run_with_force_exits_0_and_keeps_the_statuses() {
    let (tmp, target) = workspace();
    let dest = target.to_str().unwrap();
    let output = rapm(tmp.path(), &["install", "tdd", "--dest", dest]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let output = rapm(
        tmp.path(),
        &["install", "tdd", "--dest", dest, "--dry-run", "--force"],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(
        stdout(&output).contains("collision"),
        "stdout: {}",
        stdout(&output)
    );
    // Still nothing new written by the dry-run itself.
    assert!(target.join("packs_index.toml").is_file());
}

#[test]
fn dry_run_on_an_unknown_pack_exits_1() {
    let (tmp, target) = workspace();
    let output = rapm(
        tmp.path(),
        &["install", "ghost", "--dest", target.to_str().unwrap(), "--dry-run"],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("ghost"), "stderr: {}", stderr(&output));
}

#[test]
fn uninstall_rejects_dry_run_as_a_usage_error() {
    let (tmp, target) = workspace();
    let output = rapm(
        tmp.path(),
        &["uninstall", "tdd", "--dest", target.to_str().unwrap(), "--dry-run"],
    );
    assert_eq!(output.status.code(), Some(2));
}
