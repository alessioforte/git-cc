use dialoguer::theme::ColorfulTheme;

fn get_prompt() -> String {
    "Select the type of change that you're committing.".to_string()
}

fn get_options() -> Vec<&'static str> {
    let options = vec![
        "feat      A new feature",
        "fix       A bug fix",
        "docs      Documentation only changes",
        "style     Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)",
        "refactor  A code change that neither fixes a bug nor adds a feature",
        "perf      A code change that improves performance",
        "test      Adding missing tests or correcting existing tests",
        "build     Changes that affect the build system or external dependencies (example scopes: gulp, broccoli, npm)",
        "ci        Changes to our CI configuration files and scripts (example scopes: Travis, Circle, BrowserStack, SauceLabs)",
        "chore     Changes to the build process or auxiliary tools and libraries such as documentation generation",
        "revert    Reverts a previous commit",
        "temp      Temporary commit that won't be included in your CHANGELOG"
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
