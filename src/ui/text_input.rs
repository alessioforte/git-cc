use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::InputMode;
use crate::ui::theme;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    prompt: &str,
    value: &str,
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

    // Input text with cursor
    let display = match mode {
        InputMode::Insert => format!("{}_", value),
        InputMode::Normal => value.to_string(),
    };
    let input_line = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::raw(display),
    ]));
    frame.render_widget(input_line, inner);

    // Error message
    if let Some(err_msg) = error {
        let err_line = Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(err_msg, theme::error_style()),
        ]));
        frame.render_widget(err_line, chunks[3]);
    }
}
