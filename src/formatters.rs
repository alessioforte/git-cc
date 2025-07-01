pub fn format_commit(
    kind: &str,
    scope: &str,
    emoji: &str,
    subject: &str,
    description: Option<String>,
    breaking_changes: Option<String>,
) -> String {
    let s = if scope.is_empty() {
        "".to_string()
    } else {
        format!("({})", scope)
    };

    let ex = if breaking_changes.is_some() { "!" } else { "" };

    let mut commit = format!("{}{}{}: {} {}", kind, s, ex, emoji, subject);

    if description.is_some() {
        commit = format!("{}\n\n{}", commit, description.as_ref().unwrap());
    }

    if breaking_changes.is_some() {
        commit = format!("{}\n\n{}", commit, breaking_changes.as_ref().unwrap());
    }

    commit
}

pub fn print_commit(commit: &str) {
    println!("");
    let padded = commit.replace("\n", "\n    ");
    println!("    {}", padded);
    println!("");
}
