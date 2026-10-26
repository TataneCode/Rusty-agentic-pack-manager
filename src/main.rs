use std::path::PathBuf;

use anyhow::{Context, Result};

use rusty_agentic_pack_manager::application::explorer::service::ExplorerService;
use rusty_agentic_pack_manager::application::pack::service::PackService;
use rusty_agentic_pack_manager::application::pack_view::service::PackViewService;
use rusty_agentic_pack_manager::infrastructure::filesystem::explorer::{
    FsExplorer, FsInstallDestination,
};
use rusty_agentic_pack_manager::infrastructure::pack::content::FsPackContentRepository;
use rusty_agentic_pack_manager::infrastructure::pack::loader::FsPackRepository;
use rusty_agentic_pack_manager::presentation::tui::app::App;
use rusty_agentic_pack_manager::presentation::tui::event::EventHandler;
use rusty_agentic_pack_manager::presentation::tui::split_view;

/// Discover the embedded packs directory: next to the binary, then the
/// system location (D3).
fn find_packs_dir() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()?
        .parent()
        .map(PathBuf::from)?
        .join("packs");
    if exe_dir.is_dir() {
        return Some(exe_dir);
    }
    let cwd_packs = PathBuf::from("packs");
    if cwd_packs.is_dir() {
        return Some(cwd_packs);
    }
    let system = PathBuf::from("/usr/share/rusty-agentic-pack-manager/packs");
    if system.is_dir() {
        return Some(system);
    }
    None
}

fn build_app(start_dir: PathBuf, packs_dir: PathBuf) -> Result<App> {
    let pack_repository = FsPackRepository::new(packs_dir.clone())
        .with_context(|| format!("packs dir unreadable: {}", packs_dir.display()))?;
    let destination = FsInstallDestination::new(packs_dir.clone());
    let pack_service = PackService::new(Box::new(pack_repository), Box::new(destination));
    let explorer = ExplorerService::new(Box::new(FsExplorer));
    let view_service = PackViewService::new(Box::new(
        FsPackContentRepository::new(packs_dir).context("pack content repo")?,
    ));
    Ok(App::new(pack_service, explorer, view_service, start_dir))
}

fn main() -> Result<()> {
    let start_dir = std::env::current_dir().context("cannot read current directory")?;
    let Some(packs_dir) = find_packs_dir() else {
        anyhow::bail!("no packs directory found (expected ./packs next to the binary, ./packs in cwd, or /usr/share/rusty-agentic-pack-manager/packs)");
    };

    // ratatui::init(): raw mode + alternate screen + panic hook. Paired with
    // ratatui::restore() in the exit path below (review finding: the terminal
    // was never set up, making every keypress line-buffered).
    let mut terminal = ratatui::init();
    let events = EventHandler::default();

    let result = (|| -> Result<()> {
        let mut app = build_app(start_dir, packs_dir)?;
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
