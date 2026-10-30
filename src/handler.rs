use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::{CursorMove, Input, Key};

use crate::app::{App, InputMode, Step};
use crate::settings;

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    // Ctrl+C always quits the flow
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.running = false;
        return;
    }

    // Esc behavior depends on context
    if key.code == KeyCode::Esc {
        // In scope input mode, go back to scope select
        if app.step == Step::Scope && app.scope_input_mode {
            app.scope_input_mode = false;
            app.scope_input.clear();
            return;
        }
        // In select Insert mode, go back to Normal mode
        match app.step {
            Step::Kind if app.kind_select.mode == InputMode::Insert => {
                app.kind_select.exit_insert();
                return;
            }
            Step::Scope if !app.scope_input_mode && app.scope_select.mode == InputMode::Insert => {
                app.scope_select.exit_insert();
                return;
            }
            Step::Emoji if app.emoji_select.mode == InputMode::Insert => {
                app.emoji_select.exit_insert();
                return;
            }
            _ => {}
        }
        // Subject / Textarea Insert mode -> back to Normal
        match app.step {
            Step::Subject if app.subject_mode == InputMode::Insert => {
                app.subject_mode = InputMode::Normal;
                return;
            }
            Step::Description if app.desc_mode == InputMode::Insert => {
                app.desc_mode = InputMode::Normal;
                return;
            }
            Step::Breaking if app.breaking_mode == InputMode::Insert => {
                app.breaking_mode = InputMode::Normal;
                return;
            }
            _ => {}
        }
        // In Normal mode, Esc does nothing (use Ctrl+C to quit)
        return;
    }

    match app.step {
        Step::Kind => handle_select(app, key, StepKind::Kind),
        Step::Scope => {
            if app.scope_input_mode {
                handle_scope_input(app, key);
            } else {
                handle_select(app, key, StepKind::Scope);
            }
        }
        Step::Emoji => handle_select(app, key, StepKind::Emoji),
        Step::Subject | Step::Description | Step::Breaking => handle_textarea(app, key),
        Step::Confirm => handle_confirm(app, key),
    }
}

#[derive(Clone, Copy)]
enum StepKind {
    Kind,
    Scope,
    Emoji,
}

fn get_select_state<'a>(app: &'a mut App, step_kind: StepKind) -> &'a mut crate::app::SelectState {
    match step_kind {
        StepKind::Kind => &mut app.kind_select,
        StepKind::Scope => &mut app.scope_select,
        StepKind::Emoji => &mut app.emoji_select,
    }
}

fn handle_select(app: &mut App, key: KeyEvent, step_kind: StepKind) {
    let mode = get_select_state(app, step_kind).mode;

    match mode {
        InputMode::Normal => handle_select_normal(app, key, step_kind),
        InputMode::Insert => handle_select_insert(app, key, step_kind),
    }
}

fn handle_select_normal(app: &mut App, key: KeyEvent, step_kind: StepKind) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            get_select_state(app, step_kind).move_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            get_select_state(app, step_kind).move_down();
        }
        KeyCode::Char('g') => {
            // gg - go to top (simplified: single g goes to top)
            let state = get_select_state(app, step_kind);
            state.cursor = 0;
            state.offset = 0;
        }
        KeyCode::Char('G') => {
            // G - go to bottom
            let state = get_select_state(app, step_kind);
            if !state.filtered.is_empty() {
                state.cursor = state.filtered.len() - 1;
            }
        }
        KeyCode::Char('/') | KeyCode::Char('i') => {
            get_select_state(app, step_kind).enter_insert();
        }
        KeyCode::Enter => {
            let state = get_select_state(app, step_kind);
            if let Some(value) = state.selected_value().map(|v| v.to_string()) {
                match step_kind {
                    StepKind::Kind => {
                        app.kind = Some(value);
                        app.kind_select.exit_insert();
                        app.go_next();
                    }
                    StepKind::Scope => {
                        if value == "__new_scope__" {
                            app.scope_input_mode = true;
                            app.scope_save = true;
                            app.scope_input.clear();
                            app.scope_select.exit_insert();
                        } else if value == "__new_scope_once__" {
                            app.scope_input_mode = true;
                            app.scope_save = false;
                            app.scope_input.clear();
                            app.scope_select.exit_insert();
                        } else {
                            app.scope = if value.is_empty() { None } else { Some(value) };
                            app.scope_select.exit_insert();
                            app.go_next();
                        }
                    }
                    StepKind::Emoji => {
                        app.emoji = Some(value);
                        app.emoji_select.exit_insert();
                        app.go_next();
                    }
                }
            }
        }
        KeyCode::Backspace => {
            app.go_back();
        }
        _ => {}
    }
}

