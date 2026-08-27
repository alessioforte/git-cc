use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::Step;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, current_step: Step) {
    let current_index = current_step.index();
    let mut spans: Vec<Span> = Vec::new();

    spans.push(Span::raw("  "));

    for (i, step) in Step::ALL.iter().enumerate() {
        let style = if i < current_index {
            theme::breadcrumb_done()
        } else if i == current_index {
            theme::breadcrumb_current()
        } else {
            theme::breadcrumb_future()
        };

        let label = if i == current_index {
            format!("[{}]", step.label())
        } else {
            step.label().to_string()
        };

        spans.push(Span::styled(label, style));

        if i < Step::ALL.len() - 1 {
            spans.push(Span::styled(" > ", theme::dimmed_style()));
        }
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
