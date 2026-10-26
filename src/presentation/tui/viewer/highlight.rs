use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

/// Syntax highlighting for pack files (.md, .js, .toml), mapped to the
/// terminal 16-color palette (theme rusty-console).
pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme: syntect::highlighting::Theme,
}

impl Highlighter {
    pub fn new() -> Highlighter {
        Highlighter {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme: ThemeSet::load_defaults().themes["base16-ocean.dark"].clone(),
        }
    }

    /// Map a syntect color to the terminal palette (approximation).
    fn map_color(color: syntect::highlighting::Color) -> Color {
        if color.a == 0 {
            return Color::White;
        }
        let (r, g, b) = (color.r, color.g, color.b);
        // Nearest of the 16 ANSI base colors by luminance/chroma heuristic.
        let palette = [
            (Color::Black, 0x000000),
            (Color::Red, 0xcd3131),
            (Color::Green, 0x0dbc79),
            (Color::Yellow, 0xe5e510),
            (Color::Blue, 0x3b8eea),
            (Color::Magenta, 0xbb88dd),
            (Color::Cyan, 0x14b3c8),
            (Color::White, 0xe5e5e5),
            (Color::Gray, 0x8a8a8a),
            (Color::LightRed, 0xff6e64),
            (Color::LightGreen, 0x7fca6f),
            (Color::LightYellow, 0xffe66d),
            (Color::LightBlue, 0x6fc2ff),
            (Color::LightMagenta, 0xff9ce5),
            (Color::LightCyan, 0x86e5f7),
            (Color::White, 0xffffff),
        ];
        let mut best = Color::White;
        let mut best_dist = u32::MAX;
        let (rr, rg2, rb) = (r as i32, g as i32, b as i32);
        for (rat_color, hex) in palette {
            let pr = ((hex >> 16) & 0xff) as u32;
            let pg = ((hex >> 8) & 0xff) as u32;
            let pb = (hex & 0xff) as u32;
            let (dr, dg, db) = (rr.unsigned_abs(), rg2.unsigned_abs(), rb.unsigned_abs());
            let dr = dr.abs_diff(pr);
            let dg = dg.abs_diff(pg);
            let db = db.abs_diff(pb);
            let dist = dr * dr + dg * dg + db * db;
            if dist < best_dist {
                best_dist = dist;
                best = rat_color;
            }
        }
        best
    }

    /// Highlight a whole file into ratatui spans per line.
    pub fn highlight(&self, path: &str, content: &str) -> Vec<Vec<Span<'static>>> {
        let syntax = self
            .syntax_set
            .find_syntax_for_file(path)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let mut hl = HighlightLines::new(syntax, &self.theme);

        let mut result = Vec::new();
        for line in syntect::util::LinesWithEndings::from(content) {
            let ranges = hl
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();
            let mut spans = Vec::new();
            for (style, text) in ranges {
                let mut rat_style = Style::default().fg(Self::map_color(style.foreground));
                if style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::BOLD)
                {
                    rat_style = rat_style.add_modifier(Modifier::BOLD);
                }
                if style
                    .font_style
                    .contains(syntect::highlighting::FontStyle::ITALIC)
                {
                    rat_style = rat_style.add_modifier(Modifier::ITALIC);
                }
                let text = text.trim_end_matches(['\n', '\r']);
                spans.push(Span::styled(text.to_string(), rat_style));
            }
            result.push(spans);
        }
        result
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Highlighter::new()
    }
}