fn handle_select_insert(app: &mut App, key: KeyEvent, step_kind: StepKind) {
    match key.code {
        KeyCode::Up => {
            get_select_state(app, step_kind).move_up();
        }
        KeyCode::Down => {
            get_select_state(app, step_kind).move_down();
        }
        KeyCode::Enter => {
            let state = get_select_state(app, step_kind);
            if let Some(value) = state.selected_value().map(|v| v.to_string()) {
                match step_kind {
                    StepKind::Kind => {
                        app.kind = Some(value);
                        app.kind_select.exit_insert();
                        app.go_next();
                    }
                    StepKind::Scope => {
                        if value == "__new_scope__" {
                            app.scope_input_mode = true;
                            app.scope_save = true;
                            app.scope_input.clear();
                            app.scope_select.exit_insert();
                        } else if value == "__new_scope_once__" {
                            app.scope_input_mode = true;
                            app.scope_save = false;
                            app.scope_input.clear();
                            app.scope_select.exit_insert();
                        } else {
                            app.scope = if value.is_empty() { None } else { Some(value) };
                            app.scope_select.exit_insert();
                            app.go_next();
                        }
                    }
                    StepKind::Emoji => {
                        app.emoji = Some(value);
                        app.emoji_select.exit_insert();
                        app.go_next();
                    }
                }
            }
        }
        KeyCode::Backspace => {
            let state = get_select_state(app, step_kind);
            if !state.filter.is_empty() {
                state.pop_char();
            } else {
                // Empty filter + backspace -> back to Normal mode
                state.exit_insert();
            }
        }
        KeyCode::Char(c) => {
            get_select_state(app, step_kind).push_char(c);
        }
        _ => {}
    }
}

fn handle_scope_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            if !app.scope_input.is_empty() {
                let scope_name = app.scope_input.clone();
                if app.scope_save {
                    app.notice = match settings::save_scope(&scope_name) {
                        Ok(()) => None,
                        Err(err) => Some(format!("Could not save scope: {err}")),
                    };
                    // Update scope select items to include the new scope
                    let insert_pos = app.scope_select.items.len() - 2;
                    app.scope_select
                        .items
                        .insert(insert_pos, (scope_name.clone(), scope_name.clone()));
                    app.scope_select.apply_filter();
                }
                app.scope = Some(scope_name);
                app.scope_input_mode = false;
                app.scope_input.clear();
                app.go_next();
            }
        }
        KeyCode::Backspace => {
            app.scope_input.pop();
        }
        KeyCode::Char(c) => {
            app.scope_input.push(c);
        }
        _ => {}
    }
}

fn subject_confirm(app: &mut App) {
    let subject = &app.subject_input.lines()[0];
    if subject.is_empty() {
        app.subject_error = true;
    } else {
        app.subject = Some(subject.clone());
        app.subject_error = false;
        app.subject_mode = InputMode::Normal;
        app.go_next();
    }
}

fn textarea_finish(app: &mut App) {
    if app.step == Step::Subject {
        subject_confirm(app);
        return;
    }

    let (textarea, mode, value) = if app.step == Step::Breaking {
        (
            &app.breaking_textarea,
            &mut app.breaking_mode,
            &mut app.breaking,
        )
    } else {
        (&app.desc_textarea, &mut app.desc_mode, &mut app.description)
    };
    let content = textarea.lines().join("\n").trim().to_string();
    *value = if content.is_empty() {
        None
    } else {
        Some(content)
    };
    *mode = InputMode::Normal;
    app.go_next();
}

