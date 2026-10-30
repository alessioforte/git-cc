use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub scopes: Vec<String>,
}

impl Settings {
    pub fn new() -> Self {
        Settings { scopes: vec![] }
    }

    /// Persist the settings inside the repository found from the current directory.
    pub fn save(&self) -> Result<(), Error> {
        let git_dir = find_git_root()?;
        self.save_to(&git_dir)
    }

    fn save_to(&self, git_dir: &Path) -> Result<(), Error> {
        let path = config_path(git_dir);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file_content = serde_json::to_string(self)
            .map_err(|err| Error::Io(io::Error::new(io::ErrorKind::InvalidData, err)))?;

        // Write to a sibling file first so an interrupted save cannot truncate
        // the settings that are already on disk.
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, file_content)?;
        if let Err(err) = std::fs::rename(&tmp_path, &path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(Error::Io(err));
        }
        Ok(())
    }
}

/// Settings read from disk plus an optional warning for the UI.
pub struct Loaded {
    pub settings: Settings,
    pub notice: Option<String>,
}

/// Loads the settings for the repository found from the current directory.
pub fn load() -> Result<Loaded, Error> {
    let git_dir = find_git_root()?;
    Ok(load_from(&git_dir))
}

/// Reads the settings stored in `git_dir`, recovering from invalid files.
fn load_from(git_dir: &Path) -> Loaded {
    let path = config_path(git_dir);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Loaded {
                settings: Settings::new(),
                notice: None,
            };
        }
        Err(err) => {
            return Loaded {
                settings: Settings::new(),
                notice: Some(format!(
                    "could not read settings.json ({err}); using defaults"
                )),
            };
        }
    };

    match serde_json::from_str(&content) {
        Ok(settings) => Loaded {
            settings,
            notice: None,
        },
        Err(err) => {
            // Keep the broken file around instead of silently overwriting it
            // with defaults on the next save.
            let backup = path.with_extension("json.bak");
            let notice = match std::fs::rename(&path, &backup) {
                Ok(()) => {
                    format!("settings.json was invalid ({err}); moved it to settings.json.bak")
                }
                Err(_) => format!("settings.json was invalid ({err}); using defaults"),
            };
            Loaded {
                settings: Settings::new(),
                notice: Some(notice),
            }
        }
    }
}

/// Adds `scope` to the saved scopes, ignoring duplicates.
pub fn save_scope(scope: &str) -> Result<(), Error> {
    let mut loaded = load()?;
    if !loaded
        .settings
        .scopes
        .iter()
        .any(|existing| existing == scope)
    {
        loaded.settings.scopes.push(scope.to_string());
        loaded.settings.save()?;
    }
    Ok(())
}

/// Finds the Git directory of the repository containing the current directory.
///
/// Delegating to `git rev-parse` keeps linked worktrees and submodules working,
/// where `.git` is a file pointing at another directory.
pub fn find_git_root() -> Result<PathBuf, Error> {
    let dir = std::env::current_dir().map_err(Error::Io)?;
    find_git_root_in(&dir)
}

fn find_git_root_in(dir: &Path) -> Result<PathBuf, Error> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--absolute-git-dir"])
        .output()
        .map_err(|err| {
            Error::Io(io::Error::new(
                err.kind(),
                format!("could not run git: {err}"),
            ))
        })?;

    if !output.status.success() {
        return Err(Error::NotAGitRepository);
    }

    let path = String::from_utf8(output.stdout)
        .map_err(|err| Error::Io(io::Error::new(io::ErrorKind::InvalidData, err)))?;
    let path = path.trim_end_matches(['\r', '\n']);
    if path.is_empty() {
        return Err(Error::NotAGitRepository);
    }
    Ok(PathBuf::from(path))
}

fn config_path(git_dir: &Path) -> PathBuf {
    git_dir.join("cc").join("settings.json")
}

#[derive(Debug)]
pub enum Error {
    NotAGitRepository,
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotAGitRepository => {
                write!(
                    f,
                    "not inside a Git repository (git must be installed and on PATH)"
                )
            }
            Error::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "git-cc-test-{label}-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).unwrap();
            TempDir(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn git(dir: &Path, args: &[&str]) -> bool {
        std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .unwrap()
            .success()
    }

    #[test]
    fn missing_settings_fall_back_to_defaults_without_a_notice() {
        let dir = TempDir::new("missing");
        let loaded = load_from(dir.path());
        assert!(loaded.settings.scopes.is_empty());
        assert!(loaded.notice.is_none());
    }

    #[test]
    fn saved_settings_round_trip_and_leave_no_temporary_file() {
        let dir = TempDir::new("round-trip");
        let settings = Settings {
            scopes: vec!["api".to_string(), "ui".to_string()],
        };

        settings.save_to(dir.path()).unwrap();

        let loaded = load_from(dir.path());
        assert_eq!(loaded.settings.scopes, ["api", "ui"]);
        assert!(loaded.notice.is_none());
        assert!(!config_path(dir.path()).with_extension("json.tmp").exists());
    }

    #[test]
    fn invalid_settings_are_backed_up_instead_of_discarded() {
        let dir = TempDir::new("invalid");
        let path = config_path(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "{ definitely not json").unwrap();

        let loaded = load_from(dir.path());

        assert!(loaded.settings.scopes.is_empty());
        let notice = loaded
            .notice
            .expect("invalid settings should warn the user");
        assert!(
            notice.contains("settings.json.bak"),
            "unexpected notice: {notice}"
        );
        assert!(path.with_extension("json.bak").exists());
        assert!(!path.exists());
    }

    #[test]
    fn git_discovery_handles_repositories_and_plain_directories() {
        let repo = TempDir::new("repo");
        assert!(git(repo.path(), &["init", "-q"]));

        let found = find_git_root_in(repo.path()).unwrap();
        assert_eq!(
            std::fs::canonicalize(found).unwrap(),
            std::fs::canonicalize(repo.path().join(".git")).unwrap()
        );

        let plain = TempDir::new("plain");
        assert!(matches!(
            find_git_root_in(plain.path()),
            Err(Error::NotAGitRepository)
        ));
    }

    #[test]
    fn git_discovery_follows_git_file_indirection() {
        // Linked worktrees and submodules use a `.git` file that points at the
        // real Git directory instead of a `.git` directory of their own.
        let repo = TempDir::new("link-target");
        assert!(git(repo.path(), &["init", "-q"]));

        let linked = TempDir::new("link");
        std::fs::write(
            linked.path().join(".git"),
            format!("gitdir: {}\n", repo.path().join(".git").display()),
        )
        .unwrap();

        let found = find_git_root_in(linked.path()).unwrap();
        assert_eq!(
            std::fs::canonicalize(found).unwrap(),
            std::fs::canonicalize(repo.path().join(".git")).unwrap()
        );
    }
}
