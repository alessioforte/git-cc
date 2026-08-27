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

    if let Some(ref desc) = description {
        commit = format!("{}\n\n{}", commit, desc);
    }

    if let Some(ref bc) = breaking_changes {
        commit = format!("{}\n\n{}", commit, bc);
    }

    commit
}
