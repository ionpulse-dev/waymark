//! Git sync: the only place that runs `git` (see `specs/foundation/git-sync`).
//!
//! A caller gets a `Repo`. Writes go through `Repo::write`, which holds the lock, brings the clone
//! up to date, lets the caller change files, then commits and pushes under the right author.

pub mod cmd;
pub mod diff;
pub mod lock;
pub mod log;
pub mod message;
pub mod probe;
mod sync;
mod write;

pub use cmd::Author;
pub use log::{CommitInfo, LogEntry, PathAuthors};
pub use message::{commit_message, parse_subject, Verb};
pub use sync::{conflict_copy, SyncResult};
pub use write::{Change, Tx, WriteResult};

use crate::error::{Error, Result};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

/// A Git working tree that waymark writes to.
pub struct Repo {
    pub(crate) dir: PathBuf,
    author: Mutex<Author>,
    pub(crate) last_fetch: Mutex<Option<Instant>>,
    fetch_failed: std::sync::atomic::AtomicBool,
    last_pull: Mutex<Option<chrono::DateTime<chrono::Utc>>>,
}

impl Repo {
    /// Open a working tree.
    pub fn open(dir: &Path, author: Author) -> Result<Repo> {
        cmd::check_git()?;
        let out = cmd::run(dir, None, &["rev-parse", "--show-toplevel"])?;
        if !out.ok {
            return Err(
                Error::setup(format!("{} is not a Git repository.", dir.display()))
                    .with_next("Pass the folder of a clone with --path, or a URL with --repo."),
            );
        }
        Ok(Repo {
            dir: dir.to_path_buf(),
            author: Mutex::new(author),
            last_fetch: Mutex::new(None),
            fetch_failed: std::sync::atomic::AtomicBool::new(false),
            last_pull: Mutex::new(None),
        })
    }

    /// Clone a remote into a folder, then open it.
    pub fn clone_from(url: &str, dir: &Path, author: Author) -> Result<Repo> {
        cmd::check_git()?;
        if let Some(parent) = dir.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::setup(format!("Cannot create {}: {e}", parent.display())))?;
        }
        let parent = dir.parent().unwrap_or(Path::new("."));
        let target = dir.to_string_lossy().to_string();
        let seconds = std::env::var("WAYMARK_CLONE_TIMEOUT_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300);
        let Some(out) = cmd::run_timeout(parent, &["clone", "--quiet", url, &target], seconds)?
        else {
            return Err(
                Error::sync("The clone took longer than 5 minutes.").with_next(
                    "Clone the repository by hand to the folder of the entry, then start again.",
                ),
            );
        };
        if !out.ok {
            return Err(Error::sync(format!(
                "Cannot clone the repository: {}",
                cmd::first_line(&out.stderr)
            ))
            .with_next("Check the URL and your Git login (an SSH key or a credential helper)."));
        }
        let repo = Repo::open(dir, author)?;
        repo.mark_fetched();
        Ok(repo)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The author of the next commits (see `login-and-identity`: resolved at every write).
    pub fn author(&self) -> Author {
        self.author.lock().unwrap().clone()
    }

    /// Did the last fetch fail? (see `notifications-and-review`)
    pub fn last_fetch_failed(&self) -> bool {
        self.fetch_failed.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// The time of the last good fetch in this session.
    pub fn last_pull(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        *self.last_pull.lock().unwrap()
    }

    pub(crate) fn mark_fetched(&self) {
        *self.last_fetch.lock().unwrap() = Some(Instant::now());
        *self.last_pull.lock().unwrap() = Some(chrono::Utc::now());
    }

    pub(crate) fn set_fetch_failed(&self, failed: bool) {
        self.fetch_failed
            .store(failed, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn set_author(&self, author: Author) {
        *self.author.lock().unwrap() = author;
    }

    /// Does the repository have a remote named `origin`?
    pub fn has_remote(&self) -> bool {
        cmd::run(&self.dir, None, &["remote", "get-url", "origin"])
            .map(|o| o.ok)
            .unwrap_or(false)
    }

    pub(crate) fn git_dir(&self) -> Result<PathBuf> {
        let out = cmd::run_ok(&self.dir, None, &["rev-parse", "--absolute-git-dir"])?;
        Ok(PathBuf::from(out.stdout.trim()))
    }

    /// Is the working tree clean for the given paths? Returns the paths with changes.
    pub fn dirty_paths(&self) -> Result<Vec<String>> {
        let out = cmd::run_ok(
            &self.dir,
            None,
            &["status", "--porcelain", "-z", "--untracked-files=all"],
        )?;
        Ok(out
            .stdout
            .split('\0')
            .filter(|l| l.len() > 3)
            .map(|l| l[3..].to_string())
            .collect())
    }
}
