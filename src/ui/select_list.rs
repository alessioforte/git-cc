use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{InputMode, SelectState};
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, state: &mut SelectState, title: &str, prompt: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // prompt
            Constraint::Length(1), // filter input
            Constraint::Length(1), // spacer
            Constraint::Min(3),   // list
        ])
        .split(area);

    // Prompt
    let prompt_line = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(prompt, theme::title_style()),
    ]));
    frame.render_widget(prompt_line, chunks[0]);

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
        InputMode::Insert => {
            Line::from(vec![
                Span::styled("  INSERT", theme::filter_style()),
                Span::styled(format!("  > {}_", state.filter), theme::filter_style()),
            ])
        }
    };
    let filter_line = Paragraph::new(filter_display);
    frame.render_widget(filter_line, chunks[1]);

    // List area
    let list_area = chunks[3];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .title(Span::styled(
            format!(" {} ", title),
            theme::title_style(),
        ));

    let inner = block.inner(list_area);
    let inner_height = inner.height as usize;

    // Adjust scroll offset using the actual inner height (excluding borders)
    state.adjust_offset_for_height(inner_height);

    frame.render_widget(block, list_area);

    // Render visible items
    let end = (state.offset + inner_height).min(state.filtered.len());
    let visible_items = &state.filtered[state.offset..end];

    for (row, &item_idx) in visible_items.iter().enumerate() {
        let (ref _value, ref label) = state.items[item_idx];
        let is_selected = state.offset + row == state.cursor;

        let style = if is_selected {
            theme::selected_style()
        } else {
            ratatui::style::Style::default()
        };

        let item_area = Rect {
            x: inner.x,
            y: inner.y + row as u16,
            width: inner.width,
            height: 1,
        };

        // Pad the display string to fill the entire row width so the
        // highlight background spans the full row, not just the text.
        let text = format!(" {} ", label);
        let padded = format!("{:<width$}", text, width = inner.width as usize);
        let line = Line::from(vec![Span::styled(padded, style)]);

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
            x: inner.x + inner.width.saturating_sub(1),
            y: inner.y + indicator_pos,
            width: 1,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new("█").style(
                ratatui::style::Style::default()
                    .fg(theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            scroll_area,
        );
    }
}
