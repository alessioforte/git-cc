use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{InputMode, SelectState};
use crate::ui::theme;

const CURSOR: &str = "▸";

pub fn render(frame: &mut Frame, area: Rect, state: &mut SelectState, title: &str, prompt: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(1), // prompt
            Constraint::Length(1), // filter input
            Constraint::Length(1), // spacer
            Constraint::Min(3),    // list
        ])
        .split(area);

    // Title
    let title_line = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(title, theme::title_style()),
    ]));
    frame.render_widget(title_line, chunks[0]);

    // Prompt
    let prompt_line = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(prompt, theme::dimmed_style()),
    ]));
    frame.render_widget(prompt_line, chunks[1]);

    // Filter input - show mode indicator
    let filter_display = match state.mode {
        InputMode::Normal => {
            if state.filter.is_empty() {
                Line::from(vec![
                    Span::styled("  NORMAL", theme::title_style()),
                    Span::styled("  Press / or i to filter", theme::dimmed_style()),
                ])
            } else {
                Line::from(vec![
                    Span::styled("  NORMAL", theme::title_style()),
                    Span::styled(format!("  > {}", state.filter), theme::filter_style()),
                ])
            }
        }
        InputMode::Insert => Line::from(vec![
            Span::styled("  INSERT", theme::filter_style()),
            Span::styled(format!("  > {}_", state.filter), theme::filter_style()),
        ]),
    };
    let filter_line = Paragraph::new(filter_display);
    frame.render_widget(filter_line, chunks[2]);

    // List area
    let list_area = chunks[4];
    let inner_height = list_area.height as usize;

    state.adjust_offset_for_height(inner_height);

    // Render visible items
    let end = (state.offset + inner_height).min(state.filtered.len());
    let visible_items = &state.filtered[state.offset..end];

    for (row, &item_idx) in visible_items.iter().enumerate() {
        let (ref _value, ref label) = state.items[item_idx];
        let is_selected = state.offset + row == state.cursor;

        let style = if is_selected {
            theme::selected_style()
        } else {
            Style::default()
        };
        let cursor = if is_selected { CURSOR } else { " " };

        let item_area = Rect {
            x: list_area.x,
            y: list_area.y + row as u16,
            width: list_area.width,
            height: 1,
        };

        let line = Line::from(vec![
            Span::styled(format!("  {cursor} "), style),
            Span::styled(label.as_str(), style),
        ]);

        frame.render_widget(Paragraph::new(line), item_area);
    }

    // Scroll indicator
    if state.filtered.len() > inner_height {
        let ratio = if !state.filtered.is_empty() {
            state.cursor as f64 / state.filtered.len() as f64
        } else {
            0.0
        };
        let indicator_pos = (ratio * (inner_height.saturating_sub(1)) as f64) as u16;

        let scroll_area = Rect {
            x: list_area.x + list_area.width.saturating_sub(1),
            y: list_area.y + indicator_pos,
            width: 1,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new("█").style(theme::selected_style()),
            scroll_area,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::style::Color;
    use ratatui::Terminal;

    #[test]
    fn selection_uses_color_and_left_cursor_without_highlight() {
        let items = vec![
            ("feat".to_string(), "feat  A new feature".to_string()),
            ("fix".to_string(), "fix   A bug fix".to_string()),
        ];
        let mut state = SelectState::new(items);
        let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
        terminal
            .draw(|frame| render(frame, frame.area(), &mut state, "Commit Types", "Pick one:"))
            .unwrap();

        let buffer = terminal.backend().buffer();
        // Title, prompt, filter line and spacer precede the list.
        assert_eq!(buffer[(2, 4)].symbol(), CURSOR);
        assert_eq!(buffer[(2, 4)].fg, theme::PRIMARY);
        assert_eq!(buffer[(2, 4)].bg, Color::Reset);
        assert_eq!(buffer[(4, 4)].symbol(), "f");
        assert_eq!(buffer[(4, 4)].fg, theme::PRIMARY);

        assert_eq!(buffer[(2, 5)].symbol(), " ");
        assert_eq!(buffer[(4, 5)].symbol(), "f");
        assert_eq!(buffer[(4, 5)].fg, Color::Reset);
        assert_eq!(buffer[(4, 5)].bg, Color::Reset);
    }
}
