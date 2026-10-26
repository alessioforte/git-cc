# Repository guidance

## Project

`git-cc` is a Rust 2021 binary that guides users through composing a conventional
commit with an optional Gitmoji shortcode, then runs `git commit -F`. The interface
uses Ratatui, Crossterm, and tui-textarea; list filtering uses fuzzy-matcher.
Settings are serialized as JSON with Serde. There is no library target or workspace.

## Code map

- `src/main.rs`: terminal setup, panic cleanup, event loop, terminal restoration,
  and commit execution after confirmation.
- `src/app.rs`: `App`, the seven-step `Step` flow, Normal/Insert modes,
  selection/filter state, and navigation.
- `src/handler.rs`: keyboard handling, field validation, and scope persistence.
- `src/event.rs`: terminal event polling.
- `src/formatters.rs`: commit message construction, shared by preview and execution.
- `src/cmd.rs`: temporary message file creation, Git subprocess, and file cleanup.
- `src/settings.rs`: Git directory discovery and saved scopes.
- `src/data/`: static commit kinds and emoji entries; emoji display glyphs and
  stored shortcodes are separate fields.
- `src/ui/`: layout, breadcrumbs, selection lists, text input, and commit preview.
  Reuse styles from `src/ui/theme.rs`.

## Development commands

Run these from the repository root. Use a Rust toolchain with Cargo, rustfmt, and
Clippy; no toolchain version or minimum Rust version is currently pinned.

```sh
cargo build --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets
cargo test --locked
```

For Rust changes, run the formatting, lint, and test checks above. Report existing
failures separately and avoid unrelated formatting churn. Unit tests live beside
the code in `#[cfg(test)]` modules; no CI workflows are currently checked in.
Documentation-only changes need a content and diff review rather than a build.

`cargo run --locked` launches the interactive TUI and requires a real terminal,
Git on `PATH`, and a working directory inside a Git repository. To exercise it in
a disposable repository, build here first and run the resulting `target/debug/git-cc`
binary by absolute path from that repository.

## Implementation conventions

- Keep state and transitions in `app.rs`/`handler.rs`, rendering in `ui/`, message
  formatting in `formatters.rs`, and filesystem/subprocess work in the existing
  boundary modules. Prefer small changes within this structure.
- Follow standard Rust naming and rustfmt formatting. Reuse existing dependencies;
  update the tracked `Cargo.lock` when intentionally changing dependencies.
- Keep preview and submitted messages consistent by using `App::formatted_commit`
  and `formatters::format_commit`. Messages store shortcodes such as `:sparkles:`,
  not the displayed emoji glyph. Breaking-change content adds `!` to the header
  and is appended as supplied; the formatter does not add `BREAKING CHANGE:`.
- Preserve terminal cleanup on cancellation, errors, and panics. Restore the
  terminal before invoking Git or printing final output; avoid adding stdout
  logging during the active TUI.
- When changing key bindings or modes, update the help text in `src/ui/mod.rs`.
  The handler is the source of truth: some existing footers say `Esc: Quit`, but
  `Ctrl+C` cancels; Escape leaves editing/filtering or scope-entry mode and does
  nothing in Normal mode.
- Account for empty filtered lists, small terminal sizes, and Unicode display
  width when changing list or text rendering.

## Runtime behavior and verification

- The flow is Kind → Scope → Emoji → Subject → Description → Breaking → Confirm.
  A subject is required; scope, emoji, description, and breaking text are optional.
  Backspace goes back in Normal mode. Ctrl+D finishes multiline input in either
  mode; Enter at Confirm commits.
- Saved scopes live in `.git/cc/settings.json` as a `scopes` array. Creating a
  saved scope writes immediately, even if the commit is later cancelled; the
  one-use scope option does not persist it.
- `find_git_root()` returns the `.git` directory, despite its name. Discovery
  currently requires a physical `.git` directory and does not handle the `.git`
  files used by linked worktrees and submodules. Use a normal temporary repository
  for filesystem and interactive tests unless changing this behavior.
- Confirmation commits the repository's staged changes; the app does not stage
  files. It writes and then removes `git_commit.txt` in the current directory.
  Use a disposable repository with sample staged files for end-to-end verification
  so testing cannot commit real work or overwrite an existing message file.
- For relevant behavior changes, add focused unit tests near the code with
  `#[cfg(test)]`, especially for formatting, filtering, and navigation. Isolate
  settings and Git tests from the developer's repository; `App::new()` reads real
  settings through the current working directory. Use `App::with_settings` with
  explicit settings for tests that do not need filesystem access.
- For UI changes, manually check the affected steps, back navigation, both input
  modes, empty search results, cancellation, and terminal restoration. Verify the
  preview matches the resulting message when testing confirmation.

Keep generated output and local state out of commits: `target/`, `git_commit.txt`,
`.snippets/`, and `.claude/` are ignored. Recent commits use prefixes such as
`feat:`, `refactor:`, and `style:`; use a concise imperative subject when a commit
is requested.
