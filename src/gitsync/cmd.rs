//! Run the `git` binary with an isolated, non-interactive environment.

use crate::error::{Error, Result};
use std::path::Path;
use std::process::{Command, Output as ProcOutput};

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub ok: bool,
    pub code: Option<i32>,
}

fn to_output(o: ProcOutput) -> Output {
    Output {
        stdout: String::from_utf8_lossy(&o.stdout).to_string(),
        stderr: String::from_utf8_lossy(&o.stderr).to_string(),
        ok: o.status.success(),
        code: o.status.code(),
    }
}

/// The first useful line of git's error text.
pub fn first_line(stderr: &str) -> String {
    stderr
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with("hint:"))
        .unwrap_or("git failed")
        .to_string()
}

/// Run git in a folder. Never prompts. Returns the output also when git fails.
pub fn run(dir: &Path, author: Option<&Author>, args: &[&str]) -> Result<Output> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(dir);
    if let Some(a) = author {
        cmd.arg("-c").arg(format!("user.name={}", a.name));
        cmd.arg("-c").arg(format!("user.email={}", a.email));
        cmd.env("GIT_AUTHOR_NAME", &a.name)
            .env("GIT_AUTHOR_EMAIL", &a.email)
            .env("GIT_COMMITTER_NAME", &a.name)
            .env("GIT_COMMITTER_EMAIL", &a.email);
    }
    cmd.arg("-c").arg("core.quotepath=false");
    cmd.args(args);
    cmd.env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .env_remove("GIT_ASKPASS");
    if std::env::var_os("GIT_SSH_COMMAND").is_none() {
        cmd.env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes");
    }
    let out = cmd.output().map_err(|e| {
        Error::setup(format!("Cannot run git: {e}"))
            .with_next("Install git, for example with: xcode-select --install")
    })?;
    Ok(to_output(out))
}

/// Like `run`, but kills git after `seconds` and returns `None` then.
pub fn run_timeout(dir: &Path, args: &[&str], seconds: u64) -> Result<Option<Output>> {
    use std::io::Read;
    use std::process::Stdio;
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(dir).args(args);
    cmd.env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .env_remove("GIT_ASKPASS")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if std::env::var_os("GIT_SSH_COMMAND").is_none() {
        cmd.env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes");
    }
    let mut child = cmd.spawn().map_err(|e| {
        Error::setup(format!("Cannot run git: {e}"))
            .with_next("Install git, for example with: xcode-select --install")
    })?;
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let t_out = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stdout.read_to_string(&mut s);
        s
    });
    let t_err = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    let status = loop {
        if let Some(s) = child
            .try_wait()
            .map_err(|e| Error::sync(format!("git: {e}")))?
        {
            break Some(s);
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    let stdout = t_out.join().unwrap_or_default();
    let stderr = t_err.join().unwrap_or_default();
    Ok(status.map(|s| Output {
        stdout,
        stderr,
        ok: s.success(),
        code: s.code(),
    }))
}

/// Run git and fail with `sync_failed` if it fails.
pub fn run_ok(dir: &Path, author: Option<&Author>, args: &[&str]) -> Result<Output> {
    let out = run(dir, author, args)?;
    if out.ok {
        Ok(out)
    } else {
        Err(Error::sync(format!(
            "git {} failed: {}",
            args.first().copied().unwrap_or(""),
            first_line(&out.stderr)
        )))
    }
}

/// Check that a working `git` exists. `/usr/bin/git` can be a stub that asks to install tools.
pub fn check_git() -> Result<String> {
    let out = Command::new("git").arg("--version").output().map_err(|e| {
        Error::setup(format!("Cannot run git: {e}"))
            .with_next("Install git, for example with: xcode-select --install")
    })?;
    if !out.status.success() {
        return Err(Error::setup("git does not work on this machine.")
            .with_next("Install the Xcode command line tools: xcode-select --install"));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_works_here() {
        assert!(check_git().unwrap().starts_with("git version"));
    }

    #[test]
    fn first_line_skips_hints() {
        assert_eq!(
            first_line("hint: do this\nfatal: bad thing\n"),
            "fatal: bad thing"
        );
        assert_eq!(first_line(""), "git failed");
    }

    #[test]
    fn a_failing_command_is_reported_with_its_output() {
        let dir = tempfile::tempdir().unwrap();
        let out = run(dir.path(), None, &["rev-parse", "--git-dir"]).unwrap();
        assert!(!out.ok);
        let err = run_ok(dir.path(), None, &["rev-parse", "--git-dir"]).unwrap_err();
        assert_eq!(err.code, crate::error::Code::SyncFailed);
    }
}
