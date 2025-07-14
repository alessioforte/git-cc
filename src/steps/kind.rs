use colored::Colorize;
use dialoguer::theme::ColorfulTheme;

fn get_prompt() -> String {
    "Select the type of change that you're committing.".to_string()
}

fn get_options() -> Vec<String> {
    let options = vec![
        format!("{}      {}", "feat".bold(), "A new feature".dimmed()),
        format!("{}       {}", "fix".bold(),       "A bug fix".dimmed()),
        format!("{}      {}", "docs".bold(),      "Documentation only changes".dimmed()),
        format!("{}     {}", "style".bold(),     "Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)".dimmed()),
        format!("{}  {}", "refactor".bold(),  "A code change that neither fixes a bug nor adds a feature".dimmed()),
        format!("{}      {}", "perf".bold(),      "A code change that improves performance".dimmed()),
        format!("{}      {}", "test".bold(),      "Adding missing tests or correcting existing tests".dimmed()),
        format!("{}     {}", "build".bold(),     "Changes that affect the build system or external dependencies (example scopes: gulp, broccoli, npm)".dimmed()),
        format!("{}        {}", "ci".bold(),        "Changes to our CI configuration files and scripts (example scopes: Travis, Circle, BrowserStack, SauceLabs)".dimmed()),
        format!("{}     {}", "chore".bold(),     "Changes to the build process or auxiliary tools and libraries such as documentation generation".dimmed()),
        format!("{}    {}", "revert".bold(),    "Reverts a previous commit".dimmed()),
        format!("{}      {}", "temp".bold(),      "Temporary commit that won't be included in your CHANGELOG".dimmed()),
    ];

    options
}

fn get_value(index: usize) -> &'static str {
    let values = vec![
        "feat", "fix", "docs", "style", "refactor", "perf", "test", "build", "ci", "chore",
        "revert", "temp",
    ];
    values[index]
}

pub fn select(theme: &ColorfulTheme) -> &'static str {
    let options = get_options();
    let selected = dialoguer::Select::with_theme(theme)
        .with_prompt(get_prompt())
        .items(&options)
        .default(0)
        .interact()
        .unwrap();

    get_value(selected)
}
