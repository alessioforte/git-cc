pub struct KindEntry {
    pub value: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

pub const KINDS: [KindEntry; 12] = [
    KindEntry {
        value: "feat",
        label: "feat",
        description: "A new feature",
    },
    KindEntry {
        value: "fix",
        label: "fix",
        description: "A bug fix",
    },
    KindEntry {
        value: "docs",
        label: "docs",
        description: "Documentation only changes",
    },
    KindEntry {
        value: "style",
        label: "style",
        description: "Changes that do not affect the meaning of the code",
    },
    KindEntry {
        value: "refactor",
        label: "refactor",
        description: "A code change that neither fixes a bug nor adds a feature",
    },
    KindEntry {
        value: "perf",
        label: "perf",
        description: "A code change that improves performance",
    },
    KindEntry {
        value: "test",
        label: "test",
        description: "Adding missing tests or correcting existing tests",
    },
    KindEntry {
        value: "build",
        label: "build",
        description: "Changes that affect the build system or external dependencies",
    },
    KindEntry {
        value: "ci",
        label: "ci",
        description: "Changes to CI configuration files and scripts",
    },
    KindEntry {
        value: "chore",
        label: "chore",
        description: "Changes to the build process or auxiliary tools",
    },
    KindEntry {
        value: "revert",
        label: "revert",
        description: "Reverts a previous commit",
    },
    KindEntry {
        value: "temp",
        label: "temp",
        description: "Temporary commit that won't be included in your CHANGELOG",
    },
];
