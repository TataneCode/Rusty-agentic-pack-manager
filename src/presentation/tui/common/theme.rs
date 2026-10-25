use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
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

    pub fn action_key_style() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn action_label_style() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn running_style() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn stopped_style() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn paused_style() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn in_use_style() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn dangling_style() -> Style {
        Style::default().fg(Color::Red)
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

    /// Shared rounded box for the layout sections (header, panels, action
    /// bar) so borders stay homogeneous across screens (rusty-console).
    pub fn section_block(title: Option<&str>) -> Block<'static> {
        let mut block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Self::focused_border_style())
            .padding(Padding::horizontal(1));
        if let Some(title) = title {
            block = block.title(Line::from(title.to_string()).style(Self::title_style()));
        }
        block
    }
}
