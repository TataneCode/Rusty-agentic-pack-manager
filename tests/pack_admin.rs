//! Integration (E13): `rapm add <dir>` / `rapm remove <name>` through the
//! real binary on a tempdir. The base directory's `agents/ skills/ rules/
//! commands/` subdirectories become the pack items; the generated
//! `pack.toml` gets one destination per loaded CLI profile.

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

/// Workspace with the vibe profile discoverable from the cwd and an empty
/// user library under `xdg/` (`$XDG_DATA_HOME/rapm/packs`).
fn workspace() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let clis = tmp.path().join("clis");
    fs::create_dir_all(&clis).unwrap();
    fs::write(clis.join("vibe.toml"), VIBE_PROFILE).unwrap();
    fs::create_dir_all(tmp.path().join("xdg")).unwrap();
    tmp
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

fn user_library(tmp: &TempDir) -> std::path::PathBuf {
    tmp.path().join("xdg/rapm/packs")
}

/// A base pack directory with one entry per kind plus an unrelated file
/// and directory at the root (ignored).
fn base_pack(tmp: &TempDir) -> std::path::PathBuf {
    let base = tmp.path().join("base-pack");
    fs::create_dir_all(base.join("skills/tdd")).unwrap();
    fs::write(base.join("skills/tdd/SKILL.md"), "# TDD").unwrap();
    fs::create_dir_all(base.join("agents")).unwrap();
    fs::write(base.join("agents/debugger.toml"), "agent").unwrap();
    fs::create_dir_all(base.join("rules")).unwrap();
    fs::write(base.join("rules/no-panic.md"), "rule").unwrap();
    fs::create_dir_all(base.join("commands")).unwrap();
    fs::write(base.join("commands/review.toml"), "cmd").unwrap();
    fs::write(base.join("README.md"), "not an item").unwrap();
    // Hidden entries are never imported as items.
    fs::write(base.join("skills/.DS_Store"), "junk").unwrap();
    base
}

#[test]
fn add_imports_a_directory_as_a_pack_and_generates_the_manifest() {
    let tmp = workspace();
    let base = base_pack(&tmp);
    let output = rapm(tmp.path(), &["add", base.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));

    let pack_dir = user_library(&tmp).join("base-pack");
    // Content copied under the canonical <kind>s/<name> layout.
    assert!(pack_dir.join("skills/tdd/SKILL.md").is_file());
    assert!(pack_dir.join("agents/debugger.toml").is_file());
    assert!(pack_dir.join("rules/no-panic.md").is_file());
    assert!(pack_dir.join("commands/review.toml").is_file());
    // Root files that are not in a kind directory are not copied.
    assert!(!pack_dir.join("README.md").exists());
    // Hidden entries of kind directories are not imported either.
    assert!(!pack_dir.join("skills/.DS_Store").exists());

    // The generated pack.toml traces the whole content, with one
    // destination per loaded profile (here: vibe only).
    let manifest = fs::read_to_string(pack_dir.join("pack.toml")).unwrap();
    assert!(
        manifest.contains("name = \"base-pack\""),
        "manifest: {manifest}"
    );
    assert!(
        manifest.contains("cli = [\"vibe\"]"),
        "manifest: {manifest}"
    );
    assert!(
        manifest.contains("kind = \"skill\""),
        "manifest: {manifest}"
    );
    assert!(
        manifest.contains("source = \"skills/tdd\""),
        "manifest: {manifest}"
    );
    assert!(
        manifest.contains("vibe = \".agents/skills/tdd/\""),
        "manifest: {manifest}"
    );
    assert!(manifest.contains("kind = \"rule\""), "manifest: {manifest}");
    assert!(
        manifest.contains("source = \"rules/no-panic.md\""),
        "manifest: {manifest}"
    );
    assert!(
        manifest.contains("vibe = \".vibe/rules/no-panic.md\""),
        "manifest: {manifest}"
    );
    assert!(
        manifest.contains("source = \"commands/review.toml\""),
        "manifest: {manifest}"
    );
    assert!(
        manifest.contains("vibe = \".vibe/commands/review.toml\""),
        "manifest: {manifest}"
    );
}

