use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::Frame;
use tui_textarea::TextArea;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::InputMode;
use crate::ui::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    prompt: &str,
    input: impl Widget,
    mode: InputMode,
    error: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // mode indicator
            Constraint::Length(1), // spacer
            Constraint::Length(3), // input box
            Constraint::Length(1), // error
            Constraint::Min(0),   // rest
        ])
        .split(area);

    // Mode indicator
    let (mode_label, mode_style) = match mode {
        InputMode::Normal => ("  NORMAL", theme::title_style()),
        InputMode::Insert => ("  INSERT", theme::filter_style()),
    };
    let hint = match mode {
        InputMode::Normal => "  Press i to type",
        InputMode::Insert => "",
    };
    let mode_line = Paragraph::new(Line::from(vec![
        Span::styled(mode_label, mode_style),
        Span::styled(hint, theme::dimmed_style()),
    ]));
    frame.render_widget(mode_line, chunks[0]);

    // Input box
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .title(Span::styled(format!(" {} ", prompt), theme::title_style()));

    let inner = block.inner(chunks[2]);
    frame.render_widget(block, chunks[2]);

    frame.render_widget(input, inner.inner(Margin::new(1, 0)));

    // Error message
    if let Some(err_msg) = error {
        let err_line = Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(err_msg, theme::error_style()),
        ]));
        frame.render_widget(err_line, chunks[3]);
    }
}

pub struct SingleLine<'a, 'b>(pub &'a TextArea<'b>);

impl Widget for SingleLine<'_, '_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if area.is_empty() {
            return;
        }

        // tui-textarea 0.7 scrolls by character count instead of display width.
        // Let the editor draw the complete row (including selection), then crop it.
        let value = &self.0.lines()[0];
        let width = (value.width().max(value.chars().count()) + 1)
            .clamp(area.width as usize, u16::MAX as usize) as u16;
        let row_area = Rect::new(0, 0, width, 1);
        let mut row = Buffer::empty(row_area);
        self.0.render(row_area, &mut row);

        let cursor_byte = value
            .char_indices()
            .nth(self.0.cursor().1)
            .map_or(value.len(), |(index, _)| index);
        let cursor_width = value[cursor_byte..]
            .chars()
            .next()
            .and_then(UnicodeWidthChar::width)
            .unwrap_or(1)
            .max(1);
        let scroll = (value[..cursor_byte].width() + cursor_width)
            .saturating_sub(area.width as usize)
            .min((width - area.width) as usize) as u16;
        for x in 0..area.width {
            buffer[(area.x + x, area.y)] = row[(scroll + x, 0)].clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::style::{Color, Modifier};
    use ratatui::Terminal;
    use tui_textarea::{CursorMove, TextArea};

    #[test]
    fn input_scrolls_to_keep_the_cursor_visible() {
        let mut terminal = Terminal::new(TestBackend::new(14, 8)).unwrap();
        let mut input = TextArea::from(["ab界defghijklmnop"]);

        for (movement, mode) in [
            (CursorMove::End, InputMode::Insert),
            (CursorMove::Head, InputMode::Normal),
        ] {
            input.move_cursor(movement);
            terminal
                .draw(|frame| {
                    render(
                        frame,
                        frame.area(),
                        "Subject",
                        SingleLine(&input),
                        mode,
                        None,
                    )
                })
                .unwrap();
            let buffer = terminal.backend().buffer();
            let cursor_x = (2..12)
                .find(|&x| buffer[(x, 3)].modifier.contains(Modifier::REVERSED))
                .unwrap_or_else(|| panic!("cursor should be visible: {buffer:?}"));
            if movement == CursorMove::End {
                assert_eq!(buffer[(cursor_x - 1, 3)].symbol(), "p");
            } else {
                assert_eq!(cursor_x, 2);
                assert_eq!(buffer[(cursor_x, 3)].symbol(), "a");
            }
        }

        input.move_cursor(CursorMove::Forward);
        input.move_cursor(CursorMove::Forward);
        terminal
            .draw(|frame| {
                render(
                    frame,
                    frame.area(),
                    "Subject",
                    SingleLine(&input),
                    InputMode::Insert,
                    None,
                )
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(4, 3)].symbol(), "界");
        assert!(buffer[(4, 3)].modifier.contains(Modifier::REVERSED));
    }

    #[test]
    fn input_renders_in_small_terminal_areas() {
        let input = TextArea::from(["é🦀界"]);
        for (width, height) in [(0, 0), (1, 1), (2, 3), (4, 6)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal
                .draw(|frame| {
                    render(
                        frame,
                        frame.area(),
                        "Subject",
                        SingleLine(&input),
                        InputMode::Insert,
                        None,
                    )
                })
                .unwrap();
        }
    }

    #[test]
    fn input_renders_the_editors_selection() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut input = TextArea::from(["fix bug"]);
        input.move_cursor(CursorMove::End);
        input.input(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        input.input(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
        let mut terminal = Terminal::new(TestBackend::new(14, 8)).unwrap();
        terminal
            .draw(|frame| {
                render(
                    frame,
                    frame.area(),
                    "Subject",
                    SingleLine(&input),
                    InputMode::Insert,
                    None,
                );
            })
            .unwrap();
        assert_eq!(input.selection_range(), Some(((0, 5), (0, 7))));
        assert_eq!(terminal.backend().buffer()[(8, 3)].bg, Color::LightBlue);
    }
}
