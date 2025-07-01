use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Editor};

pub fn select(theme: &ColorfulTheme) -> Option<String> {
    let long_description = Confirm::with_theme(theme)
        .with_prompt("Do you want to add a long description?")
        .default(false)
        .show_default(true)
        .interact()
        .unwrap();

    let mut body = None;
    if long_description {
        body = Editor::new().edit("").unwrap();
    }
    if body.is_some() {
        Some(body.unwrap())
    } else {
        None
    }
}
