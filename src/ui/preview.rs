use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(1), // spacer
            Constraint::Min(5),   // preview block
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("Review your commit:", theme::title_style()),
    ]));
    frame.render_widget(title, chunks[0]);

    // Commit preview
    let commit_text = app.formatted_commit();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .title(Span::styled(" Commit Message ", theme::title_style()));

    let preview = Paragraph::new(format!("\n  {}\n", commit_text.replace('\n', "\n  ")))
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(preview, chunks[2]);
}
