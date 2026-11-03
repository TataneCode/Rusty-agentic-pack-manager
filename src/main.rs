use std::path::PathBuf;

use anyhow::{Context, Result};

use rusty_agentic_pack_manager::application::cli_profile::service::CliProfileService;
use rusty_agentic_pack_manager::application::cli_profile::traits::CliProfileRepository;
use rusty_agentic_pack_manager::application::config::dto::AppConfigDto;
use rusty_agentic_pack_manager::application::config::service::ConfigService;
use rusty_agentic_pack_manager::application::explorer::service::ExplorerService;
use rusty_agentic_pack_manager::application::library::traits::LibraryInitializer;
use rusty_agentic_pack_manager::application::pack::service::PackService;
use rusty_agentic_pack_manager::application::pack_admin::service::PackAdminService;
use rusty_agentic_pack_manager::application::pack_view::service::PackViewService;
use rusty_agentic_pack_manager::infrastructure::cli_profile::loader::FsCliProfileRepository;
use rusty_agentic_pack_manager::infrastructure::config::fs_config::FsConfigRepository;
use rusty_agentic_pack_manager::infrastructure::filesystem::explorer::{
    FsExplorer, FsInstallDestination,
};
use rusty_agentic_pack_manager::infrastructure::filesystem::library::FsLibraryInitializer;
use rusty_agentic_pack_manager::infrastructure::pack::content::FsPackContentRepository;
use rusty_agentic_pack_manager::infrastructure::pack::index::FsInstallIndexRepository;
use rusty_agentic_pack_manager::infrastructure::pack::loader::FsPackRepository;
use rusty_agentic_pack_manager::infrastructure::pack::scan::FsBaseDirScanner;
use rusty_agentic_pack_manager::infrastructure::pack::writer::FsPackWriter;
use rusty_agentic_pack_manager::presentation::cli;
use rusty_agentic_pack_manager::presentation::library::PackLibrary;
use rusty_agentic_pack_manager::presentation::tui::app::App;
use rusty_agentic_pack_manager::presentation::tui::event::EventHandler;
use rusty_agentic_pack_manager::presentation::tui::split_view;

/// Load the user configuration file (E12). No config file (or no config
/// directory in the environment) means an empty config.
fn load_config() -> Result<AppConfigDto> {
    match FsConfigRepository::discover() {
        Some(repository) => Ok(ConfigService::new(Box::new(repository)).load()?),
        None => Ok(AppConfigDto::default()),
    }
}

fn build_app(
    start_dir: PathBuf,
    library: &PackLibrary,
    default_cli: Option<String>,
) -> Result<App> {
    let content =
        FsPackContentRepository::new(library.roots.clone()).context("pack content repo")?;
    let pack_service = build_pack_service(library, default_cli)?;
    let explorer = ExplorerService::new(Box::new(FsExplorer));
    let view_service = PackViewService::new(Box::new(content));
    Ok(App::new(pack_service, explorer, view_service, start_dir))
}

fn ensure_user_root(library: &PackLibrary) -> Result<()> {
    FsLibraryInitializer
        .ensure_user_root(&library.user_root)
        .with_context(|| {
            format!(
                "cannot create user pack library: {}",
                library.user_root.display()
            )
        })
}

/// The `PackAdminService` behind `rapm add` / `rapm remove` (E13): packs
/// are written to (and removed from) the writable root of the library.
fn build_admin_service(library: &PackLibrary) -> Result<PackAdminService> {
    ensure_user_root(library)?;
    let profiles = FsCliProfileRepository::new(library.profile_roots.clone());
    let registry = profiles.registry().context("CLI profiles unreadable")?;
    let repository = FsPackRepository::with_profiles(library.roots.clone(), registry.clone())
        .with_context(|| format!("pack library unreadable: {:?}", library.roots))?;
    let writer = FsPackWriter::new(library.user_root.clone());
    Ok(PackAdminService::new(
        Box::new(repository),
        Box::new(writer),
        Box::new(FsBaseDirScanner),
    )
    .with_profiles(registry))
}

/// The `PackService` used by both the TUI and the headless CLI (E03):
/// same adapters, same profile resolver. `default_cli` is the configured
/// default target CLI (E12).
fn build_pack_service(library: &PackLibrary, default_cli: Option<String>) -> Result<PackService> {
    let profiles = FsCliProfileRepository::new(library.profile_roots.clone());
    let registry = profiles.registry().context("CLI profiles unreadable")?;
    let pack_repository = FsPackRepository::with_profiles(library.roots.clone(), registry)
        .with_context(|| format!("pack library unreadable: {:?}", library.roots))?;
    let destination = FsInstallDestination::new(library.roots.clone());
    let content =
        FsPackContentRepository::new(library.roots.clone()).context("pack content repo")?;
    Ok(PackService::new(
        Box::new(pack_repository),
        Box::new(destination),
        Box::new(FsInstallIndexRepository),
    )
    .with_profiles(CliProfileService::new(
        Box::new(profiles),
        Box::new(content),
    ))
    .with_default_cli(default_cli))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = load_config()?;
    let library = PackLibrary::discover_with(&config).ok_or_else(|| {
        anyhow::anyhow!(
            "no pack library found (expected the user library under the XDG data dir, ./packs next to the binary, ./packs in cwd, or /usr/share/rusty-agentic-pack-manager/packs)"
        )
    })?;

    // A configured `default_cli` must name a loaded CLI profile (E12).
    let profiles = FsCliProfileRepository::new(library.profile_roots.clone());
    let registry = profiles.registry().context("CLI profiles unreadable")?;
    if let Some(cli) = &config.default_cli {
        registry
            .validate(cli)
            .with_context(|| format!("invalid default_cli in the config file: {cli}"))?;
    }

    // The CLI runs before any terminal setup: `rapm <subcommand>` never
    // enters raw mode, `rapm` alone launches the TUI.
    if cli::is_cli_invocation(&args) {
        let admin = build_admin_service(&library)?;
        let pack = build_pack_service(&library, config.default_cli.clone())?;
        match cli::run(&args, &admin, &pack) {
            Ok(code) => std::process::exit(code),
            Err(err) => {
                eprintln!("error: {err}");
                // Usage errors exit 2 (shell convention), runtime errors 1.
                let code = match err {
                    rusty_agentic_pack_manager::application::error::ApplicationError::Usage(_) => 2,
                    _ => 1,
                };
                std::process::exit(code);
            }
        }
    }

    // The user library is the first read root: make sure it exists so the
    // TUI also starts on a fresh machine (idempotent create_dir_all via the
    // LibraryInitializer port).
    ensure_user_root(&library)?;

    let start_dir = std::env::current_dir().context("cannot read current directory")?;

    // ratatui::init(): raw mode + alternate screen + panic hook. Paired with
    // ratatui::restore() in the exit path below (review finding: the terminal
    // was never set up, making every keypress line-buffered).
    let mut terminal = ratatui::init();
    let events = EventHandler::default();

    let result = (|| -> Result<()> {
        let mut app = build_app(start_dir, &library, config.default_cli.clone())?;
        loop {
            terminal.draw(|frame| split_view::render(frame, &mut app))?;
            match events.next()? {
                rusty_agentic_pack_manager::presentation::tui::event::AppEvent::Key(key) => {
                    app.dispatch(&key);
                }
                rusty_agentic_pack_manager::presentation::tui::event::AppEvent::Tick => {}
            }
            if app.quit {
                break;
            }
        }
        Ok(())
    })();

    ratatui::restore();
    result
}
