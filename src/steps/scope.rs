use dialoguer::theme::ColorfulTheme;

use crate::settings::get_settings;

fn get_prompt() -> String {
    "Select the scope of this change".to_string()
}

fn get_options() -> Vec<String> {
    let settings = get_settings();
    let mut options = vec!["None".to_string()];

    settings.scopes.iter().for_each(|scope| {
        options.push(scope.clone());
    });

    options.push("New scope".to_string());
    options.push("New scope (only use once)".to_string());
    options
}

fn save_scope(scope: &str) {
    let mut settings = get_settings();
    if !settings.scopes.contains(&scope.to_string()) {
        settings.scopes.push(scope.to_string());
    }
    settings.save();
}

pub fn select(theme: &ColorfulTheme) -> Option<String> {
    let options = get_options();
    let selected_scope = dialoguer::Select::with_theme(theme)
        .with_prompt(get_prompt())
        .items(&options)
        .default(0)
        .interact()
        .unwrap();

    if selected_scope == options.len() - 1 || selected_scope == options.len() - 2 {
        let new_scope = dialoguer::Input::<String>::with_theme(theme)
            .with_prompt("Enter the new scope")
            .interact()
            .unwrap();
        if selected_scope == options.len() - 2 {
            save_scope(&new_scope);
        }
        Some(new_scope)
    } else if selected_scope > 0 {
        Some(options[selected_scope].clone())
    } else {
        None
    }
}
