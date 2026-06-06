use ratatui::style::{Color, Modifier, Style};

pub const PRIMARY: Color = Color::Cyan;
pub const SUCCESS: Color = Color::Green;
pub const DANGER: Color = Color::Red;
pub const MUTED: Color = Color::DarkGray;
pub const FILTER_COLOR: Color = Color::Yellow;

pub fn title_style() -> Style {
    Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default().bg(PRIMARY).fg(Color::Black)
}

pub fn dimmed_style() -> Style {
    Style::default().fg(MUTED)
}

pub fn filter_style() -> Style {
    Style::default().fg(FILTER_COLOR)
}

pub fn breadcrumb_current() -> Style {
    Style::default()
        .fg(PRIMARY)
        .add_modifier(Modifier::BOLD)
}

pub fn breadcrumb_done() -> Style {
    Style::default().fg(SUCCESS)
}

pub fn breadcrumb_future() -> Style {
    Style::default().fg(MUTED)
}

pub fn help_style() -> Style {
    Style::default().fg(MUTED)
}

pub fn error_style() -> Style {
    Style::default().fg(DANGER).add_modifier(Modifier::BOLD)
}

pub fn border_style() -> Style {
    Style::default().fg(PRIMARY)
}
