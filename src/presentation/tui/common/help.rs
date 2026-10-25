use ratatui::text::Line;

use super::theme::Theme;

/// Contextual help content per focus/mode (spacebar).
pub fn packs_help() -> Vec<Line<'static>> {
    vec![
        help_line("Tab", "Switch to the explorer panel"),
        help_line("↑/↓ or k/j", "Move in the pack list"),
        help_line("Enter", "Expand pack details"),
        help_line("A", "Apply the selected pack (choose target dir)"),
        help_line("V", "View the pack files with syntax highlighting"),
        help_line("Ctrl-C", "Quit"),
    ]
}

pub fn explorer_help() -> Vec<Line<'static>> {
    vec![
        help_line("Tab", "Switch to the packs panel"),
        help_line("↑/↓ or k/j", "Move in the directory list"),
        help_line("Enter", "Open the selected directory"),
        help_line("Backspace", "Go to the parent directory"),
        help_line("R", "Create a folder here"),
        help_line("Ctrl-C", "Quit"),
    ]
}

pub fn install_help() -> Vec<Line<'static>> {
    vec![
        help_line("↑/↓ or k/j", "Move in the directory list"),
        help_line("Enter", "Open the selected directory"),
        help_line("Backspace", "Go to the parent directory"),
        help_line("I", "Install the pack in this directory"),
        help_line("Esc", "Cancel the installation"),
    ]
}

pub fn viewer_help() -> Vec<Line<'static>> {
    vec![
        help_line("Tab", "Switch between file list and content"),
        help_line("↑/↓, PgUp/PgDn, Home/End", "Scroll"),
        help_line("Backspace", "Back to the file list"),
        help_line("Esc", "Close the viewer"),
    ]
}

fn help_line(key: &str, label: &str) -> Line<'static> {
    Line::from(vec![
        ratatui::text::Span::styled(format!("{key:<24}"), Theme::header_style()),
        ratatui::text::Span::raw(label.to_string()),
    ])
}
