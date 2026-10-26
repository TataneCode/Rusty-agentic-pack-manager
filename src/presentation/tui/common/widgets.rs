use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

/// A simple selectable list widget with selection state (rusty-console
/// common/list pattern, synchronous version).
pub struct SelectList<T: Clone> {
    pub items: Vec<T>,
    pub state: ListState,
}

impl<T: Clone> SelectList<T> {
    pub fn new(items: Vec<T>) -> SelectList<T> {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        SelectList { items, state }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn selected(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let next = self
            .state
            .selected()
            .map(|i| (i + 1).min(self.items.len() - 1));
        self.state.select(next.or(Some(0)));
    }

    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let prev = self.state.selected().map(|i| i.saturating_sub(1));
        self.state.select(prev.or(Some(0)));
    }
}

/// Render a text list with per-item styling.
pub fn render_lines(
    frame: &mut Frame,
    area: Rect,
    items: Vec<Line<'static>>,
    state: &mut ListState,
) {
    let list_items: Vec<ListItem> = items.into_iter().map(ListItem::new).collect();
    let list = List::new(list_items).highlight_style(super::theme::Theme::highlight_style());
    frame.render_stateful_widget(list, area, state);
}

/// Render plain wrapped text.
pub fn render_text(frame: &mut Frame, area: Rect, text: &str, style: Style) {
    let p = Paragraph::new(text).wrap(Wrap { trim: true }).style(style);
    frame.render_widget(p, area);
}

/// Single-line text input (for folder creation dialog).
pub struct TextInput {
    pub value: String,
    pub cursor: usize,
}

impl TextInput {
    pub fn new() -> TextInput {
        TextInput {
            value: String::new(),
            cursor: 0,
        }
    }

    pub fn input(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.value.remove(self.cursor);
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, title: &str) {
        let block = super::theme::Theme::block(title, true);
        let inner_width = block.inner(area).width.saturating_sub(1) as usize; // keep one column for the cursor
                                                                              // Sliding window over the end of the value: the typed text and the
                                                                              // cursor stay visible even when the value is longer than the field.
        let visible: String = if self.value.chars().count() >= inner_width {
            self.value
                .chars()
                .skip(self.value.chars().count() - inner_width)
                .collect()
        } else {
            self.value.clone()
        };
        let line = Line::from(vec![
            Span::raw(visible),
            Span::styled("_", super::theme::Theme::header_style()),
        ]);
        let p = Paragraph::new(line).block(block);
        frame.render_widget(p, area);
    }
}

impl Default for TextInput {
    fn default() -> Self {
        TextInput::new()
    }
}
