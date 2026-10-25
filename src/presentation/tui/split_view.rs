use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::presentation::tui::app::{App, Focus, Popup};
use crate::presentation::tui::common::action_bar;
use crate::presentation::tui::common::help;
use crate::presentation::tui::common::theme::Theme;
use crate::presentation::tui::common::widgets;

/// Main screen: split packs/explorer + action bar + popups.
pub fn render(frame: &mut Frame, app: &mut App) {
    if app.viewer.is_some() {
        crate::presentation::tui::viewer::view::render(frame, app);
        render_popups(frame, app);
        return;
    }

    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(3),
        Constraint::Length(3),
    ])
    .split(area);

    render_header(frame, chunks[0], app);

    let panels = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    render_packs_panel(frame, panels[0], app);
    render_explorer_panel(frame, panels[1], app);

    action_bar::render_action_bar(frame, chunks[2], &action_bar::base_footer_text());

    render_input(frame, app);
    render_confirm(frame, app);
    render_popups(frame, app);
}

/// Application title header (rusty-console layout: header / content /
/// action bar, all in shared section blocks).
fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let selected = app
        .selected_pack()
        .map(|p| format!(" — {} v{}", p.name, p.version))
        .unwrap_or_default();
    let title = format!(" Rusty Agentic Pack Manager{selected} ");
    let header = Paragraph::new(ratatui::text::Text::from(Span::styled(
        title,
        Theme::title_style(),
    )))
    .block(Theme::section_block(None));
    frame.render_widget(header, area);
}

fn render_packs_panel(frame: &mut Frame, area: Rect, app: &mut App) {
    let block = Theme::block("Packs", app.focus == Focus::Packs);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    let mut current_cli: Option<String> = None;
    for pack in app.packs.clone() {
        let cli_label = pack.clis.first().map(|c| c.label()).unwrap_or("Unknown");
        if current_cli.as_deref() != Some(cli_label) {
            current_cli = Some(cli_label.to_string());
            lines.push(Line::from(Span::styled(
                format!(" {cli_label} "),
                Theme::header_style(),
            )));
        }
        lines.push(Line::from(Span::raw(format!(
            " {} v{} — {} ({} items)",
            pack.name, pack.version, pack.description, pack.item_count
        ))));
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " No packs found",
            Theme::error_style(),
        )));
    }

    let mut state = app.pack_list.state;
    widgets::render_lines(frame, inner, lines, &mut state);
    app.pack_list.state = state;
}

fn render_explorer_panel(frame: &mut Frame, area: Rect, app: &mut App) {
    let focused = app.focus == Focus::Explorer;
    let title = format!("Explorer — {}", app.current_dir.display());
    let block = Theme::block(&title, focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    for (name, is_dir) in app.dir_entries.items.clone() {
        let span = if is_dir {
            Span::styled(format!(" {name}/"), Theme::dir_style())
        } else {
            Span::styled(format!(" {name}"), Theme::file_style())
        };
        lines.push(Line::from(span));
    }
    if lines.is_empty() {
        lines.push(Line::from(Span::raw(" (empty)")));
    }

    let mut state = app.dir_entries.state;
    widgets::render_lines(frame, inner, lines, &mut state);
    app.dir_entries.state = state;
}

fn render_input(frame: &mut Frame, app: &App) {
    let Some(input) = &app.input else {
        return;
    };
    let title = match &app.input_kind {
        Some(crate::presentation::tui::app::InputKind::Rename { .. }) => " New name ",
        _ => " New folder name ",
    };
    let area = action_bar::centered_rect(frame.area(), 50, 3);
    frame.render_widget(ratatui::widgets::Clear, area);
    input.render(frame, area, title);
}

fn render_confirm(frame: &mut Frame, app: &App) {
    use ratatui::text::Span;
    let Some(dialog) = &app.confirm else {
        return;
    };
    let yes = Span::styled(
        if dialog.selected_yes {
            " [Yes] "
        } else {
            "  Yes  "
        },
        Theme::highlight_style(),
    );
    let no = Span::styled(
        if dialog.selected_yes {
            "  No  "
        } else {
            " [No] "
        },
        if dialog.selected_yes {
            Theme::file_style()
        } else {
            Theme::highlight_style()
        },
    );
    let lines = vec![
        Line::from(Span::styled(
            format!(" {} ", dialog.message),
            Theme::error_style(),
        )),
        Line::from(vec![yes, no]),
        Line::from(Span::styled(
            " ←/→ select · Enter confirm · Esc cancel",
            Theme::file_style(),
        )),
    ];
    action_bar::render_popup(frame, frame.area(), " Confirm ", lines);
}

fn render_popups(frame: &mut Frame, app: &App) {
    match &app.popup {
        Some(Popup::Help) => {
            let lines = match (&app.viewer.is_some(), &app.focus) {
                (true, _) => help::viewer_help(),
                (false, Focus::Packs) => help::packs_help(),
                (false, Focus::Explorer) => help::explorer_help(),
            };
            action_bar::render_popup(frame, frame.area(), " Help ", lines);
        }
        Some(Popup::Error(message)) => {
            let lines = vec![Line::from(Span::styled(
                message.clone(),
                Theme::error_style(),
            ))];
            action_bar::render_popup(frame, frame.area(), " Error ", lines);
        }
        Some(Popup::Success(message)) => {
            let lines = vec![Line::from(Span::styled(
                message.clone(),
                Theme::help_style(),
            ))];
            action_bar::render_popup(frame, frame.area(), " Success ", lines);
        }
        None => {}
    }
}
