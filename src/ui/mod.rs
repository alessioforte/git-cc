pub mod breadcrumb;
pub mod preview;
pub mod select_list;
pub mod text_input;
pub mod theme;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, Step};

pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // breadcrumb + separator
            Constraint::Min(5),   // main content
            Constraint::Length(1), // help footer
        ])
        .split(frame.area());

    // Breadcrumb
    breadcrumb::render(frame, chunks[0], app.step);

    // Main content area
    let main_area = chunks[1];
    // Add some horizontal padding
    let padded = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(main_area)[1];

    match app.step {
        Step::Kind => {
            select_list::render(
                frame,
                padded,
                &mut app.kind_select,
                "Commit Types",
                "Select the type of change that you're committing:",
            );
        }
        Step::Scope => {
            if app.scope_input_mode {
                let input_area = centered_input_area(padded);
                text_input::render(
                    frame,
                    input_area,
                    "Enter the new scope",
                    &app.scope_input.clone(),
                    crate::app::InputMode::Insert,
                    None,
                );
            } else {
                select_list::render(
                    frame,
                    padded,
                    &mut app.scope_select,
                    "Scopes",
                    "Select the scope of this change:",
                );
            }
        }
        Step::Emoji => {
            select_list::render(
                frame,
                padded,
                &mut app.emoji_select,
                "Gitmoji",
                "Select an emoji:",
            );
        }
        Step::Subject => {
            let input_area = centered_input_area(padded);
            let error = if app.subject_error {
                Some("Subject cannot be empty")
            } else {
                None
            };
            text_input::render(
                frame,
                input_area,
                "Write a short, imperative tense description of the change",
                &app.subject_input.clone(),
                app.subject_mode,
                error,
            );
        }
        Step::Description => {
            let mode_label = match app.desc_mode {
                crate::app::InputMode::Normal => " NORMAL ",
                crate::app::InputMode::Insert => " INSERT ",
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .title(Span::styled(
                    " Description (optional) ",
                    theme::title_style(),
                ));

            let textarea_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // prompt + mode
                    Constraint::Length(1), // spacer
                    Constraint::Min(3),   // textarea
                ])
                .split(padded);

            let mode_style = match app.desc_mode {
                crate::app::InputMode::Normal => theme::title_style(),
                crate::app::InputMode::Insert => theme::filter_style(),
            };
            let prompt = Paragraph::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Enter a longer description of the change:",
                    theme::title_style(),
                ),
                Span::raw("  "),
                Span::styled(mode_label, mode_style),
            ]));
            frame.render_widget(prompt, textarea_area[0]);

            app.desc_textarea.set_block(block);
            frame.render_widget(&app.desc_textarea, textarea_area[2]);
        }
        Step::Breaking => {
            let mode_label = match app.breaking_mode {
                crate::app::InputMode::Normal => " NORMAL ",
                crate::app::InputMode::Insert => " INSERT ",
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border_style())
                .title(Span::styled(
                    " Breaking Changes (optional) ",
                    theme::title_style(),
                ));

            let textarea_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Min(3),
                ])
                .split(padded);

            let mode_style = match app.breaking_mode {
                crate::app::InputMode::Normal => theme::title_style(),
                crate::app::InputMode::Insert => theme::filter_style(),
            };
            let prompt = Paragraph::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Describe any breaking changes:",
                    theme::title_style(),
                ),
                Span::raw("  "),
                Span::styled(mode_label, mode_style),
            ]));
            frame.render_widget(prompt, textarea_area[0]);

            app.breaking_textarea.set_block(block);
            frame.render_widget(&app.breaking_textarea, textarea_area[2]);
        }
        Step::Confirm => {
            preview::render(frame, padded, app);
        }
    }

    // Help footer
    use crate::app::InputMode;
    let help_text = match app.step {
        Step::Kind => match app.kind_select.mode {
            InputMode::Normal => "  j/k: Navigate | /,i: Filter | Enter: Select | Backspace: Back | Esc: Quit",
            InputMode::Insert => "  ↑/↓: Navigate | Type to filter | Enter: Select | Esc: Normal mode",
        },
        Step::Emoji => match app.emoji_select.mode {
            InputMode::Normal => "  j/k: Navigate | /,i: Filter | Enter: Select | Backspace: Back | Esc: Quit",
            InputMode::Insert => "  ↑/↓: Navigate | Type to filter | Enter: Select | Esc: Normal mode",
        },
        Step::Scope if app.scope_input_mode => {
            "  Enter: Confirm | Esc: Back to list"
        }
        Step::Scope => match app.scope_select.mode {
            InputMode::Normal => "  j/k: Navigate | /,i: Filter | Enter: Select | Backspace: Back | Esc: Quit",
            InputMode::Insert => "  ↑/↓: Navigate | Type to filter | Enter: Select | Esc: Normal mode",
        },
        Step::Subject => match app.subject_mode {
            InputMode::Normal => "  i: Insert | Enter: Confirm | Backspace: Back | Esc: Quit",
            InputMode::Insert => "  Type to edit | Enter: Confirm | Esc: Normal mode",
        },
        Step::Description => match app.desc_mode {
            InputMode::Normal => "  h/j/k/l: Move | i: Insert | a: Append | o: Open line | Enter: Done | Backspace: Back | Esc: Quit",
            InputMode::Insert => "  Type to edit | Ctrl+D: Done | Esc: Normal mode",
        },
        Step::Breaking => match app.breaking_mode {
            InputMode::Normal => "  h/j/k/l: Move | i: Insert | a: Append | o: Open line | Enter: Done | Backspace: Back | Esc: Quit",
            InputMode::Insert => "  Type to edit | Ctrl+D: Done | Esc: Normal mode",
        },
        Step::Confirm => {
            "  Enter: Commit | Backspace: Back | Ctrl+C: Quit"
        }
    };

    let help = Paragraph::new(Line::from(vec![
        Span::styled(help_text, theme::help_style()),
    ]));
    frame.render_widget(help, chunks[2]);
}

fn centered_input_area(area: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(6), // mode + spacer + input box + error
            Constraint::Min(0),
        ])
        .split(area);
    v[1]
}
