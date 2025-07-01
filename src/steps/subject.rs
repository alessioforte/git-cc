use dialoguer::theme::ColorfulTheme;

pub fn get_prompt() -> String {
    "Write a short, imperative tense description of the change".to_string()
}

pub fn select(theme: &ColorfulTheme) -> String {
    dialoguer::Input::<String>::with_theme(theme)
        .with_prompt(get_prompt())
        .interact()
        .unwrap()
}
