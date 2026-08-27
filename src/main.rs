mod app;
mod cmd;
mod data;
mod event;
mod formatters;
mod handler;
mod settings;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::Event;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

fn main() -> io::Result<()> {
    // Install panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        disable_raw_mode().ok();
        io::stdout().execute(LeaveAlternateScreen).ok();
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run event loop
    let mut app = app::App::new();

    while app.running {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        if let Some(Event::Key(key)) = event::next_event(Duration::from_millis(50)) {
            handler::handle_key_event(&mut app, key);
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Execute git commit if confirmed
    if app.committed {
        let commit = app.formatted_commit();
        println!();
        let padded = commit.replace('\n', "\n    ");
        println!("    {}", padded);
        println!();

        let status = cmd::run_cmd(&commit);
        if !status.success() {
            eprintln!("Git commit failed with status: {}", status);
            std::process::exit(1);
        } else {
            println!("🚀 Commit successful");
        }
    } else {
        println!();
        println!("❌ Commit cancelled.");
    }

    Ok(())
}
