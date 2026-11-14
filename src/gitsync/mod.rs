//! Git sync: the only place that runs `git` (see `specs/foundation/git-sync`).
//!
//! A caller gets a `Repo`. Writes go through `Repo::write`, which holds the lock, brings the clone
//! up to date, lets the caller change files, then commits and pushes under the right author.

pub mod cmd;
pub mod lock;
pub mod log;
pub mod message;
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
    pub(crate) author: Author,
    pub(crate) last_fetch: Mutex<Option<Instant>>,
}

impl Author {
    /// The author from arguments or variables, or else from the Git settings of the person.
    pub fn resolve(name: Option<&str>, email: Option<&str>, dir: &Path) -> Result<Author> {
        let from_git = |key: &str| {
            cmd::run(dir, None, &["config", "--get", key])
                .ok()
                .filter(|o| o.ok)
                .map(|o| o.stdout.trim().to_string())
                .filter(|s| !s.is_empty())
        };
        let name = name
            .map(String::from)
            .filter(|s| !s.is_empty())
            .or_else(|| from_git("user.name"));
        let email = email
            .map(String::from)
            .filter(|s| !s.is_empty())
            .or_else(|| from_git("user.email"));
        match (name, email) {
            (Some(name), Some(email)) => Ok(Author { name, email }),
            _ => Err(Error::setup("No author is set for the commits.")
                .with_next("Set git config user.name and user.email, or pass --author-name and --author-email.")),
        }
    }
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
            author,
            last_fetch: Mutex::new(None),
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
        let out = cmd::run(parent, None, &["clone", "--quiet", url, &target])?;
        if !out.ok {
            return Err(Error::sync(format!(
                "Cannot clone the repository: {}",
                cmd::first_line(&out.stderr)
            ))
            .with_next("Check the URL and your Git login (an SSH key or a credential helper)."));
        }
        let repo = Repo::open(dir, author)?;
        *repo.last_fetch.lock().unwrap() = Some(Instant::now());
        Ok(repo)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn author(&self) -> &Author {
        &self.author
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
