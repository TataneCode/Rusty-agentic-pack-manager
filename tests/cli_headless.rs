//! Integration (E11): the headless CLI — `rapm install|uninstall|help` —
//! driven through the real binary (`CARGO_BIN_EXE_rapm`) on a tempdir, with
//! the environment isolated (HOME/XDG_DATA_HOME on the tempdir) so the user
//! library and the discovered `./packs`/`./clis` roots are all local. No TTY.
//! `add`/`remove` are covered by `tests/pack_admin.rs`, the config file by
//! `tests/config_cli.rs`.

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
rule      = ".vibe/rules/{name}.md"
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
fn help_exits_0_and_lists_the_fundamental_commands() {
    let (tmp, _target) = workspace();
    let output = rapm(tmp.path(), &["help"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    let out = stdout(&output);
    for cmd in [
        "install <pack>",
        "uninstall <pack>",
        "add <dir>",
        "remove <name>",
    ] {
        assert!(out.contains(cmd), "stdout missing {cmd}: {out}");
    }
}

#[test]
fn removed_commands_and_flags_exit_2() {
    let (tmp, target) = workspace();
    let dest = target.to_str().unwrap();
    for args in [
        vec!["list"],
        vec!["show", "tdd"],
        vec!["pack", "create", "--name", "x"],
        vec!["frobnicate"],
        vec!["install", "tdd", "--cli", "vibe", "--dest", dest],
        vec!["install", "tdd", "--dry-run", "--dest", dest],
        vec!["install"],
        vec!["install", "tdd", "--dest"],
        vec!["uninstall", "tdd", "--force"],
        vec!["remove"],
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

#[test]
fn install_copies_files_writes_the_index_and_exits_0() {
    let (tmp, target) = workspace();
    let dest = target.to_str().unwrap();
    let output = rapm(tmp.path(), &["install", "tdd", "--dest", dest]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(
        target.join(".agents/skills/tdd/SKILL.md").is_file(),
        "skill file must be copied"
    );
    let index = fs::read_to_string(target.join("packs_index.toml")).unwrap();
    assert!(index.contains("tdd"), "index: {index}");
    assert!(index.contains(".agents/skills/tdd"), "index: {index}");

    // Re-install without --force: collision, exit 1.
    let output = rapm(tmp.path(), &["install", "tdd", "--dest", dest]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("already exists"),
        "stderr: {}",
        stderr(&output)
    );

    // --force reinstalls, exit 0.
    let output = rapm(tmp.path(), &["install", "tdd", "--dest", dest, "--force"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
}

#[test]
fn install_defaults_to_the_packs_first_declared_cli_and_dest_to_cwd() {
    let (tmp, _target) = workspace();
    // No --dest: installs into the cwd (the workspace), which holds packs/.
    let output = rapm(tmp.path(), &["install", "tdd"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(tmp.path().join(".agents/skills/tdd/SKILL.md").is_file());
}

#[test]
fn read_only_commands_do_not_create_the_user_library() {
    let (tmp, target) = workspace();
    let user_lib = tmp.path().join("xdg/rapm/packs");
    assert!(!user_lib.exists());
    for args in [
        vec!["install", "tdd", "--dest", target.to_str().unwrap()],
        vec!["uninstall", "tdd", "--dest", target.to_str().unwrap()],
        vec!["help"],
    ] {
        let output = rapm(tmp.path(), &args);
        assert_ne!(output.status.code(), Some(2), "args {args:?}");
    }
    // install/uninstall/help are read-only w.r.t. the user library: the
    // directory is created only by `add`.
    assert!(!user_lib.exists(), "user library must not be created");
}

#[test]
fn install_unknown_pack_exits_1() {
    let (tmp, target) = workspace();
    let output = rapm(
        tmp.path(),
        &["install", "ghost", "--dest", target.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("ghost"),
        "stderr: {}",
        stderr(&output)
    );
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
