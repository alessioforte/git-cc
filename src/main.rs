mod app;
mod cmd;
mod data;
mod event;
mod formatters;
mod handler;
mod settings;
mod ui;

use std::io::{self, Stdout};
use std::process::ExitStatus;
use std::time::Duration;

use crossterm::event::Event;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

type Tui = Terminal<CrosstermBackend<Stdout>>;

fn main() -> io::Result<()> {
    // Install panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        disable_raw_mode().ok();
        io::stdout().execute(LeaveAlternateScreen).ok();
        original_hook(panic_info);
    }));

    // Resolve the settings before touching the terminal so a missing repository
    // or missing git produces a plain error message.
    let mut app = match app::App::new() {
        Ok(app) => app,
        Err(err) => {
            eprintln!("git-cc: {err}");
            std::process::exit(1);
        }
    };

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    loop {
        enter_tui(&mut terminal)?;
        run_tui(&mut terminal, &mut app)?;
        leave_tui(&mut terminal)?;

        if !app.committed {
            println!();
            println!("❌ Commit cancelled.");
            return Ok(());
        }

        app.committed = false;
        let commit = app.formatted_commit();
        println!();
        println!("    {}", commit.replace('\n', "\n    "));
        println!();

        match cmd::commit(&commit) {
            Ok(status) if status.success() => {
                println!("🚀 Commit successful");
                return Ok(());
            }
            Ok(status) => {
                app.commit_error = Some(commit_failed_notice(status));
            }
            Err(err) => {
                app.commit_error = Some(format!("Could not run git: {err}"));
            }
        }

        // Keep the TUI (and the composed message) alive after a failure so the
        // user can fix the issue and retry, or go back and edit.
        app.running = true;
    }
}

fn run_tui(terminal: &mut Tui, app: &mut app::App<'_>) -> io::Result<()> {
    while app.running {
        terminal.draw(|frame| ui::render(frame, app))?;

        if let Some(Event::Key(key)) = event::next_event(Duration::from_millis(50)) {
            handler::handle_key_event(app, key);
        }
    }
    Ok(())
}

fn enter_tui(terminal: &mut Tui) -> io::Result<()> {
    enable_raw_mode()?;
    terminal.backend_mut().execute(EnterAlternateScreen)?;
    terminal.clear()
}

fn leave_tui(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn commit_failed_notice(status: ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("Commit failed (exit code {code}) — Enter: retry | Backspace: edit"),
        None => {
            "Commit failed (terminated by a signal) — Enter: retry | Backspace: edit".to_string()
        }
    }
}
