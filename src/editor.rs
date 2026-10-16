use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mcpv::error::{AppError, Result};
use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use serde_json::{Map, Value};

pub const MAX_ARGUMENT_BYTES: usize = 1024 * 1024;

pub struct Editor {
    pub tool: String,
    pub text: String,
    pub error: Option<Arc<AppError>>,
    cursor: usize,
    top: usize,
    left: usize,
}

impl Editor {
    pub fn new(tool: String, text: String) -> Self {
        Self {
            tool,
            text,
            error: None,
            cursor: 0,
            top: 0,
            left: 0,
        }
    }

    pub fn replace(&mut self, text: String) {
        self.text = text;
        self.cursor = 0;
        self.top = 0;
        self.left = 0;
        self.error = None;
    }

    pub fn arguments(&self) -> Result<Map<String, Value>> {
        if self.text.len() > MAX_ARGUMENT_BYTES {
            return Err(AppError::ArgumentSize);
        }
        match serde_json::from_str(&self.text).map_err(AppError::ArgumentJson)? {
            Value::Object(arguments) => Ok(arguments),
            _ => Err(AppError::ArgumentRoot),
        }
    }

    pub fn insert(&mut self, text: &str) {
        let text = text
            .replace("\r\n", "\n")
            .replace('\r', "\n")
            .replace('\t', "  ");
        let text: String = text
            .chars()
            .filter(|ch| *ch == '\n' || !ch.is_control())
            .collect();
        if self.text.len().saturating_add(text.len()) > MAX_ARGUMENT_BYTES {
            self.error = Some(Arc::new(AppError::ArgumentSize));
            return;
        }
        self.text.insert_str(self.cursor, &text);
        self.cursor += text.len();
        self.error = None;
    }

    fn line_start(&self) -> usize {
        self.text[..self.cursor]
            .rfind('\n')
            .map_or(0, |index| index + 1)
    }

    fn line_end(&self) -> usize {
        self.text[self.cursor..]
            .find('\n')
            .map_or(self.text.len(), |index| self.cursor + index)
    }

