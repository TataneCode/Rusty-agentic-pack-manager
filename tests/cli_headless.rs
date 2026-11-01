//! Integration (E03): the headless CLI — `rapm list|show|install|uninstall` —
//! driven through the real binary (`CARGO_BIN_EXE_rapm`) on a tempdir, with
//! the environment isolated (HOME/XDG_DATA_HOME on the tempdir) so the user
//! library and the discovered `./packs`/`./clis` roots are all local. No TTY.

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
fn list_shows_available_packs_without_a_terminal() {
    let (tmp, _target) = workspace();
    let output = rapm(tmp.path(), &["list"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("tdd"), "stdout: {out}");
    assert!(out.contains("1.0.0"), "stdout: {out}");
    assert!(out.contains("vibe"), "stdout: {out}");
}

#[test]
fn list_cli_filters_by_cli() {
    let (tmp, _target) = workspace();
    let output = rapm(tmp.path(), &["list", "--cli", "vibe"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("tdd"));
    let output = rapm(tmp.path(), &["list", "--cli", "gemini"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(!stdout(&output).contains("tdd"));
}

#[test]
fn show_prints_the_manifest_summary() {
    let (tmp, _target) = workspace();
    let output = rapm(tmp.path(), &["show", "tdd"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("name: tdd"), "stdout: {out}");
    assert!(out.contains("version: 1.0.0"), "stdout: {out}");
    assert!(out.contains("description: TDD pack"), "stdout: {out}");
    assert!(out.contains("clis: vibe"), "stdout: {out}");
    assert!(out.contains("skill"), "stdout: {out}");
    assert!(out.contains("skills/tdd"), "stdout: {out}");
    assert!(out.contains("vibe=.agents/skills/tdd"), "stdout: {out}");
}

#[test]
fn show_unknown_pack_exits_1() {
    let (tmp, _target) = workspace();
    let output = rapm(tmp.path(), &["show", "ghost"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("ghost"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn install_copies_files_writes_the_index_and_exits_0() {
    let (tmp, target) = workspace();
    let dest = target.to_str().unwrap();
    let output = rapm(
        tmp.path(),
        &["install", "tdd", "--dest", dest, "--cli", "vibe"],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(
        target.join(".agents/skills/tdd/SKILL.md").is_file(),
        "skill file must be copied"
    );
    let index = fs::read_to_string(target.join("packs_index.toml")).unwrap();
    assert!(index.contains("tdd"), "index: {index}");
    assert!(index.contains(".agents/skills/tdd"), "index: {index}");

    // Re-install without --force: collision, exit 1.
    let output = rapm(
        tmp.path(),
        &["install", "tdd", "--dest", dest, "--cli", "vibe"],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("already exists"),
        "stderr: {}",
        stderr(&output)
    );

    // --force reinstalls, exit 0.
    let output = rapm(
        tmp.path(),
        &["install", "tdd", "--dest", dest, "--cli", "vibe", "--force"],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
}

#[test]
fn install_defaults_cli_to_the_packs_first_declared_cli() {
    let (tmp, target) = workspace();
    let output = rapm(
        tmp.path(),
        &["install", "tdd", "--dest", target.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(target.join(".agents/skills/tdd/SKILL.md").is_file());
}

#[test]
fn list_installed_reads_the_index_of_the_target() {
    let (tmp, target) = workspace();
    let dest = target.to_str().unwrap();
    let output = rapm(tmp.path(), &["install", "tdd", "--dest", dest]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let output = rapm(tmp.path(), &["list", "--installed", "--dest", dest]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("tdd"), "stdout: {out}");
    assert!(out.contains("vibe"), "stdout: {out}");
    assert!(out.contains(".agents/skills/tdd"), "stdout: {out}");
}

#[test]
fn uninstall_removes_the_recorded_assets_and_the_index_entry() {
    let (tmp, target) = workspace();
    let dest = target.to_str().unwrap();
    let output = rapm(tmp.path(), &["install", "tdd", "--dest", dest]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let output = rapm(tmp.path(), &["uninstall", "tdd", "--dest", dest]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(!target.join(".agents/skills/tdd/SKILL.md").exists());
    let index = fs::read_to_string(target.join("packs_index.toml")).unwrap();
    assert!(!index.contains("tdd"), "index: {index}");

    // Uninstalling again: no recorded install, exit 1.
    let output = rapm(tmp.path(), &["uninstall", "tdd", "--dest", dest]);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn usage_errors_exit_2() {
    let (tmp, _target) = workspace();
    for args in [
        vec!["frobnicate"],
        vec!["show"],
        vec!["install"],
        vec!["list", "--wat"],
        vec!["install", "tdd", "--dest"],
    ] {
        let output = rapm(tmp.path(), &args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "args {args:?}: stderr: {}",
            stderr(&output)
        );
    }
}
