//! Integration (E12): the config file `$XDG_CONFIG_HOME/rapm/config.toml`
//! (`packs_dir`, `default_cli`) is honored by the headless CLI through the
//! real binary on a tempdir.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const VIBE_PROFILE: &str = r#"
[cli]
name = "vibe"
[defaults]
skill     = ".agents/skills/{name}/SKILL.md"
skill_dir = ".agents/skills/{name}/"
agent     = ".vibe/agents/{name}.toml"
rule      = ".vibe/rules/{name}.md"
command   = ".vibe/commands/{name}.toml"
"#;

const GEMINI_PROFILE: &str = r#"
[cli]
name = "gemini"
[defaults]
skill     = ".gemini/skills/{name}/SKILL.md"
skill_dir = ".gemini/skills/{name}/"
agent     = ".gemini/agents/{name}.toml"
rule      = ".gemini/rules/{name}.md"
command   = ".gemini/commands/{name}.toml"
"#;

const MULTI_MANIFEST: &str = r#"
name = "multi"
version = "1.0.0"
cli = ["vibe", "gemini"]

[[items]]
kind = "skill"
source = "skills/multi"
[items.destinations]
vibe = ".agents/skills/multi"
gemini = ".gemini/skills/multi"
"#;

/// Workspace with both profiles in ./clis and a `multi` pack in ./packs.
fn workspace() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let clis = tmp.path().join("clis");
    fs::create_dir_all(&clis).unwrap();
    fs::write(clis.join("vibe.toml"), VIBE_PROFILE).unwrap();
    fs::write(clis.join("gemini.toml"), GEMINI_PROFILE).unwrap();
    let pack_dir = tmp.path().join("packs/multi/skills/multi");
    fs::create_dir_all(&pack_dir).unwrap();
    fs::write(pack_dir.join("SKILL.md"), "# multi").unwrap();
    fs::write(tmp.path().join("packs/multi/pack.toml"), MULTI_MANIFEST).unwrap();
    fs::create_dir_all(tmp.path().join("xdg")).unwrap();
    fs::create_dir_all(tmp.path().join("xdg-config")).unwrap();
    tmp
}

fn rapm(workspace: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rapm"))
        .args(args)
        .current_dir(workspace)
        .env("HOME", workspace.join("home"))
        .env("XDG_DATA_HOME", workspace.join("xdg"))
        .env("XDG_CONFIG_HOME", workspace.join("xdg-config"))
        .output()
        .expect("failed to spawn rapm")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn write_config(tmp: &TempDir, content: &str) {
    let dir = tmp.path().join("xdg-config/rapm");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("config.toml"), content).unwrap();
}

#[test]
fn without_a_config_file_the_xdg_defaults_apply() {
    let tmp = workspace();
    // No config.toml written: `add` writes into the XDG user library.
    let base = tmp.path().join("my-pack");
    fs::create_dir_all(base.join("rules")).unwrap();
    fs::write(base.join("rules/x.md"), "rule").unwrap();
    let output = rapm(tmp.path(), &["add", base.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(tmp
        .path()
        .join("xdg/rapm/packs/my-pack/pack.toml")
        .is_file());
}

#[test]
fn packs_dir_relocates_the_writable_library() {
    let tmp = workspace();
    let custom = tmp.path().join("custom-packs");
    write_config(
        &tmp,
        &format!("packs_dir = {:?}\n", custom.to_str().unwrap()),
    );
    let base = tmp.path().join("my-pack");
    fs::create_dir_all(base.join("skills/tdd")).unwrap();
    fs::write(base.join("skills/tdd/SKILL.md"), "# TDD").unwrap();
    let output = rapm(tmp.path(), &["add", base.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(custom.join("my-pack/pack.toml").is_file());
    // The XDG default library was not used.
    assert!(!tmp.path().join("xdg/rapm/packs/my-pack").exists());

    // The relocated library is also a read root: the added pack installs.
    // No default_cli configured: the pack's first declared CLI wins — the
    // generated manifest lists the profiles in registry order, so gemini.
    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();
    let output = rapm(
        tmp.path(),
        &["install", "my-pack", "--dest", target.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(target.join(".gemini/skills/tdd/SKILL.md").is_file());
}

#[test]
fn default_cli_selects_the_install_target() {
    let tmp = workspace();
    write_config(&tmp, "default_cli = \"gemini\"\n");
    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();
    let output = rapm(
        tmp.path(),
        &["install", "multi", "--dest", target.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    // gemini destination, not the pack's first declared CLI (vibe).
    assert!(target.join(".gemini/skills/multi/SKILL.md").is_file());
    assert!(!target.join(".agents/skills/multi/SKILL.md").exists());
    assert!(
        stdout(&output).contains("gemini"),
        "stdout: {}",
        stdout(&output)
    );
}

#[test]
fn default_cli_not_supported_by_the_pack_falls_back_to_the_first_declared_cli() {
    let tmp = workspace();
    write_config(&tmp, "default_cli = \"gemini\"\n");
    // A vibe-only pack: gemini is not declared, so vibe (first declared) wins.
    let pack_dir = tmp.path().join("packs/vibe-only/skills/vo");
    fs::create_dir_all(&pack_dir).unwrap();
    fs::write(pack_dir.join("SKILL.md"), "# vo").unwrap();
    fs::write(
        tmp.path().join("packs/vibe-only/pack.toml"),
        r#"
name = "vibe-only"
version = "1.0.0"
cli = ["vibe"]
[[items]]
kind = "skill"
source = "skills/vo"
[items.destinations]
vibe = ".agents/skills/vo"
"#,
    )
    .unwrap();
    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();
    let output = rapm(
        tmp.path(),
        &["install", "vibe-only", "--dest", target.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert!(target.join(".agents/skills/vo/SKILL.md").is_file());
}

#[test]
fn a_relative_packs_dir_is_rejected_with_the_config_path() {
    let tmp = workspace();
    write_config(&tmp, "packs_dir = \"relative/packs\"\n");
    let output = rapm(tmp.path(), &["help"]);
    assert_eq!(output.status.code(), Some(1));
    let err = stderr(&output);
    assert!(err.contains("config.toml"), "stderr: {err}");
    assert!(err.contains("absolute"), "stderr: {err}");
}

#[test]
fn a_malformed_config_file_is_rejected_with_the_config_path() {
    let tmp = workspace();
    write_config(&tmp, "not = [toml");
    let output = rapm(tmp.path(), &["help"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("config.toml"),
        "stderr: {}",
        stderr(&output)
    );
}

#[test]
fn an_unknown_default_cli_profile_is_rejected() {
    let tmp = workspace();
    write_config(&tmp, "default_cli = \"no-such-cli\"\n");
    let output = rapm(tmp.path(), &["help"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("default_cli"),
        "stderr: {}",
        stderr(&output)
    );
}