fn handle_textarea(app: &mut App, key: KeyEvent) {
    let single_line = app.step == Step::Subject;
    if (key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::CONTROL))
        || (single_line && key.code == KeyCode::Enter)
    {
        textarea_finish(app);
        return;
    }

    let (textarea, mode) = match app.step {
        Step::Subject => (&mut app.subject_input, &mut app.subject_mode),
        Step::Description => (&mut app.desc_textarea, &mut app.desc_mode),
        Step::Breaking => (&mut app.breaking_textarea, &mut app.breaking_mode),
        _ => return,
    };

    match *mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('i') | KeyCode::Char('/') => *mode = InputMode::Insert,
            KeyCode::Char('a') => {
                *mode = InputMode::Insert;
                textarea.move_cursor(CursorMove::Forward);
            }
            KeyCode::Char('o') if !single_line => {
                *mode = InputMode::Insert;
                textarea.move_cursor(CursorMove::End);
                textarea.insert_newline();
            }
            KeyCode::Char('h') | KeyCode::Left => textarea.move_cursor(CursorMove::Back),
            KeyCode::Char('l') | KeyCode::Right => textarea.move_cursor(CursorMove::Forward),
            KeyCode::Char('j') | KeyCode::Down => textarea.move_cursor(CursorMove::Down),
            KeyCode::Char('k') | KeyCode::Up => textarea.move_cursor(CursorMove::Up),
            KeyCode::Char('0') | KeyCode::Home => textarea.move_cursor(CursorMove::Head),
            KeyCode::Char('$') | KeyCode::End => textarea.move_cursor(CursorMove::End),
            KeyCode::Char('w') => textarea.move_cursor(CursorMove::WordForward),
            KeyCode::Char('b') => textarea.move_cursor(CursorMove::WordBack),
            KeyCode::Enter => textarea_finish(app),
            KeyCode::Backspace => app.go_back(),
            _ => {}
        },
        InputMode::Insert => {
            let input = Input::from(key);
            // Enter confirms Subject above. Block the editor's other newline inputs.
            if single_line
                && matches!(
                    input,
                    Input {
                        key: Key::Char('\n' | '\r'),
                        ..
                    } | Input {
                        key: Key::Char('m'),
                        ctrl: true,
                        ..
                    }
                )
            {
                return;
            }
            if textarea.input(input) && single_line {
                app.subject_error = false;
            }
        }
    }
}

