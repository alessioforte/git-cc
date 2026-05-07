use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
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
            Constraint::Length(1), // spacer
            Constraint::Length(1), // buttons
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

    // Action buttons
    let button_labels = ["  Commit  ", "  Go Back  ", "  Cancel  "];
    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw("      "));

    for (i, label) in button_labels.iter().enumerate() {
        let style = if i == app.confirm_cursor {
            theme::confirm_button_selected()
        } else {
            theme::confirm_button_normal()
        };

        spans.push(Span::styled(format!("[{}]", label), style));
        if i < button_labels.len() - 1 {
            spans.push(Span::raw("    "));
        }
    }

    let buttons = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);
    frame.render_widget(buttons, chunks[4]);
}
