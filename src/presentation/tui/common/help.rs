use ratatui::text::Line;

use super::theme::Theme;

/// Keys shared by every panel: focus moves and direct access.
fn focus_lines(next: &str, prev: &str) -> Vec<Line<'static>> {
    vec![
        help_line("Tab / Shift+Tab", &format!("{next} / {prev}")),
        help_line("Shift+P/I/E", "Direct access: packs / installed / explorer"),
    ]
}

/// Keys closing the help popup and quitting, shared by every context.
fn common_tail_lines() -> Vec<Line<'static>> {
    vec![
        help_line("Space / Esc", "Close this help"),
        help_line("Ctrl+C", "Quit"),
    ]
}

/// Contextual help content per focus/mode (spacebar).
pub fn packs_help() -> Vec<Line<'static>> {
    let mut lines = focus_lines(
        "Switch to the installed panel",
        "Switch to the explorer panel",
    );
    lines.extend([
        help_line("\u{2191}/\u{2193} or k/j", "Move in the pack list"),
        help_line("c", "Collapse/expand the CLI group of the selected row"),
        help_line(
            "a",
            "Install the selected pack in the explorer's current directory",
        ),
        help_line("p", "Preview the install plan"),
        help_line("v", "View the pack files with syntax highlighting"),
    ]);
    lines.extend(common_tail_lines());
    lines
}

pub fn installed_help() -> Vec<Line<'static>> {
    let mut lines = focus_lines("Switch to the explorer panel", "Switch to the packs panel");
    lines.extend([
        help_line("\u{2191}/\u{2193} or k/j", "Move in the installed list"),
        help_line(
            "u",
            "Uninstall the selected pack (removes its files, confirm)",
        ),
    ]);
    lines.extend(common_tail_lines());
    lines
}

pub fn explorer_help() -> Vec<Line<'static>> {
    let mut lines = focus_lines("Switch to the packs panel", "Switch to the installed panel");
    lines.extend([
        help_line("\u{2191}/\u{2193} or k/j", "Move in the directory list"),
        help_line("Enter", "Open the selected directory"),
        help_line("Backspace", "Go to the parent directory"),
        help_line("r", "Create a folder here"),
        help_line("m", "Rename the selected folder/file"),
        help_line("d", "Delete the selected folder/file (confirm)"),
        help_line("Shift+D", "Force delete (non-empty folder, no confirm)"),
    ]);
    lines.extend(common_tail_lines());
    lines
}

pub fn viewer_help() -> Vec<Line<'static>> {
    let mut lines = vec![
        help_line("Tab / Shift+Tab", "Switch between file list and content"),
        help_line("\u{2191}/\u{2193} or k/j", "Scroll / change file"),
        help_line("Enter", "Show the selected file's content"),
        help_line("Backspace", "Back to the file list"),
        help_line("Esc", "Close the viewer"),
    ];
    lines.extend(common_tail_lines());
    lines
}

fn help_line(key: &str, label: &str) -> Line<'static> {
    Line::from(vec![
        ratatui::text::Span::styled(format!("{key:<24}"), Theme::header_style()),
        ratatui::text::Span::raw(label.to_string()),
    ])
}
