use ratatui::{
    layout::{Constraint, Layout},
    text::{Line, Span},
    Frame,
};

use crate::presentation::tui::app::App;
use crate::presentation::tui::common::action_bar;
use crate::presentation::tui::common::theme::Theme;
use crate::presentation::tui::common::widgets;

/// Viewer screen: pack file list + file content (pattern rusty-console
/// logs screen).
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let chunks = Layout::vertical([Constraint::Min(3), Constraint::Length(1)]).split(area);

    let Some(viewer) = &app.viewer else {
        return;
    };

    let show_content = viewer.show_content && !viewer.lines.is_empty();
    let panels = if show_content {
        Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(chunks[0])
    } else {
        Layout::horizontal([Constraint::Percentage(100)]).split(chunks[0])
    };

    let title = format!("Pack: {}", viewer.pack);
    let block = Theme::block(&title, true);
    let inner = block.inner(panels[0]);
    frame.render_widget(block, panels[0]);

    let mut lines = Vec::new();
    for path in viewer.files.items.clone() {
        let style = if Some(&path) == viewer.files.selected() {
            Theme::highlight_style()
        } else {
            Theme::file_style()
        };
        lines.push(Line::from(Span::styled(format!(" {path}"), style)));
    }
    let mut state = viewer.files.state;
    widgets::render_lines(frame, inner, lines, &mut state);
    let scroll_snapshot: Vec<Vec<Span>> = viewer.lines.clone();
    let scroll_pos = viewer.content_scroll;

    if show_content {
        let content_title = viewer.files.selected().cloned().unwrap_or_default();
        let block = Theme::block(&content_title, true);
        let inner = block.inner(panels[1]);
        frame.render_widget(block, panels[1]);

        let visible: Vec<Line> = scroll_snapshot
            .iter()
            .skip(scroll_pos as usize)
            .map(|spans| Line::from(spans.clone()))
            .collect();
        let text = ratatui::text::Text::from(visible);
        let p = ratatui::widgets::Paragraph::new(text);
        frame.render_widget(p, inner);
    }

    action_bar::render_action_bar(frame, chunks[1], &action_bar::base_footer_text());
}