#[test]
fn add_with_name_uses_the_explicit_name() {
    let tmp = workspace();
    let base = base_pack(&tmp);
    let output = rapm(
        tmp.path(),
        &["add", base.to_str().unwrap(), "--name", "custom-name"],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(user_library(&tmp).join("custom-name/pack.toml").is_file());
    assert!(!user_library(&tmp).join("base-pack").exists());
}

#[test]
fn add_of_an_existing_pack_without_force_exits_1() {
    let tmp = workspace();
    let base = base_pack(&tmp);
    let args = &["add", base.to_str().unwrap()];
    assert_eq!(rapm(tmp.path(), args).status.code(), Some(0));
    let output = rapm(tmp.path(), args);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("already exists"),
        "stderr: {}",
        stderr(&output)
    );
    // With --force the pack is replaced, exit 0.
    let mut forced = args.to_vec();
    forced.push("--force");
    let output = rapm(tmp.path(), &forced);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
}

#[test]
fn add_of_a_directory_without_kind_directories_exits_1() {
    let tmp = workspace();
    let base = tmp.path().join("empty-pack");
    fs::create_dir_all(&base).unwrap();
    fs::write(base.join("README.md"), "nothing here").unwrap();
    let output = rapm(tmp.path(), &["add", base.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("no pack content"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn add_of_a_missing_directory_exits_1() {
    let tmp = workspace();
    let output = rapm(
        tmp.path(),
        &["add", tmp.path().join("missing").to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn an_added_pack_installs_with_the_generated_destinations() {
    let tmp = workspace();
    let base = base_pack(&tmp);
    assert_eq!(
        rapm(tmp.path(), &["add", base.to_str().unwrap()])
            .status
            .code(),
        Some(0)
    );

    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();
    let output = rapm(
        tmp.path(),
        &["install", "base-pack", "--dest", target.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(target.join(".agents/skills/tdd/SKILL.md").is_file());
    assert!(target.join(".vibe/agents/debugger.toml").is_file());
    assert!(target.join(".vibe/rules/no-panic.md").is_file());
    assert!(target.join(".vibe/commands/review.toml").is_file());
}

#[test]
fn remove_deletes_the_pack_from_the_user_library() {
    let tmp = workspace();
    let base = base_pack(&tmp);
    assert_eq!(
        rapm(tmp.path(), &["add", base.to_str().unwrap()])
            .status
            .code(),
        Some(0)
    );
    let pack_dir = user_library(&tmp).join("base-pack");
    assert!(pack_dir.is_dir());

    let output = rapm(tmp.path(), &["remove", "base-pack"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(!pack_dir.exists());
    assert!(stdout(&output).contains("base-pack"));

    // Removing again: not in the user library, exit 1.
    let output = rapm(tmp.path(), &["remove", "base-pack"]);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn remove_of_a_system_only_pack_exits_1() {
    let tmp = workspace();
    // A pack discovered from ./packs (cwd) but not present in the user
    // library: removal must fail (system/cwd roots are read-only).
    let pack_dir = tmp.path().join("packs/tdd/skills/tdd");
    fs::create_dir_all(&pack_dir).unwrap();
    fs::write(pack_dir.join("SKILL.md"), "# TDD").unwrap();
    fs::write(
        tmp.path().join("packs/tdd/pack.toml"),
        r#"
name = "tdd"
version = "1.0.0"
cli = ["vibe"]
[[items]]
kind = "skill"
source = "skills/tdd"
[items.destinations]
vibe = ".agents/skills/tdd"
"#,
    )
    .unwrap();

    let output = rapm(tmp.path(), &["remove", "tdd"]);
    assert_eq!(output.status.code(), Some(1));
    // The cwd pack is untouched.
    assert!(tmp.path().join("packs/tdd/pack.toml").is_file());
}

#[test]
fn remove_rejects_an_invalid_name_with_exit_1() {
    let tmp = workspace();
    // The name passes parsing (positional) but fails `PackName` validation
    // in the service: a runtime error, exit 1 (usage errors are exit 2).
    let output = rapm(tmp.path(), &["remove", "../escape"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("invalid pack name"),
        "stderr: {}",
        stderr(&output)
    );
}