fn handle_confirm(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.preview_scroll = app.preview_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.preview_scroll = app
                .preview_scroll
                .saturating_add(1)
                .min(app.preview_max_scroll);
        }
        KeyCode::PageUp => {
            app.preview_scroll = app.preview_scroll.saturating_sub(app.preview_height);
        }
        KeyCode::PageDown => {
            app.preview_scroll = app
                .preview_scroll
                .saturating_add(app.preview_height)
                .min(app.preview_max_scroll);
        }
        KeyCode::Home | KeyCode::Char('g') => app.preview_scroll = 0,
        KeyCode::End | KeyCode::Char('G') => app.preview_scroll = app.preview_max_scroll,
        KeyCode::Enter => {
            app.commit_error = None;
            app.committed = true;
            app.running = false;
        }
        KeyCode::Backspace => {
            app.go_back();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject_app() -> App<'static> {
        let mut app = App::with_settings(settings::Settings::new());
        app.step = Step::Subject;
        app.subject_mode = InputMode::Insert;
        app
    }

    fn press(app: &mut App, code: KeyCode) {
        handle_key_event(app, KeyEvent::new(code, KeyModifiers::NONE));
    }

    fn type_text(app: &mut App, text: &str) {
        for c in text.chars() {
            press(app, KeyCode::Char(c));
        }
    }

    #[test]
    fn subject_edits_at_the_cursor() {
        let mut app = subject_app();
        type_text(&mut app, "fix bug");
        for _ in 0..3 {
            press(&mut app, KeyCode::Left);
        }
        type_text(&mut app, "the ");
        assert_eq!(app.subject_input.lines(), &["fix the bug"]);
        assert_eq!(app.subject_input.cursor(), (0, 8));

        press(&mut app, KeyCode::Delete);
        press(&mut app, KeyCode::Right);
        press(&mut app, KeyCode::Backspace);
        type_text(&mut app, "do");
        assert_eq!(app.subject_input.lines(), &["fix the dog"]);

        press(&mut app, KeyCode::Enter);
        assert_eq!(app.step, Step::Description);
        assert_eq!(app.subject.as_deref(), Some("fix the dog"));
        assert!(app.formatted_commit().ends_with("fix the dog"));
    }

    #[test]
    fn subject_navigation_and_deletion_handle_unicode() {
        let mut app = subject_app();
        type_text(&mut app, "café🦀界");
        press(&mut app, KeyCode::Left);
        press(&mut app, KeyCode::Left);
        press(&mut app, KeyCode::Backspace);
        type_text(&mut app, "è");
        press(&mut app, KeyCode::Delete);
        assert_eq!(app.subject_input.lines(), &["cafè界"]);
        assert_eq!(app.subject_input.cursor(), (0, 4));
        press(&mut app, KeyCode::Right);
        type_text(&mut app, "!");
        assert_eq!(app.subject_input.lines(), &["cafè界!"]);
    }

    #[test]
    fn subject_uses_the_same_editor_commands_as_multiline_fields() {
        let commands = [
            (KeyCode::Char('b'), KeyModifiers::ALT),
            (KeyCode::Char('f'), KeyModifiers::ALT),
            (KeyCode::Char('a'), KeyModifiers::CONTROL),
            (KeyCode::Char('e'), KeyModifiers::CONTROL),
            (KeyCode::Left, KeyModifiers::CONTROL),
            (KeyCode::Right, KeyModifiers::CONTROL),
            (KeyCode::Left, KeyModifiers::SHIFT),
            (KeyCode::Char('!'), KeyModifiers::NONE),
            (KeyCode::Char('u'), KeyModifiers::CONTROL),
            (KeyCode::Char('r'), KeyModifiers::CONTROL),
            (KeyCode::Backspace, KeyModifiers::ALT),
            (KeyCode::Char('y'), KeyModifiers::CONTROL),
            (KeyCode::Char('j'), KeyModifiers::CONTROL),
            (KeyCode::Char('u'), KeyModifiers::CONTROL),
        ];
        for step in [Step::Description, Step::Breaking] {
            let mut app = subject_app();
            type_text(&mut app, "fix café bug");
            app.desc_textarea.insert_str("fix café bug");
            app.breaking_textarea.insert_str("fix café bug");
            app.desc_mode = InputMode::Insert;
            app.breaking_mode = InputMode::Insert;

            for (code, modifiers) in commands {
                let key = KeyEvent::new(code, modifiers);
                app.step = Step::Subject;
                handle_key_event(&mut app, key);
                app.step = step;
                handle_key_event(&mut app, key);
                let other = if step == Step::Description {
                    &app.desc_textarea
                } else {
                    &app.breaking_textarea
                };
                assert_eq!(app.subject_input.lines(), other.lines(), "{key:?}");
                assert_eq!(app.subject_input.cursor(), other.cursor(), "{key:?}");
                assert_eq!(app.subject_input.selection_range(), other.selection_range());
                assert_eq!(app.subject_input.lines().len(), 1);
            }
        }
    }

    #[test]
    fn only_multiline_fields_allow_newlines() {
        for step in [Step::Subject, Step::Description, Step::Breaking] {
            let mut app = subject_app();
            app.step = step;
            app.desc_mode = InputMode::Insert;
            app.breaking_mode = InputMode::Insert;
            type_text(&mut app, "first");
            for key in [
                KeyEvent::new(KeyCode::Char('m'), KeyModifiers::CONTROL),
                KeyEvent::new(KeyCode::Char('\n'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('\r'), KeyModifiers::NONE),
            ] {
                handle_key_event(&mut app, key);
            }
            type_text(&mut app, "second");
            if step == Step::Subject {
                assert_eq!(app.subject_input.lines(), &["firstsecond"]);
                press(&mut app, KeyCode::Enter);
                assert_eq!(app.subject.as_deref(), Some("firstsecond"));
            } else {
                press(&mut app, KeyCode::Enter);
                type_text(&mut app, "third");
                handle_key_event(
                    &mut app,
                    KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
                );
                let value = if step == Step::Description {
                    &app.description
                } else {
                    &app.breaking
                };
                assert_eq!(value.as_deref(), Some("first\n\n\nsecond\nthird"));
            }
            assert_eq!(app.step, step.next().unwrap());
        }
    }

    #[test]
    fn subject_normal_mode_cannot_open_another_line() {
        let mut app = subject_app();
        type_text(&mut app, "subject");
        press(&mut app, KeyCode::Esc);
        press(&mut app, KeyCode::Char('o'));
        assert_eq!(app.subject_input.lines(), &["subject"]);
        assert_eq!(app.subject_mode, InputMode::Normal);
        press(&mut app, KeyCode::Char('a'));
        assert_eq!(app.subject_mode, InputMode::Insert);
        handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
        );
        assert_eq!(app.subject.as_deref(), Some("subject"));
        assert_eq!(app.step, Step::Description);
    }

    #[test]
    fn subject_cursor_stops_at_line_boundaries() {
        let mut app = subject_app();
        press(&mut app, KeyCode::Left);
        press(&mut app, KeyCode::Right);
        press(&mut app, KeyCode::Backspace);
        press(&mut app, KeyCode::Delete);
        assert_eq!(app.subject_input.cursor(), (0, 0));
        assert_eq!(app.subject_input.lines(), &[""]);

        type_text(&mut app, "text");
        press(&mut app, KeyCode::Home);
        press(&mut app, KeyCode::Left);
        press(&mut app, KeyCode::Backspace);
        assert_eq!(app.subject_input.cursor(), (0, 0));
        press(&mut app, KeyCode::End);
        press(&mut app, KeyCode::Right);
        press(&mut app, KeyCode::Delete);
        assert_eq!(app.subject_input.cursor(), (0, 4));
        assert_eq!(app.subject_input.lines(), &["text"]);
    }

    #[test]
    fn subject_preserves_cursor_across_modes_and_step_navigation() {
        let mut app = subject_app();
        type_text(&mut app, "fix bug");
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.subject_mode, InputMode::Normal);
        press(&mut app, KeyCode::Home);
        press(&mut app, KeyCode::Right);
        press(&mut app, KeyCode::Right);
        press(&mut app, KeyCode::Left);
        assert_eq!(app.subject_input.cursor(), (0, 1));
        press(&mut app, KeyCode::Char('i'));
        type_text(&mut app, "!");
        press(&mut app, KeyCode::Enter);
        press(&mut app, KeyCode::Backspace);
        assert_eq!(app.step, Step::Subject);
        assert_eq!(app.subject_input.cursor(), (0, 2));
        assert_eq!(app.subject_input.lines(), &["f!ix bug"]);
        press(&mut app, KeyCode::Backspace);
        assert_eq!(app.step, Step::Emoji);
    }

    #[test]
    fn subject_keeps_validation_and_cancellation() {
        let mut app = subject_app();
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.step, Step::Subject);
        assert!(app.subject_error);
        type_text(&mut app, "fix");
        assert!(!app.subject_error);

        press(&mut app, KeyCode::Esc);
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.subject.as_deref(), Some("fix"));
        press(&mut app, KeyCode::Backspace);
        handle_key_event(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(!app.running);
        assert!(!app.committed);
    }
}
