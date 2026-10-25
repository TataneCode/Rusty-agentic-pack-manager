//! Integration UI: renders the split screen with ratatui TestBackend and
//! asserts on the flattened buffer text (pattern rusty-console UI tests).

use std::fs;

use ratatui::{backend::TestBackend, Terminal as RTerminal};

use rusty_agentic_pack_manager::application::explorer::service::ExplorerService;
use rusty_agentic_pack_manager::application::pack::service::PackService;
use rusty_agentic_pack_manager::application::pack_view::service::PackViewService;
use rusty_agentic_pack_manager::infrastructure::filesystem::explorer::{
    FsExplorer, FsInstallDestination,
};
use rusty_agentic_pack_manager::infrastructure::pack::content::FsPackContentRepository;
use rusty_agentic_pack_manager::infrastructure::pack::loader::FsPackRepository;
use rusty_agentic_pack_manager::presentation::tui::app::{App, Focus, Mode};
use rusty_agentic_pack_manager::presentation::tui::split_view;
use tempfile::TempDir;

fn make_pack(packs_dir: &std::path::Path) {
    let pack_dir = packs_dir.join("review");
    fs::create_dir_all(pack_dir.join("skills/code-review")).unwrap();
    fs::write(
        pack_dir.join("skills/code-review/SKILL.md"),
        "# Code review skill",
    )
    .unwrap();
    fs::write(
        pack_dir.join("pack.toml"),
        r#"
name = "review"
version = "1.0.0"
description = "Code review pack"
cli = ["vibe"]

[[items]]
kind = "skill"
source = "skills/code-review"
[items.destinations]
vibe = ".agents/skills/code-review"
"#,
    )
    .unwrap();
}

fn build_app(tmp: &TempDir) -> App {
    let packs_dir = tmp.path().join("packs");
    fs::create_dir_all(&packs_dir).unwrap();
    make_pack(&packs_dir);
    fs::create_dir_all(tmp.path().join("start/subdir")).unwrap();
    fs::write(tmp.path().join("start/file.txt"), "x").unwrap();

    let repo = FsPackRepository::new(packs_dir.clone()).unwrap();
    let destination = FsInstallDestination::new(packs_dir.clone());
    let pack_service = PackService::new(Box::new(repo), Box::new(destination));
    let explorer = ExplorerService::new(Box::new(FsExplorer));
    let view_service =
        PackViewService::new(Box::new(FsPackContentRepository::new(packs_dir).unwrap()));
    App::new(
        pack_service,
        explorer,
        view_service,
        tmp.path().join("start"),
    )
}

fn render_snapshot(app: &mut App) -> String {
    let backend = TestBackend::new(100, 24);
    let mut terminal = RTerminal::new(backend).unwrap();
    terminal.draw(|f| split_view::render(f, app)).unwrap();
    let mut out = String::new();
    let buffer = terminal.backend().buffer();
    let width = buffer.area().width as usize;
    for y in 0..buffer.area().height as usize {
        let row: String = buffer
            .content()
            .iter()
            .skip(y * width)
            .take(width)
            .map(|c| c.symbol().to_string())
            .collect();
        let trimmed = row.trim_end();
        out.push_str(trimmed);
        out.push('\n');
    }
    out
}

#[test]
fn split_screen_shows_both_panels_and_pack() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    let snapshot = render_snapshot(&mut app);
    assert!(snapshot.contains("Packs"), "packs panel title missing");
    assert!(
        snapshot.contains("Explorer"),
        "explorer panel title missing"
    );
    assert!(snapshot.contains("Vibe CLI"), "CLI group header missing");
    assert!(snapshot.contains("review"), "pack entry missing");
    assert!(snapshot.contains("subdir"), "directory entry missing");
}

#[test]
fn tab_toggles_focus() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    assert_eq!(app.focus, Focus::Packs);
    app.dispatch(&key(crossterm::event::KeyCode::Tab));
    assert_eq!(app.focus, Focus::Explorer);
    app.dispatch(&key(crossterm::event::KeyCode::Tab));
    assert_eq!(app.focus, Focus::Packs);
}

#[test]
fn apply_switches_to_install_mode_and_esc_cancels() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    app.dispatch(&key(crossterm::event::KeyCode::Char('a')));
    assert!(matches!(app.mode, Mode::Installing { .. }));
    assert_eq!(app.focus, Focus::Explorer);
    app.dispatch(&key(crossterm::event::KeyCode::Esc));
    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.focus, Focus::Packs);
}

#[test]
fn space_shows_contextual_help_and_closes() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    app.dispatch(&key(crossterm::event::KeyCode::Char(' ')));
    let snapshot = render_snapshot(&mut app);
    assert!(snapshot.contains("Help"), "help popup missing");
    assert!(snapshot.contains("Apply the selected pack"));
    app.dispatch(&key(crossterm::event::KeyCode::Char(' ')));
    assert!(app.popup.is_none());
}

#[test]
fn viewer_opens_with_v_and_lists_files() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    app.dispatch(&key(crossterm::event::KeyCode::Char('v')));
    assert!(app.viewer.is_some());
    assert!(app
        .viewer
        .as_ref()
        .unwrap()
        .files
        .items
        .iter()
        .any(|p| p.contains("SKILL.md")));
    app.dispatch(&key(crossterm::event::KeyCode::Esc));
    assert!(app.viewer.is_none());
}

fn key(code: crossterm::event::KeyCode) -> crossterm::event::KeyEvent {
    crossterm::event::KeyEvent::new(code, crossterm::event::KeyModifiers::NONE)
}
