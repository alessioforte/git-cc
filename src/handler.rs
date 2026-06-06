use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
        Step::Subject => handle_subject(app, key),
        Step::Description => handle_textarea(app, key, false),
        Step::Breaking => handle_textarea(app, key, true),
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
                            app.scope = if value.is_empty() {
                                None
                            } else {
                                Some(value)
                            };
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
                            app.scope = if value.is_empty() {
                                None
                            } else {
                                Some(value)
                            };
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
                    let mut settings_data = settings::get_settings();
                    if !settings_data.scopes.contains(&scope_name) {
                        settings_data.scopes.push(scope_name.clone());
                    }
                    settings_data.save();
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
    if app.subject_input.is_empty() {
        app.subject_error = true;
    } else {
        app.subject = Some(app.subject_input.clone());
        app.subject_error = false;
        app.subject_mode = InputMode::Normal;
        app.go_next();
    }
}

fn handle_subject(app: &mut App, key: KeyEvent) {
    match app.subject_mode {
        InputMode::Normal => {
            match key.code {
                KeyCode::Char('i') | KeyCode::Char('/') => {
                    app.subject_mode = InputMode::Insert;
                }
                KeyCode::Enter => {
                    subject_confirm(app);
                }
                KeyCode::Backspace => {
                    app.go_back();
                }
                _ => {}
            }
        }
        InputMode::Insert => {
            match key.code {
                KeyCode::Enter => {
                    subject_confirm(app);
                }
                KeyCode::Backspace => {
                    app.subject_input.pop();
                    app.subject_error = false;
                }
                KeyCode::Char(c) => {
                    app.subject_input.push(c);
                    app.subject_error = false;
                }
                _ => {}
            }
        }
    }
}

fn textarea_finish(app: &mut App, is_breaking: bool) {
    let textarea = if is_breaking {
        &app.breaking_textarea
    } else {
        &app.desc_textarea
    };
    let content = textarea.lines().join("\n").trim().to_string();
    if is_breaking {
        app.breaking = if content.is_empty() {
            None
        } else {
            Some(content)
        };
        app.breaking_mode = InputMode::Normal;
    } else {
        app.description = if content.is_empty() {
            None
        } else {
            Some(content)
        };
        app.desc_mode = InputMode::Normal;
    }
    app.go_next();
}

fn handle_textarea(app: &mut App, key: KeyEvent, is_breaking: bool) {
    let mode = if is_breaking {
        app.breaking_mode
    } else {
        app.desc_mode
    };

    // Ctrl+D to finish (works in both modes)
    if key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::CONTROL) {
        textarea_finish(app, is_breaking);
        return;
    }

    match mode {
        InputMode::Normal => {
            match key.code {
                KeyCode::Char('i') => {
                    if is_breaking {
                        app.breaking_mode = InputMode::Insert;
                    } else {
                        app.desc_mode = InputMode::Insert;
                    }
                }
                KeyCode::Char('a') => {
                    // Append: enter insert mode and move cursor right
                    let textarea = if is_breaking {
                        app.breaking_mode = InputMode::Insert;
                        &mut app.breaking_textarea
                    } else {
                        app.desc_mode = InputMode::Insert;
                        &mut app.desc_textarea
                    };
                    textarea.move_cursor(tui_textarea::CursorMove::Forward);
                }
                KeyCode::Char('o') => {
                    // Open line below: enter insert mode with new line
                    let textarea = if is_breaking {
                        app.breaking_mode = InputMode::Insert;
                        &mut app.breaking_textarea
                    } else {
                        app.desc_mode = InputMode::Insert;
                        &mut app.desc_textarea
                    };
                    textarea.move_cursor(tui_textarea::CursorMove::End);
                    textarea.insert_newline();
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    let textarea = if is_breaking {
                        &mut app.breaking_textarea
                    } else {
                        &mut app.desc_textarea
                    };
                    textarea.move_cursor(tui_textarea::CursorMove::Back);
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    let textarea = if is_breaking {
                        &mut app.breaking_textarea
                    } else {
                        &mut app.desc_textarea
                    };
                    textarea.move_cursor(tui_textarea::CursorMove::Forward);
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    let textarea = if is_breaking {
                        &mut app.breaking_textarea
                    } else {
                        &mut app.desc_textarea
                    };
                    textarea.move_cursor(tui_textarea::CursorMove::Down);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let textarea = if is_breaking {
                        &mut app.breaking_textarea
                    } else {
                        &mut app.desc_textarea
                    };
                    textarea.move_cursor(tui_textarea::CursorMove::Up);
                }
                KeyCode::Char('0') => {
                    let textarea = if is_breaking {
                        &mut app.breaking_textarea
                    } else {
                        &mut app.desc_textarea
                    };
                    textarea.move_cursor(tui_textarea::CursorMove::Head);
                }
                KeyCode::Char('$') => {
                    let textarea = if is_breaking {
                        &mut app.breaking_textarea
                    } else {
                        &mut app.desc_textarea
                    };
                    textarea.move_cursor(tui_textarea::CursorMove::End);
                }
                KeyCode::Char('w') => {
                    let textarea = if is_breaking {
                        &mut app.breaking_textarea
                    } else {
                        &mut app.desc_textarea
                    };
                    textarea.move_cursor(tui_textarea::CursorMove::WordForward);
                }
                KeyCode::Char('b') => {
                    let textarea = if is_breaking {
                        &mut app.breaking_textarea
                    } else {
                        &mut app.desc_textarea
                    };
                    textarea.move_cursor(tui_textarea::CursorMove::WordBack);
                }
                KeyCode::Enter => {
                    textarea_finish(app, is_breaking);
                }
                KeyCode::Backspace => {
                    app.go_back();
                }
                _ => {}
            }
        }
        InputMode::Insert => {
            // Esc -> back to Normal mode
            if key.code == KeyCode::Esc {
                if is_breaking {
                    app.breaking_mode = InputMode::Normal;
                } else {
                    app.desc_mode = InputMode::Normal;
                }
                return;
            }

            // Forward everything else to textarea
            let textarea = if is_breaking {
                &mut app.breaking_textarea
            } else {
                &mut app.desc_textarea
            };
            textarea.input(key);
        }
    }
}

fn handle_confirm(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            app.committed = true;
            app.running = false;
        }
        KeyCode::Backspace => {
            app.go_back();
        }
        _ => {}
    }
}
