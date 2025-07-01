use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Editor};

pub fn select(theme: &ColorfulTheme) -> Option<String> {
    let breaking = Confirm::with_theme(theme)
        .with_prompt("Are there any breaking changes?")
        .default(false)
        .show_default(true)
        .interact()
        .unwrap();

    let mut breaking_changes = None;
    if breaking {
        breaking_changes = Editor::new().edit("BREAKING CHANGE:").unwrap();
    }

    if breaking_changes.is_some() {
        Some(breaking_changes.unwrap())
    } else {
        None
    }
}
