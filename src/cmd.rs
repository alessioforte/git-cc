use std::io::{self, Write};
use std::process::{Command, ExitStatus, Stdio};

/// Commits the staged changes with `message` provided on stdin.
///
/// Passing the message through stdin avoids writing a temporary file in the
/// working tree, so an existing `git_commit.txt` can no longer be overwritten
/// and failed runs cannot leave files behind.
pub fn commit(message: &str) -> io::Result<ExitStatus> {
    let mut child = Command::new("git")
        .args(["commit", "-F", "-"])
        .stdin(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().ok_or_else(|| {
        io::Error::new(io::ErrorKind::BrokenPipe, "git commit stdin was not piped")
    })?;
    let write_result = stdin.write_all(message.as_bytes());
    drop(stdin);

    let status = child.wait()?;

    match write_result {
        // Git closes the pipe early when it aborts (for example on a failing
        // hook); its exit status then carries the useful information.
        Err(err) if err.kind() != io::ErrorKind::BrokenPipe => Err(err),
        _ => Ok(status),
    }
}
