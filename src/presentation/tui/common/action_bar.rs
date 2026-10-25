use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use super::keys::AppAction;
use super::theme::Theme;

/// Bottom action bar: shows the contextual keys with highlighted letters
/// (pattern rusty-console action_bar).
pub fn key_span(key: &str, label: &str) -> Span<'static> {
    Span::styled(
        format!("[{key}] {label}  "),
        Style::default()
            .fg(ratatui::style::Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
}

pub fn label_span(label: &str) -> Span<'static> {
    Span::styled(
        format!("{label}  "),
        Style::default().fg(ratatui::style::Color::DarkGray),
    )
}

pub fn packs_panel_actions() -> Vec<Span<'static>> {
    vec![
        key_span("Tab", "Explorer"),
        key_span("A", "Apply"),
        key_span("V", "View"),
        key_span("Space", "Help"),
        key_span("Ctrl-C", "Quit"),
    ]
}

pub fn explorer_panel_actions() -> Vec<Span<'static>> {
    vec![
        key_span("Tab", "Packs"),
        key_span("R", "New folder"),
        key_span("Space", "Help"),
        key_span("Ctrl-C", "Quit"),
    ]
}

pub fn install_mode_actions() -> Vec<Span<'static>> {
    vec![
        key_span("Enter", "Open"),
        key_span("Bksp", "Up"),
        key_span("I", "Install here"),
        key_span("Esc", "Cancel"),
        key_span("Space", "Help"),
    ]
}

pub fn viewer_actions() -> Vec<Span<'static>> {
    vec![
        key_span("Tab", "Files/Content"),
        key_span("↑↓", "Scroll"),
        key_span("Esc", "Back"),
    ]
}

pub fn render(frame: &mut Frame, area: Rect, spans: Vec<Span<'static>>) {
    let line = Line::from(spans);
    let bar = Paragraph::new(line).style(Theme::default_style());
    frame.render_widget(bar, area);
}

/// Full-width modal popup (help, errors).
pub fn render_popup(frame: &mut Frame, area: Rect, title: &str, lines: Vec<Line<'static>>) {
    let block = Theme::block(title, true);
    let popup = Paragraph::new(lines)
        .block(block)
        .style(Theme::default_style());
    let popup_area = centered_rect(area, 60, 60);
    let _ = Clear;
    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

pub fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = ratatui::layout::Layout::vertical([
        ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ratatui::layout::Constraint::Percentage(percent_y),
        ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    ratatui::layout::Layout::horizontal([
        ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ratatui::layout::Constraint::Percentage(percent_x),
        ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

/// Escape helper for action imports in widgets module.
pub fn action_label(action: AppAction) -> &'static str {
    match action {
        AppAction::Quit => "Quit",
        AppAction::ToggleFocus => "Switch panel",
        AppAction::NavigateUp => "Up",
        AppAction::NavigateDown => "Down",
        AppAction::Enter => "Open",
        AppAction::Back => "Parent",
        AppAction::Apply => "Apply pack",
        AppAction::Install => "Install here",
        AppAction::View => "View pack",
        AppAction::CreateFolder => "New folder",
        AppAction::Help => "Help",
        AppAction::Close => "Close",
    }
}
