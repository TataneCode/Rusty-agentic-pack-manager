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
use rusty_agentic_pack_manager::presentation::tui::app::{App, Focus};
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
fn apply_installs_in_the_explorer_current_dir_and_refreshes() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    app.dispatch(&key(crossterm::event::KeyCode::Char('a')));
    assert!(matches!(
        app.popup,
        Some(rusty_agentic_pack_manager::presentation::tui::app::Popup::Success(_))
    ));
    // files installed under the explorer's current dir (start/)
    assert!(tmp
        .path()
        .join("start/.agents/skills/code-review/SKILL.md")
        .is_file());
    // explorer refreshed: the new .agents dir is listed
    assert!(app
        .dir_entries
        .items
        .iter()
        .any(|(name, is_dir)| name == ".agents" && *is_dir));
}

#[test]
fn space_shows_contextual_help_and_closes() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    app.dispatch(&key(crossterm::event::KeyCode::Char(' ')));
    let snapshot = render_snapshot(&mut app);
    assert!(snapshot.contains("Help"), "help popup missing");
    assert!(snapshot.contains("Install the selected pack"));
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

#[test]
fn rename_opens_prefilled_input_and_renames() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    fs::write(tmp.path().join("start/file.txt"), "x").unwrap();
    app.refresh_dir();
    app.focus = Focus::Explorer;

    // select file.txt (dirs first: subdir at 0, file.txt at 1)
    app.dir_entries.state.select(Some(1));
    app.dispatch(&key(crossterm::event::KeyCode::Char('m')));
    let (input, kind) = (app.input.as_ref().unwrap(), app.input_kind.clone());
    assert_eq!(input.value, "file.txt");
    assert!(matches!(
        kind,
        Some(rusty_agentic_pack_manager::presentation::tui::app::InputKind::Rename { .. })
    ));

    // type new name: clear with backspaces then type
    for _ in 0..8 {
        app.dispatch(&key(crossterm::event::KeyCode::Backspace));
    }
    for c in "renamed.txt".chars() {
        app.dispatch(&key(crossterm::event::KeyCode::Char(c)));
    }
    app.dispatch(&key(crossterm::event::KeyCode::Enter));
    assert!(tmp.path().join("start/renamed.txt").is_file());
    assert!(app.input.is_none() && app.input_kind.is_none());
}

#[test]
fn delete_shows_confirm_and_deletes_on_yes() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    fs::write(tmp.path().join("start/file.txt"), "x").unwrap();
    app.refresh_dir();
    app.focus = Focus::Explorer;

    app.dir_entries.state.select(Some(1));
    app.dispatch(&key(crossterm::event::KeyCode::Char('d')));
    assert!(app.confirm.is_some());

    // default is No; select Yes with ArrowUp then confirm
    app.dispatch(&key(crossterm::event::KeyCode::Up));
    app.dispatch(&key(crossterm::event::KeyCode::Enter));
    assert!(app.confirm.is_none());
    assert!(!tmp.path().join("start/file.txt").exists());
}

#[test]
fn delete_cancel_keeps_file() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    fs::write(tmp.path().join("start/file.txt"), "x").unwrap();
    app.refresh_dir();
    app.focus = Focus::Explorer;

    app.dir_entries.state.select(Some(1));
    app.dispatch(&key(crossterm::event::KeyCode::Char('d')));
    app.dispatch(&key(crossterm::event::KeyCode::Esc));
    assert!(app.confirm.is_none());
    assert!(tmp.path().join("start/file.txt").exists());
}

#[test]
fn force_delete_deletes_non_empty_folder_directly() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    fs::create_dir_all(tmp.path().join("start/subdir/inner")).unwrap();
    fs::write(tmp.path().join("start/subdir/inner/x.md"), "#").unwrap();
    app.refresh_dir();
    app.focus = Focus::Explorer;

    app.dir_entries.state.select(Some(0)); // subdir (dirs first)
    app.dispatch(&key(crossterm::event::KeyCode::Char('D')));
    assert!(app.confirm.is_none());
    assert!(!tmp.path().join("start/subdir").exists());
}

#[test]
fn ctrl_c_quits_from_any_mode() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);

    // from the packs panel
    app.dispatch(&ctrl('c'));
    assert!(app.quit);

    // from the explorer panel
    let mut app = build_app(&tmp);
    app.dispatch(&key(crossterm::event::KeyCode::Tab));
    app.dispatch(&ctrl('c'));
    assert!(app.quit);
}

fn ctrl(c: char) -> crossterm::event::KeyEvent {
    crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char(c),
        crossterm::event::KeyModifiers::CONTROL,
    )
}

#[test]
fn input_dialog_shows_typed_text_even_when_long() {
    let tmp = TempDir::new().unwrap();
    let mut app = build_app(&tmp);
    app.focus = Focus::Explorer;
    app.dispatch(&key(crossterm::event::KeyCode::Char('r')));

    let long_name = "a-very-long-folder-name-typed-by-the-user";
    for c in long_name.chars() {
        app.dispatch(&key(crossterm::event::KeyCode::Char(c)));
    }
    let snapshot = render_snapshot(&mut app);
    // the tail of the typed text and the cursor must be visible
    let tail: String = long_name.chars().skip(20).collect();
    assert!(
        snapshot.contains(&tail),
        "typed text tail should be visible in the input dialog"
    );
    assert!(snapshot.contains('_'), "cursor should be visible");
}
