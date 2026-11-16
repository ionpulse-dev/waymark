//! The write transaction: lock, update, change files, commit, push.

use super::cmd::{self, run_ok};
use super::lock::Lock;
use super::Repo;
use crate::error::{Error, Result};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// What a write changes: the paths to commit (below the repository folder) and the message.
#[derive(Debug, Clone)]
pub struct Change {
    pub paths: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct WriteResult {
    /// The short hash of the new commit. `None` for a preview or if nothing changed.
    pub commit: Option<String>,
    pub pushed: bool,
    pub preview: bool,
    /// The write changed nothing, because the files were already as asked.
    pub unchanged: bool,
    pub notices: Vec<String>,
}

enum Undo {
    /// A file that did not exist. Remove it.
    Created(PathBuf),
    /// A file that existed. Put the old bytes back.
    Restore(PathBuf, Vec<u8>),
}

/// The working tree during a write. Every change is remembered, so that it can be rolled back.
pub struct Tx<'a> {
    repo: &'a Repo,
    undo: RefCell<Vec<Undo>>,
}

impl<'a> Tx<'a> {
    fn new(repo: &'a Repo) -> Tx<'a> {
        Tx {
            repo,
            undo: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn new_for_sync(repo: &'a Repo) -> Tx<'a> {
        Tx::new(repo)
    }

    pub fn repo(&self) -> &Repo {
        self.repo
    }

    /// Forget the changes so far. They are committed, so a rollback must not put the old bytes
    /// back.
    pub(crate) fn forget(&self) {
        self.undo.borrow_mut().clear();
    }

    fn abs(&self, rel: &str) -> PathBuf {
        self.repo.dir.join(rel)
    }

    /// The text of a file, or `None` if it does not exist.
    pub fn read(&self, rel: &str) -> Result<Option<String>> {
        match fs::read(self.abs(rel)) {
            Ok(b) => Ok(Some(String::from_utf8_lossy(&b).to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::setup(format!("Cannot read {rel}: {e}"))),
        }
    }

    fn remember(&self, rel: &str) {
        let p = self.abs(rel);
        match fs::read(&p) {
            Ok(bytes) => self.undo.borrow_mut().push(Undo::Restore(p, bytes)),
            Err(_) => self.undo.borrow_mut().push(Undo::Created(p)),
        }
    }

    /// Write a file. Creates folders as needed.
    pub fn write_file(&self, rel: &str, content: &str) -> Result<()> {
        self.remember(rel);
        let p = self.abs(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::setup(format!("Cannot create {}: {e}", parent.display())))?;
        }
        fs::write(&p, content).map_err(|e| Error::setup(format!("Cannot write {rel}: {e}")))
    }

    /// Remove a file.
    pub fn remove_file(&self, rel: &str) -> Result<()> {
        self.remember(rel);
        fs::remove_file(self.abs(rel))
            .map_err(|e| Error::setup(format!("Cannot remove {rel}: {e}")))?;
        // Git keeps no empty folder, so the disk must not keep one either.
        self.remove_empty_parents(&self.abs(rel));
        Ok(())
    }

    /// Move a file. Git sees a rename when the content is the same.
    pub fn rename(&self, from: &str, to: &str) -> Result<()> {
        let bytes = fs::read(self.abs(from))
            .map_err(|e| Error::setup(format!("Cannot read {from}: {e}")))?;
        self.remember(from);
        self.remember(to);
        let dest = self.abs(to);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::setup(format!("Cannot create {}: {e}", parent.display())))?;
        }
        fs::write(&dest, bytes).map_err(|e| Error::setup(format!("Cannot write {to}: {e}")))?;
        fs::remove_file(self.abs(from))
            .map_err(|e| Error::setup(format!("Cannot remove {from}: {e}")))?;
        self.remove_empty_parents(&self.abs(from));
        Ok(())
    }

    /// The short hash of the last commit of a path. Used as the `version` of a document.
    pub fn last_commit(&self, rel: &str) -> Result<Option<String>> {
        self.repo.last_commit(rel)
    }

    /// Check that a document did not change since the caller read it. `base` is the `version`
    /// that `show` returned. A different version is `stale_base`.
    pub fn check_base(&self, rel: &str, base: Option<&str>) -> Result<()> {
        let now = self.repo.last_commit(rel)?;
        match (now.as_deref(), base) {
            (Some(n), Some(b)) if n != b => Err(Error::new(
                crate::error::Code::StaleBase,
                format!("The document changed since you read it (version {n}, you have {b})."),
            )
            .with_next("Read the document again with show, then redo the change.")),
            _ => Ok(()),
        }
    }

    /// Remove the empty folders above a path, up to the repository folder.
    fn remove_empty_parents(&self, path: &std::path::Path) {
        let mut cur = path.parent();
        while let Some(dir) = cur {
            if dir == self.repo.dir || !dir.starts_with(&self.repo.dir) {
                break;
            }
            if fs::remove_dir(dir).is_err() {
                break;
            }
            cur = dir.parent();
        }
    }

    fn rollback(&self) {
        let mut undo = self.undo.borrow_mut();
        while let Some(u) = undo.pop() {
            match u {
                Undo::Created(p) => {
                    let _ = fs::remove_file(&p);
                    self.remove_empty_parents(&p);
                }
                Undo::Restore(p, bytes) => {
                    if let Some(parent) = p.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    let _ = fs::write(p, bytes);
                }
            }
        }
    }
}

const LOCK_WAIT: Duration = Duration::from_secs(30);

impl Repo {
    /// Run a write. Steps: take the lock, fetch and fast-forward, run the closure (checks and file
    /// changes), then commit and push. If the closure fails, or `preview` is true, the files are
    /// restored and nothing is committed.
    ///
    /// The closure does the checks of the write flow: base check, validation, approval.
    pub fn write<T>(
        &self,
        preview: bool,
        f: impl FnOnce(&Tx) -> Result<(Change, T)>,
    ) -> Result<(WriteResult, T)> {
        let _lock = Lock::acquire(&self.git_dir()?, LOCK_WAIT)?;
        let mut result = WriteResult {
            preview,
            ..Default::default()
        };
        let update_notices = self.update(true)?;
        result.notices.extend(update_notices);

        let tx = Tx::new(self);
        let (change, value) = match f(&tx) {
            Ok(v) => v,
            Err(e) => {
                tx.rollback();
                return Err(e);
            }
        };
        if preview {
            tx.rollback();
            return Ok((result, value));
        }
        match self.commit_change(&change) {
            Ok(Some(hash)) => result.commit = Some(hash),
            Ok(None) => {
                result.unchanged = true;
                // The closure may have made commits of its own (see `move`). Push them.
                if self.has_remote() && self.has_upstream() && self.ahead_behind()?.0 > 0 {
                    match self.push()? {
                        Some(notice) => result.notices.push(notice),
                        None => result.pushed = true,
                    }
                }
                return Ok((result, value));
            }
            Err(e) => {
                tx.rollback();
                return Err(e);
            }
        }
        match self.push() {
            Ok(Some(notice)) => {
                result.notices.push(notice);
            }
            Ok(None) => result.pushed = true,
            Err(first) => {
                // Another writer may have pushed in between. Pull, rebase, and push once more.
                if self.update(true).is_err() {
                    return Err(first);
                }
                match self.push() {
                    Ok(Some(notice)) => result.notices.push(notice),
                    Ok(None) => result.pushed = true,
                    Err(e) => return Err(e),
                }
            }
        }
        Ok((result, value))
    }

    /// Remove the commits after `head` and keep the files as they are in the working tree.
    pub(crate) fn undo_commit_keep_files(&self, head: &str) -> Result<()> {
        run_ok(&self.dir, None, &["reset", "-q", "--mixed", head])?;
        Ok(())
    }

    /// Stage the paths and commit only them. Returns the short hash, or `None` if nothing changed.
    pub(crate) fn commit_change(&self, change: &Change) -> Result<Option<String>> {
        // Never run `git add -A` without paths: it would stage everything.
        if change.paths.is_empty() {
            return Ok(None);
        }
        let paths: Vec<&str> = change.paths.iter().map(String::as_str).collect();
        // A path that is gone from the disk and the index (a revert removed it) cannot be added.
        let tracked = cmd::run(&self.dir, None, &{
            let mut a = vec!["ls-files", "--"];
            a.extend(&paths);
            a
        })?;
        let tracked: Vec<&str> = tracked.stdout.lines().collect();
        let to_add: Vec<&str> = paths
            .iter()
            .copied()
            .filter(|p| self.dir.join(p).exists() || tracked.contains(p))
            .collect();
        if !to_add.is_empty() {
            let mut add = vec!["add", "-A", "--"];
            add.extend(&to_add);
            run_ok(&self.dir, Some(&self.author()), &add)?;
        }
        let mut check = vec!["diff", "--cached", "--quiet", "--"];
        check.extend(&paths);
        let out = cmd::run(&self.dir, Some(&self.author()), &check)?;
        if out.ok {
            return Ok(None);
        }
        let mut commit = vec!["commit", "-q", "-m", &change.message, "--"];
        commit.extend(&paths);
        let out = cmd::run(&self.dir, Some(&self.author()), &commit)?;
        if !out.ok {
            return Err(Error::sync(format!(
                "The commit failed: {}",
                cmd::first_line(&format!("{}\n{}", out.stderr, out.stdout))
            ))
            .with_next("Check the Git hooks of the repository."));
        }
        let h = run_ok(&self.dir, None, &["rev-parse", "--short", "HEAD"])?;
        Ok(Some(h.stdout.trim().to_string()))
    }
}
