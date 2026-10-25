use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Padding},
};

/// Design system taken from rusty-console.
pub struct Theme;

impl Theme {
    pub fn default_style() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn highlight_style() -> Style {
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_style() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    pub fn title_style() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn focused_border_style() -> ratatui::style::Style {
        ratatui::style::Style::default().fg(Color::Cyan)
    }

    pub fn unfocused_border_style() -> ratatui::style::Style {
        ratatui::style::Style::default().fg(Color::DarkGray)
    }

    pub fn dir_style() -> Style {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    }

    pub fn file_style() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn help_style() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn error_style() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn block(title: &str, focused: bool) -> Block<'static> {
        let border = if focused {
            Self::focused_border_style()
        } else {
            Self::unfocused_border_style()
        };
        Block::bordered()
            .title(format!(" {title} "))
            .title_style(Self::title_style())
            .border_type(BorderType::Rounded)
            .border_style(border)
            .padding(Padding::new(1, 1, 0, 0))
    }
}
