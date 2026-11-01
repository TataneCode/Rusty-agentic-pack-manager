//! Integration (E02): CLI profiles drive install destinations.
//!
//! Acceptance criteria from `.evolutions/e02-profils-cli.md`:
//! 1. a pack declared for ailed CLI with a profile but no enum recompiles
//!    installs (here: `claude`, unknown to the v1 enum);
//! 2. a user profile overrides the shipped profile of the same name;
//! 3. the 16 existing packs still install unmodified (loop over the real
//!    `packs/*/pack.toml` + real `clis/`).

use std::fs;
use std::path::{Path, PathBuf};

use rusty_agentic_pack_manager::application::cli_profile::service::CliProfileService;
use rusty_agentic_pack_manager::application::cli_profile::traits::CliProfileRepository;
use rusty_agentic_pack_manager::application::pack::service::PackService;
use rusty_agentic_pack_manager::application::pack::traits::{
    InstallIndexRepository, PackRepository,
};
use rusty_agentic_pack_manager::application::pack_view::traits::PackContentRepository;
use rusty_agentic_pack_manager::domain::cli_profile::value_objects::CliId;
use rusty_agentic_pack_manager::domain::pack::error::plan_install;
use rusty_agentic_pack_manager::infrastructure::cli_profile::loader::FsCliProfileRepository;
use rusty_agentic_pack_manager::infrastructure::filesystem::explorer::FsInstallDestination;
use rusty_agentic_pack_manager::infrastructure::pack::content::FsPackContentRepository;
use rusty_agentic_pack_manager::infrastructure::pack::index::FsInstallIndexRepository;
use rusty_agentic_pack_manager::infrastructure::pack::loader::FsPackRepository;
use tempfile::TempDir;

const CLAUDE_PROFILE: &str = r#"
[cli]
name = "claude"
label = "Claude Code"
[defaults]
skill     = ".claude/skills/{name}/SKILL.md"
skill_dir = ".claude/skills/{name}/"
agent     = ".claude/agents/{name}.md"
command   = ".claude/commands/{name}.md"
"#;

const CLAUDE_PROFILE_OVERRIDE: &str = r#"
[cli]
name = "claude"
label = "My Claude"
[defaults]
skill     = ".custom-claude/skills/{name}/SKILL.md"
skill_dir = ".custom-claude/skills/{name}/"
agent     = ".custom-claude/agents/{name}.md"
command   = ".custom-claude/commands/{name}.md"
"#;

fn write_profile(root: &Path, file: &str, content: &str) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join(file), content).unwrap();
}

/// A pack library + CLI profile roots + wired PackService, exactly like
/// main.rs, all on tempdirs.
fn wired(packs_root: &Path, profile_roots: Vec<PathBuf>) -> (FsCliProfileRepository, PackService) {
    let profiles = FsCliProfileRepository::new(profile_roots);
    let registry = profiles.registry().unwrap();
    let repository =
        FsPackRepository::with_profiles(vec![packs_root.to_path_buf()], registry).unwrap();
    let content = FsPackContentRepository::new(vec![packs_root.to_path_buf()]).unwrap();
    let service = PackService::new(
        Box::new(repository),
        Box::new(FsInstallDestination::new(vec![packs_root.to_path_buf()])),
        Box::new(FsInstallIndexRepository),
    )
    .with_profiles(CliProfileService::new(
        Box::new(profiles.clone()),
        Box::new(content),
    ));
    (profiles, service)
}

/// A pack for `claude` with NO explicit destinations (profile-driven).
fn write_claude_pack(packs_root: &Path) {
    let dir = packs_root.join("demo");
    fs::create_dir_all(dir.join("skills/demo")).unwrap();
    fs::write(dir.join("skills/demo/SKILL.md"), "# Demo").unwrap();
    fs::write(
        dir.join("pack.toml"),
        r#"
name = "demo"
version = "1.0.0"
cli = ["claude"]

[[items]]
kind = "skill"
source = "skills/demo/SKILL.md"
"#,
    )
    .unwrap();
}

#[test]
fn pack_for_a_profiled_cli_without_explicit_destinations_installs() {
    let tmp = TempDir::new().unwrap();
    let packs = tmp.path().join("packs");
    let clis = tmp.path().join("clis");
    write_claude_pack(&packs);
    write_profile(&clis, "claude.toml", CLAUDE_PROFILE);

    let (_profiles, service) = wired(&packs, vec![clis]);
    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();

    let result = service
        .install("demo", "claude", target.to_str().unwrap(), false)
        .expect("a pack for a profiled CLI must install without enum recompiles");
    assert_eq!(result.copied_count, 1);
    assert!(target.join(".claude/skills/demo/SKILL.md").is_file());
    // The install index recorded the resolved path.
    let entries = FsInstallIndexRepository
        .load(target.to_str().unwrap())
        .unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].cli.as_str(), "claude");
    assert_eq!(
        entries[0].paths,
        vec![PathBuf::from(".claude/skills/demo/SKILL.md")]
    );
}

