use std::path::PathBuf;

use anyhow::{Context, Result};

use rusty_agentic_pack_manager::application::explorer::service::ExplorerService;
use rusty_agentic_pack_manager::application::pack::service::PackService;
use rusty_agentic_pack_manager::application::pack_create::service::PackCreationService;
use rusty_agentic_pack_manager::application::pack_view::service::PackViewService;
use rusty_agentic_pack_manager::infrastructure::filesystem::explorer::{
    FsExplorer, FsInstallDestination,
};
use rusty_agentic_pack_manager::infrastructure::pack::content::FsPackContentRepository;
use rusty_agentic_pack_manager::infrastructure::pack::index::FsInstallIndexRepository;
use rusty_agentic_pack_manager::infrastructure::pack::loader::FsPackRepository;
use rusty_agentic_pack_manager::infrastructure::pack::writer::FsPackWriter;
use rusty_agentic_pack_manager::presentation::cli;
use rusty_agentic_pack_manager::presentation::library::PackLibrary;
use rusty_agentic_pack_manager::presentation::tui::app::App;
use rusty_agentic_pack_manager::presentation::tui::event::EventHandler;
use rusty_agentic_pack_manager::presentation::tui::split_view;

fn build_app(start_dir: PathBuf, library: &PackLibrary) -> Result<App> {
    let pack_repository = FsPackRepository::new(library.roots.clone())
        .with_context(|| format!("pack library unreadable: {:?}", library.roots))?;
    let destination = FsInstallDestination::new(library.roots.clone());
    let pack_service = PackService::new(
        Box::new(pack_repository),
        Box::new(destination),
        Box::new(FsInstallIndexRepository),
    );
    let explorer = ExplorerService::new(Box::new(FsExplorer));
    let view_service = PackViewService::new(Box::new(
        FsPackContentRepository::new(library.roots.clone()).context("pack content repo")?,
    ));
    Ok(App::new(pack_service, explorer, view_service, start_dir))
}

fn build_creation_service(library: &PackLibrary) -> Result<PackCreationService> {
    library.ensure_user_root().with_context(|| {
        format!(
            "cannot create user pack library: {}",
            library.user_root.display()
        )
    })?;
    let repository = FsPackRepository::new(library.roots.clone())
        .with_context(|| format!("pack library unreadable: {:?}", library.roots))?;
    let writer = FsPackWriter::new(library.user_root.clone());
    Ok(PackCreationService::new(
        Box::new(repository),
        Box::new(writer),
    ))
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let library = PackLibrary::discover().ok_or_else(|| {
        anyhow::anyhow!(
            "no pack library found (expected the user library under the XDG data dir, ./packs next to the binary, ./packs in cwd, or /usr/share/rusty-agentic-pack-manager/packs)"
        )
    })?;

    // The CLI runs before any terminal setup: `rapm <subcommand>` never
    // enters raw mode, `rapm` alone launches the TUI.
    if cli::is_cli_invocation(&args) {
        let service = build_creation_service(&library)?;
        match cli::run(&args, &service) {
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
    // TUI also starts on a fresh machine (idempotent create_dir_all).
    library.ensure_user_root().with_context(|| {
        format!(
            "cannot create user pack library: {}",
            library.user_root.display()
        )
    })?;

    let start_dir = std::env::current_dir().context("cannot read current directory")?;

    // ratatui::init(): raw mode + alternate screen + panic hook. Paired with
    // ratatui::restore() in the exit path below (review finding: the terminal
    // was never set up, making every keypress line-buffered).
    let mut terminal = ratatui::init();
    let events = EventHandler::default();

    let result = (|| -> Result<()> {
        let mut app = build_app(start_dir, &library)?;
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
