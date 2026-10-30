use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::app::App;
use crate::ui::theme;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(1), // spacer
            Constraint::Min(5),    // preview block
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
    let inner = chunks[2];

    app.preview_height = inner.height;
    if inner.is_empty() {
        app.preview_scroll = 0;
        app.preview_max_scroll = 0;
        return;
    }

    let preview = Paragraph::new(format!("  {}\n", commit_text.replace('\n', "\n  ")))
        .wrap(Wrap { trim: false });
    let line_count = preview.line_count(inner.width);
    app.preview_max_scroll = line_count
        .saturating_sub(inner.height as usize)
        .min(u16::MAX as usize) as u16;
    app.preview_scroll = app.preview_scroll.min(app.preview_max_scroll);

    frame.render_widget(preview.scroll((app.preview_scroll, 0)), inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use crate::app::Step;
    use crate::handler::handle_key_event;
    use crate::settings::Settings;

    fn preview_app() -> App<'static> {
        let mut app = App::with_settings(Settings::new());
        app.step = Step::Confirm;
        app.kind = Some("feat".into());
        app.subject = Some("Preview scrolling".into());
        app.description = Some((0..20).map(|i| format!("Line {i:02}\n")).collect());
        app.breaking = Some("LAST LINE".into());
        app
    }

    fn draw(terminal: &mut Terminal<TestBackend>, app: &mut App) -> String {
        terminal
            .draw(|frame| render(frame, frame.area(), app))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    fn press(app: &mut App, code: KeyCode) {
        handle_key_event(app, KeyEvent::new(code, KeyModifiers::NONE));
    }

    #[test]
    fn preview_scrolls_by_line_page_and_to_both_ends() {
        let mut app = preview_app();
        let commit = app.formatted_commit();
        let mut terminal = Terminal::new(TestBackend::new(32, 10)).unwrap();
        let top = draw(&mut terminal, &mut app);
        assert!(app.preview_max_scroll > app.preview_height);
        assert!(!top.contains("LAST LINE"));

        press(&mut app, KeyCode::Down);
        assert_eq!(app.preview_scroll, 1);
        assert_ne!(draw(&mut terminal, &mut app), top);
        press(&mut app, KeyCode::Up);
        assert_eq!(draw(&mut terminal, &mut app), top);
        press(&mut app, KeyCode::Char('j'));
        press(&mut app, KeyCode::Char('k'));
        assert_eq!(app.preview_scroll, 0);

        press(&mut app, KeyCode::PageDown);
        assert_eq!(app.preview_scroll, app.preview_height);
        press(&mut app, KeyCode::PageUp);
        assert_eq!(app.preview_scroll, 0);
        press(&mut app, KeyCode::End);
        assert_eq!(app.preview_scroll, app.preview_max_scroll);
        assert!(draw(&mut terminal, &mut app).contains("LAST LINE"));
        press(&mut app, KeyCode::Down);
        press(&mut app, KeyCode::PageDown);
        assert_eq!(app.preview_scroll, app.preview_max_scroll);
        press(&mut app, KeyCode::Home);
        press(&mut app, KeyCode::Up);
        assert_eq!(draw(&mut terminal, &mut app), top);
        assert_eq!(app.formatted_commit(), commit);
        assert!(app.running);
        assert!(!app.committed);

        press(&mut app, KeyCode::Enter);
        assert!(app.committed);
        assert!(!app.running);
    }

    #[test]
    fn preview_counts_wrapped_unicode_lines_and_clamps_after_resize() {
        let mut app = preview_app();
        app.description = Some(format!("{}END", "café 界 🦀 ".repeat(20)));
        app.breaking = None;
        let mut terminal = Terminal::new(TestBackend::new(24, 8)).unwrap();
        draw(&mut terminal, &mut app);
        assert!(app.preview_max_scroll > 3);
        press(&mut app, KeyCode::Char('G'));
        assert!(draw(&mut terminal, &mut app).contains("END"));

        terminal.backend_mut().resize(100, 30);
        let expanded = draw(&mut terminal, &mut app);
        assert_eq!(app.preview_scroll, 0);
        assert_eq!(app.preview_max_scroll, 0);
        assert!(expanded.contains("END"));

        terminal.backend_mut().resize(24, 8);
        draw(&mut terminal, &mut app);
        assert!(app.preview_max_scroll > 0);
        press(&mut app, KeyCode::End);
        press(&mut app, KeyCode::Backspace);
        assert_eq!(app.step, Step::Breaking);
        app.go_next();
        assert_eq!(app.preview_scroll, 0);
        assert!(!draw(&mut terminal, &mut app).contains("END"));
    }

    #[test]
    fn preview_handles_tiny_areas_without_scrolling_past_the_content() {
        let mut app = preview_app();
        for (width, height) in [(0, 0), (1, 1), (2, 3), (4, 6)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            app.preview_scroll = u16::MAX;
            draw(&mut terminal, &mut app);
            assert!(app.preview_scroll <= app.preview_max_scroll);
            press(&mut app, KeyCode::PageDown);
            assert!(app.preview_scroll <= app.preview_max_scroll);
        }
    }
}
