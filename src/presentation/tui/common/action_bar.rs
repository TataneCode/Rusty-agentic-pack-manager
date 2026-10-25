use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use super::keys::AppAction;
use super::theme::Theme;

const SEPARATOR: &str = " | ";

/// Splits a help string (" Tab: Switch panel | Space: Help ") into styled
/// spans: keys in yellow bold, labels in white, separators dimmed
/// (rusty-console action_bar pattern).
pub fn action_bar_line(help_text: &str) -> Line<'static> {
    let mut spans = Vec::new();
    for (index, segment) in help_text.split('|').map(str::trim).enumerate() {
        if index > 0 {
            spans.push(Span::styled(SEPARATOR, Theme::help_style()));
        }
        match segment.split_once(':') {
            Some((key, label)) => {
                spans.push(Span::styled(key.to_string(), Theme::action_key_style()));
                spans.push(Span::styled(
                    format!(":{label}"),
                    Theme::action_label_style(),
                ));
            }
            None => spans.push(Span::styled(
                segment.to_string(),
                Theme::action_label_style(),
            )),
        }
    }
    Line::from(spans)
}

/// The two always-available commands (user decision: the contextual actions
/// live in the Space help, not in the footer).
pub fn base_footer_text() -> String {
    "Tab: Switch panel | Space: Help".to_string()
}

pub fn render_action_bar(frame: &mut Frame, area: Rect, help_text: &str) {
    let bar = Paragraph::new(action_bar_line(help_text)).block(Theme::section_block(None));
    frame.render_widget(bar, area);
}

pub fn render_popup(frame: &mut Frame, area: Rect, title: &str, lines: Vec<Line<'static>>) {
    let block = Theme::block(title, true);
    let popup = Paragraph::new(lines)
        .block(block)
        .style(Theme::default_style());
    let popup_area = centered_rect(area, 60, 60);
    frame.render_widget(Clear, popup_area);
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

/// Escape helper kept for potential future use of key spans outside the
/// parsed footer format.
pub fn action_label(action: AppAction) -> &'static str {
    match action {
        AppAction::Quit => "Quit",
        AppAction::ToggleFocus => "Switch panel",
        AppAction::NavigateUp => "Up",
        AppAction::NavigateDown => "Down",
        AppAction::Enter => "Open",
        AppAction::Back => "Parent",
        AppAction::Apply => "Apply pack",
        AppAction::View => "View pack",
        AppAction::CreateFolder => "New folder",
        AppAction::Rename => "Rename",
        AppAction::Delete => "Delete",
        AppAction::ForceDelete => "Force delete",
        AppAction::Help => "Help",
        AppAction::Close => "Close",
    }
}

/// Style helper re-exported for popup renderers that need a raw style.
pub fn error_style() -> Style {
    Theme::error_style()
}