    pub fn key(&mut self, key: KeyEvent) {
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return;
        }
        match key.code {
            KeyCode::Char(ch) => self.insert(&ch.to_string()),
            KeyCode::Enter => self.insert("\n"),
            KeyCode::Tab => self.insert("  "),
            KeyCode::Left => {
                self.cursor = self.text[..self.cursor]
                    .char_indices()
                    .next_back()
                    .map_or(0, |(index, _)| index)
            }
            KeyCode::Right => {
                self.cursor += self.text[self.cursor..]
                    .chars()
                    .next()
                    .map_or(0, char::len_utf8)
            }
            KeyCode::Home => self.cursor = self.line_start(),
            KeyCode::End => self.cursor = self.line_end(),
            KeyCode::Up | KeyCode::Down => {
                let start = self.line_start();
                let column = self.text[start..self.cursor].chars().count();
                let range = if key.code == KeyCode::Up && start > 0 {
                    let end = start - 1;
                    Some((
                        self.text[..end].rfind('\n').map_or(0, |index| index + 1),
                        end,
                    ))
                } else if key.code == KeyCode::Down
                    && self.line_end() < self.text.len()
                {
                    let next = self.line_end() + 1;
                    Some((
                        next,
                        self.text[next..]
                            .find('\n')
                            .map_or(self.text.len(), |index| next + index),
                    ))
                } else {
                    None
                };
                if let Some((start, end)) = range {
                    self.cursor = start
                        + self.text[start..end]
                            .char_indices()
                            .nth(column)
                            .map_or(end - start, |(index, _)| index);
                }
            }
            KeyCode::Backspace if self.cursor > 0 => {
                let previous = self.text[..self.cursor]
                    .char_indices()
                    .next_back()
                    .map_or(0, |(index, _)| index);
                self.text.drain(previous..self.cursor);
                self.cursor = previous;
                self.error = None;
            }
            KeyCode::Delete if self.cursor < self.text.len() => {
                let length = self.text[self.cursor..]
                    .chars()
                    .next()
                    .map_or(0, char::len_utf8);
                self.text.drain(self.cursor..self.cursor + length);
                self.error = None;
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let border = Block::default()
            .borders(Borders::ALL)
            .title(format!("Arguments: {}", crate::ui::single_line(&self.tool)));
        let inner = border.inner(area);
        frame.render_widget(border, area);
        if inner.width == 0 || inner.height == 0 {
            return;
        }
        let row = self.text[..self.cursor]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        let column =
            Line::raw(&self.text[self.line_start()..self.cursor]).width();
        let width = usize::from(inner.width);
        let height = usize::from(inner.height);
        let line_width =
            Line::raw(&self.text[self.line_start()..self.line_end()]).width();
        self.left = self
            .left
            .min(line_width.saturating_add(1).saturating_sub(width));
        self.top = self
            .top
            .min(row)
            .max(row.saturating_add(1).saturating_sub(height));
        self.left = self
            .left
            .min(column)
            .max(column.saturating_add(1).saturating_sub(width));
        let lines: Vec<Line<'_>> = self
            .text
            .split('\n')
            .skip(self.top)
            .take(height)
            .map(|line| Line::raw(crop(line, self.left, width)))
            .collect();
        frame.render_widget(Paragraph::new(lines), inner);
        frame.set_cursor_position((
            inner.x + (column - self.left) as u16,
            inner.y + (row - self.top) as u16,
        ));
    }
}

fn crop(line: &str, left: usize, width: usize) -> String {
    let mut position: usize = 0;
    let mut text = String::new();
    for ch in line.chars() {
        let cells = Line::raw(ch.to_string()).width();
        let end = position.saturating_add(cells);
        if end > left && position < left.saturating_add(width) {
            if position < left || end > left.saturating_add(width) {
                text.push(' ');
            } else {
                text.push(ch);
            }
        } else if cells == 0
            && position > left
            && position <= left.saturating_add(width)
        {
            text.push(ch);
        }
        position = end;
        if position > left.saturating_add(width) {
            break;
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn unicode_editing_movement_and_line_join_are_safe() {
        let mut editor = Editor::new("echo".into(), "é界\nabc".into());
        editor.key(key(KeyCode::Right));
        editor.key(key(KeyCode::Delete));
        assert_eq!(editor.text, "é\nabc");
        editor.key(key(KeyCode::Down));
        editor.key(key(KeyCode::Home));
        editor.key(key(KeyCode::Backspace));
        assert_eq!(editor.text, "éabc");
        editor.key(key(KeyCode::Home));
        editor.key(key(KeyCode::Delete));
        assert_eq!(editor.text, "abc");
        editor.key(key(KeyCode::End));
        editor.key(key(KeyCode::Enter));
        editor.key(key(KeyCode::Tab));
        editor.insert("你好");
        editor.key(key(KeyCode::Left));
        editor.key(key(KeyCode::Backspace));
        assert_eq!(editor.text, "abc\n  好");
    }

    #[test]
    fn paste_and_validation_preserve_errors_without_dispatch() {
        let mut editor = Editor::new("echo".into(), String::new());
        editor.insert("{\r\n\t\"text\":\"jkgGlrq12\"\r\n}\u{1b}");
        assert_eq!(
            editor.arguments().ok(),
            Some(
                serde_json::json!({"text":"jkgGlrq12"})
                    .as_object()
                    .cloned()
                    .unwrap_or_default()
            )
        );
        editor.replace("[]".into());
        assert!(matches!(editor.arguments(), Err(AppError::ArgumentRoot)));
        editor.replace("{".into());
        assert!(matches!(editor.arguments(), Err(AppError::ArgumentJson(_))));
        editor.insert(&"x".repeat(MAX_ARGUMENT_BYTES));
        assert_eq!(editor.text, "{");
        assert!(matches!(
            editor.error.as_deref(),
            Some(AppError::ArgumentSize)
        ));
        editor.replace(" ".repeat(MAX_ARGUMENT_BYTES + 1));
        assert!(matches!(editor.arguments(), Err(AppError::ArgumentSize)));
    }

    #[test]
    fn viewport_follows_cursor_and_handles_wide_characters()
    -> std::result::Result<(), std::convert::Infallible> {
        let mut editor =
            Editor::new("echo".into(), format!("{}界\nlast", "x".repeat(70000)));
        editor.key(key(KeyCode::End));
        let mut terminal =
            ratatui::Terminal::new(ratatui::backend::TestBackend::new(20, 5))?;
        terminal.draw(|frame| editor.draw(frame, frame.area()))?;
        assert!(editor.left > 65535);
        editor.key(key(KeyCode::Down));
        editor.key(key(KeyCode::End));
        terminal.draw(|frame| editor.draw(frame, frame.area()))?;
        assert_eq!(editor.left, 0);
        assert_eq!(crop("界abc", 1, 3), " ab");
        Ok(())
    }
}
