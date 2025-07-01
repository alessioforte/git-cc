pub fn confirm(theme: &dialoguer::theme::ColorfulTheme) -> bool {
    dialoguer::Confirm::with_theme(theme)
        .with_prompt("Do you want to commit this?")
        .default(true)
        .show_default(true)
        .interact()
        .unwrap()
}
