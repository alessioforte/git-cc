mod formatters;
mod settings;
mod steps;

use crate::steps::{confirm::confirm, emoji, kind, scope, subject};
use dialoguer::theme::ColorfulTheme;

fn main() {
    let theme = ColorfulTheme::default();
    println!("");

    let t = kind::select(&theme);
    let s = scope::select(&theme).unwrap_or("".to_string());
    let e = emoji::select(&theme);
    let sub = subject::select(&theme);
    let body = steps::desc::select(&theme);
    let breaking = steps::breaking::select(&theme);

    let commit = formatters::format_commit(t, &s, &e, &sub, body, breaking);
    formatters::print_commit(&commit);

    if !confirm(&theme) {
        println!("");
        println!("❌ Commit cancelled.");
        return;
    }

    println!("");
    let status = steps::cmd::run_cmd(&commit);
    if !status.success() {
        eprintln!("Git commit failed with status: {}", status);
    } else {
        println!("🚀 Commit successful");
    }
}
