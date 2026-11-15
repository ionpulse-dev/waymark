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

/// The transports that waymark lets `git` use. This excludes `ext::`, which runs a command.
const ALLOWED_PROTOCOLS: &str = "file:git:http:https:ssh";

/// A source of a short HTTPS credential for one Git host (see `personal-server`). The token goes
/// to `git` only through the environment of the child process. It is never on a command line or
/// on disk.
pub trait CredentialProvider: Send + Sync {
    /// The current token, or `None` if none can be made.
    fn token(&self) -> Option<String>;
    /// Forget the current token, so that the next call makes a new one.
    fn invalidate(&self);
}

static CREDENTIAL: std::sync::OnceLock<(String, Box<dyn CredentialProvider>)> =
    std::sync::OnceLock::new();

/// Use a credential for every `git` call to the URLs that start with `url_prefix`
/// (for example `https://github.com/`). It can be set once per process.
pub fn set_credential(url_prefix: String, provider: Box<dyn CredentialProvider>) {
    let _ = CREDENTIAL.set((url_prefix, provider));
}

fn apply_credential(cmd: &mut Command) {
    use base64::Engine;
    if let Some((prefix, provider)) = CREDENTIAL.get() {
        if let Some(token) = provider.token() {
            let basic =
                base64::engine::general_purpose::STANDARD.encode(format!("x-access-token:{token}"));
            cmd.env("GIT_CONFIG_COUNT", "1")
                .env("GIT_CONFIG_KEY_0", format!("http.{prefix}.extraheader"))
                .env(
                    "GIT_CONFIG_VALUE_0",
                    format!("Authorization: Basic {basic}"),
                );
        }
    }
}

/// Did `git` fail because the credential was refused? Then a new token may help.
fn is_auth_failure(stderr: &str) -> bool {
    // With a refused header, git asks for a user name next. It cannot, and says so.
    [
        "Authentication failed",
        "error: 401",
        "returned error: 401",
        "could not read Username",
    ]
    .iter()
    .any(|m| stderr.contains(m))
}

/// After a refused credential: drop the token. True if there was a credential to drop.
fn retry_with_new_token(out: &Output) -> bool {
    match CREDENTIAL.get() {
        Some((_, p)) if !out.ok && is_auth_failure(&out.stderr) => {
            p.invalidate();
            true
        }
        _ => false,
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

/// Run git in a folder. Never prompts. Returns the output also when git fails. If a credential
/// is set and git says that it was refused, it runs once more with a new token.
pub fn run(dir: &Path, author: Option<&Author>, args: &[&str]) -> Result<Output> {
    let out = run_once(dir, author, args)?;
    if retry_with_new_token(&out) {
        return run_once(dir, author, args);
    }
    Ok(out)
}

fn run_once(dir: &Path, author: Option<&Author>, args: &[&str]) -> Result<Output> {
    let mut cmd = Command::new("git");
    apply_credential(&mut cmd);
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
        .env("GIT_ALLOW_PROTOCOL", ALLOWED_PROTOCOLS)
        .env_remove("GIT_ASKPASS");
    if std::env::var_os("GIT_SSH_COMMAND").is_none() {
        cmd.env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes");
    }
    let out = cmd.output().map_err(|e| {
        Error::setup(format!("Cannot run git: {e}")).with_next(crate::platform::git_install_hint())
    })?;
    Ok(to_output(out))
}

/// Like `run`, but kills git after `seconds` and returns `None` then.
pub fn run_timeout(dir: &Path, args: &[&str], seconds: u64) -> Result<Option<Output>> {
    let out = run_timeout_once(dir, args, seconds)?;
    if out.as_ref().is_some_and(retry_with_new_token) {
        return run_timeout_once(dir, args, seconds);
    }
    Ok(out)
}

fn run_timeout_once(dir: &Path, args: &[&str], seconds: u64) -> Result<Option<Output>> {
    use std::io::Read;
    use std::process::Stdio;
    let mut cmd = Command::new("git");
    apply_credential(&mut cmd);
    cmd.arg("-C").arg(dir).args(args);
    cmd.env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .env("GIT_ALLOW_PROTOCOL", ALLOWED_PROTOCOLS)
        .env_remove("GIT_ASKPASS")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if std::env::var_os("GIT_SSH_COMMAND").is_none() {
        cmd.env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes");
    }
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = cmd.spawn().map_err(|e| {
        Error::setup(format!("Cannot run git: {e}")).with_next(crate::platform::git_install_hint())
    })?;
    let pid = child.id();
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
            // Kill the whole group: git may have started `ssh`, which holds the pipes.
            // The `--` is needed: without it, `kill` of procps (Linux) reads `-<pid>` as an
            // option and can kill every process of the user.
            let _ = std::process::Command::new("kill")
                .args(["-KILL", "--", &format!("-{pid}")])
                .stderr(Stdio::null())
                .status();
            let _ = child.kill();
            let _ = child.wait();
            return Ok(None);
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    // A grandchild may keep a pipe open after git exits. Wait a short time for the readers.
    let (tx, rx) = std::sync::mpsc::channel();
    let tx2 = tx.clone();
    std::thread::spawn(move || {
        let _ = tx.send((0, t_out.join().unwrap_or_default()));
    });
    std::thread::spawn(move || {
        let _ = tx2.send((1, t_err.join().unwrap_or_default()));
    });
    let (mut stdout, mut stderr) = (String::new(), String::new());
    for _ in 0..2 {
        match rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok((0, s)) => stdout = s,
            Ok((_, s)) => stderr = s,
            Err(_) => break,
        }
    }
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
        Error::setup(format!("Cannot run git: {e}")).with_next(crate::platform::git_install_hint())
    })?;
    if !out.status.success() {
        return Err(Error::setup("git does not work on this machine.")
            .with_next(crate::platform::git_install_hint()));
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