#[test]
fn user_profile_overrides_the_shipped_profile_of_same_name() {
    let tmp = TempDir::new().unwrap();
    let packs = tmp.path().join("packs");
    let user_clis = tmp.path().join("user/clis");
    let ship_clis = tmp.path().join("ship/clis");
    write_claude_pack(&packs);
    write_profile(&ship_clis, "claude.toml", CLAUDE_PROFILE);
    write_profile(&user_clis, "claude.toml", CLAUDE_PROFILE_OVERRIDE);

    // User library first, like PackLibrary::discover.
    let (profiles, service) = wired(&packs, vec![user_clis, ship_clis.clone()]);
    let profile = profiles.find(&CliId::new("claude").unwrap()).unwrap();
    assert_eq!(profile.label, "My Claude");

    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();
    service
        .install("demo", "claude", target.to_str().unwrap(), false)
        .unwrap();
    assert!(target.join(".custom-claude/skills/demo/SKILL.md").is_file());
    assert!(!target.join(".claude/skills/demo/SKILL.md").exists());

    // Without the user root, the shipped profile applies.
    let (_profiles2, service2) = wired(&packs, vec![ship_clis]);
    let target2 = tmp.path().join("repo2");
    fs::create_dir_all(&target2).unwrap();
    service2
        .install("demo", "claude", target2.to_str().unwrap(), false)
        .unwrap();
    assert!(target2.join(".claude/skills/demo/SKILL.md").is_file());
}

#[test]
fn a_cli_without_any_profile_is_rejected() {
    let tmp = TempDir::new().unwrap();
    let packs = tmp.path().join("packs");
    let clis = tmp.path().join("clis");
    write_claude_pack(&packs);
    // Only a vibe profile: "claude" has none.
    write_profile(&clis, "vibe.toml", include_str!("../clis/vibe.toml"));

    let (_profiles, service) = wired(&packs, vec![clis]);
    let target = tmp.path().join("repo");
    fs::create_dir_all(&target).unwrap();
    let err = service
        .install("demo", "claude", target.to_str().unwrap(), false)
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("no CLI profile loaded for: claude"),
        "unexpected error: {err}"
    );
}

/// Criterion 3: the 16 existing packs still install unmodified. Loads the
/// real `packs/*/pack.toml` with the real shipped `clis/` and runs the
/// domain install plan for every declared CLI.
#[test]
fn all_shipped_packs_plan_an_install_for_every_declared_cli() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let packs_root = manifest.join("packs");
    let clis_root = manifest.join("clis");

    let profiles = FsCliProfileRepository::new(vec![clis_root]);
    let registry = profiles.registry().unwrap();
    assert!(
        registry.len() >= 7,
        "expected at least 7 shipped profiles, got {}",
        registry.len()
    );
    let repository = FsPackRepository::with_profiles(vec![packs_root.clone()], registry).unwrap();
    let content = FsPackContentRepository::new(vec![packs_root.clone()]).unwrap();
    let resolver = CliProfileService::new(Box::new(profiles), Box::new(content));

    let packs = repository.list().unwrap();
    assert_eq!(packs.len(), 16, "the 16 shipped packs must load");

    let tmp = TempDir::new().unwrap();
    let target = tmp.path().join("repo");
    for pack in &packs {
        for cli in &pack.clis {
            let resolved = resolver
                .resolve_pack(pack, cli)
                .unwrap_or_else(|err| panic!("resolve {} for {}: {err}", pack.name, cli));
            let plan = plan_install(&resolved, cli, &target)
                .unwrap_or_else(|err| panic!("plan {} for {}: {err}", pack.name, cli));
            assert!(
                !plan.is_empty(),
                "pack {} for {} plans no copy",
                pack.name,
                cli
            );
        }
    }
}

/// The shipped profiles must render a valid destination for every item of
/// every shipped pack (explicit destinations in the manifests still win at
/// install time; some packs like vibe-wise-commands deliberately override
/// the default layout, which is the documented escape hatch).
#[test]
fn shipped_profiles_render_a_valid_destination_for_every_shipped_item() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let packs_root = manifest.join("packs");
    let clis_root = manifest.join("clis");

    let profiles = FsCliProfileRepository::new(vec![clis_root]);
    let registry = profiles.registry().unwrap();
    let repository = FsPackRepository::with_profiles(vec![packs_root.clone()], registry).unwrap();
    let content = FsPackContentRepository::new(vec![packs_root.clone()]).unwrap();

    let packs = repository.list().unwrap();
    let mut checked = 0;
    for pack in &packs {
        for item in &pack.items {
            for cli in &pack.clis {
                let profile = profiles.find(cli).unwrap();
                let is_dir = content
                    .source_is_dir(pack.name.as_str(), &item.source)
                    .unwrap();
                let rendered = profile.destination(item, is_dir).unwrap_or_else(|err| {
                    panic!(
                        "pack {} item {} for {}: invalid profile destination: {err}",
                        pack.name,
                        item.source.display(),
                        cli
                    )
                });
                assert!(
                    rusty_agentic_pack_manager::domain::pack::error::validate_item_destination(
                        &rendered
                    )
                    .is_ok(),
                    "rendered destination must be installable: {rendered}"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 0);
}
