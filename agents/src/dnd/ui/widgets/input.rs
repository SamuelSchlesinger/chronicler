//! Input field widget

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::dnd::ui::theme::GameTheme;

/// Input field widget
pub struct InputWidget<'a> {
    content: &'a str,
    cursor_position: usize,
    theme: &'a GameTheme,
    placeholder: &'a str,
}

impl<'a> InputWidget<'a> {
    pub fn new(content: &'a str, theme: &'a GameTheme) -> Self {
        Self {
            content,
            cursor_position: content.chars().count(),
            theme,
            placeholder: "Enter your action...",
        }
    }

    pub fn cursor_position(mut self, pos: usize) -> Self {
        self.cursor_position = pos;
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }
}

impl Widget for InputWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border_style(true));

        let inner = block.inner(area);
        block.render(area, buf);

        // Build the input line with cursor
        let line = if self.content.is_empty() {
            Line::from(vec![
                Span::styled("> ", self.theme.player_style()),
                Span::styled(
                    self.placeholder,
                    Style::default().add_modifier(Modifier::DIM),
                ),
            ])
        } else {
            // Use char_indices for UTF-8 safe slicing
            let char_indices: Vec<(usize, char)> = self.content.char_indices().collect();
            let cursor_pos = self.cursor_position.min(char_indices.len());

            // Find byte boundaries for slicing
            let before_byte_end = if cursor_pos < char_indices.len() {
                char_indices[cursor_pos].0
            } else {
                self.content.len()
            };

            let before_cursor = &self.content[..before_byte_end];

            let (at_cursor, after_cursor) = if cursor_pos < char_indices.len() {
                let cursor_char = char_indices[cursor_pos].1.to_string();
                let after_byte_start = if cursor_pos + 1 < char_indices.len() {
                    char_indices[cursor_pos + 1].0
                } else {
                    self.content.len()
                };
                (cursor_char, &self.content[after_byte_start..])
            } else {
                (" ".to_string(), "")
            };

            Line::from(vec![
                Span::styled("> ", self.theme.player_style()),
                Span::raw(before_cursor),
                Span::styled(
                    at_cursor,
                    Style::default()
                        .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
                        .fg(self.theme.player_text),
                ),
                Span::raw(after_cursor),
            ])
        };

        let paragraph = Paragraph::new(line);
        paragraph.render(inner, buf);
    }
}
