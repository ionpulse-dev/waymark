//! Update from the remote, push, and the conflict rule.

use super::cmd::{self, run, run_ok};
use super::lock::Lock;
use super::message::{commit_message, Verb};
use super::write::{Change, Tx};
use super::Repo;
use crate::error::{Error, Result};
use crate::format;
use sha2::{Digest, Sha256};
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    pub commits: Vec<String>,
    pub pushed: bool,
    /// The number of commits that the pull brought in.
    pub pulled: usize,
    pub notices: Vec<String>,
    /// Pairs (original path, conflict copy path) of documents that were edited at the same time.
    pub conflicts: Vec<(String, String)>,
}

impl Repo {
    /// Does the current branch have an upstream?
    pub(crate) fn has_upstream(&self) -> bool {
        run(
            &self.dir,
            None,
            &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
        )
        .map(|o| o.ok)
        .unwrap_or(false)
    }

    /// (ahead, behind) of the upstream.
    pub(crate) fn ahead_behind(&self) -> Result<(usize, usize)> {
        let out = run_ok(
            &self.dir,
            None,
            &["rev-list", "--left-right", "--count", "HEAD...@{u}"],
        )?;
        let mut p = out.stdout.split_whitespace();
        let a = p.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        let b = p.next().and_then(|x| x.parse().ok()).unwrap_or(0);
        Ok((a, b))
    }

    /// For a clone with no commit: look at the remote again, and take its branch if it has one.
    pub fn refresh_if_unborn(&self) {
        if self.head().is_none() && self.has_remote() {
            let _ = self.update(false);
        }
    }

    /// A clone of an empty repository has no commit. If the remote got commits since, check out
    /// its branch, so that a second person's `init` sees them (see `create-memory-repository`).
    fn join_remote_branch_if_unborn(&self) -> Result<()> {
        if self.head().is_some() {
            return Ok(());
        }
        let refs = run(
            &self.dir,
            None,
            &[
                "for-each-ref",
                "--format=%(refname:short)",
                "refs/remotes/origin",
            ],
        )?;
        let branches: Vec<String> = refs
            .stdout
            .lines()
            .filter_map(|l| l.strip_prefix("origin/"))
            .filter(|b| *b != "HEAD")
            .map(String::from)
            .collect();
        let pick = ["main", "master"]
            .iter()
            .find(|b| branches.iter().any(|x| x == *b))
            .map(|b| b.to_string())
            .or_else(|| branches.first().cloned());
        let Some(branch) = pick else { return Ok(()) };
        let out = run(
            &self.dir,
            Some(&self.author()),
            &[
                "checkout",
                "-q",
                "-B",
                &branch,
                "--track",
                &format!("origin/{branch}"),
            ],
        )?;
        if !out.ok {
            return Err(Error::sync(format!(
                "Cannot bring the local copy up to date: {}",
                cmd::first_line(&out.stderr)
            ))
            .with_next("Look at the local files in the memory repository folder."));
        }
        Ok(())
    }

    /// Fetch and bring the clone up to date. In a write a failure is an error. In a read it is a notice.
    /// Returns notices for the result.
    pub(crate) fn update(&self, for_write: bool) -> Result<Vec<String>> {
        if !self.has_remote() {
            return Ok(vec![
                "This repository has no remote, so nothing is shared.".to_string()
            ]);
        }
        let fetched = run(&self.dir, None, &["fetch", "--quiet"])?;
        self.set_fetch_failed(!fetched.ok);
        if !fetched.ok {
            let msg = format!(
                "Cannot reach the remote: {}",
                cmd::first_line(&fetched.stderr)
            );
            if for_write {
                return Err(Error::sync(msg)
                    .with_next("Check the network and your Git login, then try again."));
            }
            return Ok(vec![format!("{msg}. The local copy may be behind.")]);
        }
        self.mark_fetched();
        if self.head().is_none() {
            self.join_remote_branch_if_unborn()?;
            return Ok(Vec::new());
        }
        if !self.has_upstream() {
            return Ok(Vec::new());
        }
        let (ahead, behind) = self.ahead_behind()?;
        if behind == 0 {
            return Ok(Vec::new());
        }
        let out = if ahead == 0 {
            run(
                &self.dir,
                Some(&self.author()),
                &["merge", "--ff-only", "--quiet", "@{u}"],
            )?
        } else {
            run(
                &self.dir,
                Some(&self.author()),
                &["pull", "--rebase", "--quiet"],
            )?
        };
        if !out.ok {
            if ahead > 0 {
                let _ = run(&self.dir, None, &["rebase", "--abort"]);
            }
            let msg = format!(
                "Cannot bring the local copy up to date: {}",
                cmd::first_line(&out.stderr)
            );
            if for_write {
                return Err(Error::sync(msg).with_next(
                    "Run sync, or look at the local changes in the memory repository folder.",
                ));
            }
            return Ok(vec![format!("{msg}. The local copy may be behind.")]);
        }
        Ok(Vec::new())
    }

    /// Update at the start of every read call. There is no freshness window. Calls that overlap in
    /// this process share one update: the first runs it, the others wait for its result.
    /// A failure never blocks a read. It gives a notice.
    pub fn refresh_for_read(&self) -> Vec<String> {
        if !self.has_remote() {
            return vec!["This repository has no remote, so nothing is shared.".to_string()];
        }
        let mut gate = self.gate.lock().unwrap();
        if gate.running {
            let seen = gate.generation;
            while gate.running && gate.generation == seen {
                gate = self.gate_cv.wait(gate).unwrap();
            }
            return gate.notices.clone();
        }
        gate.running = true;
        drop(gate);
        let notices =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match Lock::acquire(&self.git_dir().unwrap_or_default(), Duration::from_secs(5)) {
                    Ok(_lock) => self.update(false).unwrap_or_else(|e| vec![e.message]),
                    Err(_) => Vec::new(),
                }
            }))
            .unwrap_or_default();
        let mut gate = self.gate.lock().unwrap();
        gate.running = false;
        gate.generation += 1;
        gate.runs += 1;
        gate.notices = notices.clone();
        drop(gate);
        self.gate_cv.notify_all();
        notices
    }

    /// The number of updates that `refresh_for_read` ran in this process (for tests).
    pub fn read_updates_run(&self) -> u64 {
        self.gate.lock().unwrap().runs
    }

    /// Push. Returns a notice instead of an error when there is no remote. A rejected push is `sync_failed`.
    pub(crate) fn push(&self) -> Result<Option<String>> {
        if !self.has_remote() {
            return Ok(Some(
                "This repository has no remote, so nothing is shared.".to_string(),
            ));
        }
        let out = if self.has_upstream() {
            run(&self.dir, None, &["push", "--quiet"])?
        } else {
            run(
                &self.dir,
                None,
                &["push", "--quiet", "-u", "origin", "HEAD"],
            )?
        };
        if out.ok {
            return Ok(None);
        }
        Err(Error::sync(format!(
            "The push was rejected: {}. The change is saved on this computer.",
            cmd::first_line(&out.stderr)
        ))
        .with_next("Run sync to merge and push it, or try again later."))
    }

    /// The `sync` operation. First, `pending` may write and commit files for edits made outside
    /// waymark, and returns the commits it made. Then pull with rebase, apply the conflict rule, and push.
    pub fn sync(&self, pending: impl FnOnce(&Tx) -> Result<Vec<String>>) -> Result<SyncResult> {
        let _lock = Lock::acquire(&self.git_dir()?, Duration::from_secs(30))?;
        let mut result = SyncResult::default();
        let tx = Tx::new_for_sync(self);
        result.commits = pending(&tx)?;
        if !self.has_remote() {
            result
                .notices
                .push("This repository has no remote, so nothing is shared.".to_string());
            return Ok(result);
        }
        let fetched = run(&self.dir, None, &["fetch", "--quiet"])?;
        if !fetched.ok {
            return Err(Error::sync(format!(
                "Cannot reach the remote: {}",
                cmd::first_line(&fetched.stderr)
            ))
            .with_next("Check the network and your Git login, then try again."));
        }
        self.mark_fetched();
        if self.has_upstream() {
            let (ahead, behind) = self.ahead_behind()?;
            if behind > 0 {
                let out = if ahead == 0 {
                    run(
                        &self.dir,
                        Some(&self.author()),
                        &["merge", "--ff-only", "--quiet", "@{u}"],
                    )?
                } else {
                    run(
                        &self.dir,
                        Some(&self.author()),
                        &["pull", "--rebase", "--quiet"],
                    )?
                };
                result.pulled = behind;
                if !out.ok {
                    self.resolve_conflicts(&mut result)?;
                }
            }
        }
        if let Some(notice) = self.push()? {
            result.notices.push(notice);
        } else {
            result.pushed = true;
        }
        Ok(result)
    }

    /// Only bring the clone up to date: fetch and fast-forward or rebase. Nothing is committed or
    /// pushed. Returns the number of commits that came in, and notices.
    pub fn pull_only(&self) -> Result<(usize, Vec<String>)> {
        let _lock = Lock::acquire(&self.git_dir()?, Duration::from_secs(30))?;
        if !self.has_remote() {
            return Ok((
                0,
                vec!["This repository has no remote, so nothing is shared.".to_string()],
            ));
        }
        let fetched = run(&self.dir, None, &["fetch", "--quiet"])?;
        if !fetched.ok {
            return Err(Error::sync(format!(
                "Cannot reach the remote: {}",
                cmd::first_line(&fetched.stderr)
            ))
            .with_next("Check the network and your Git login, then try again."));
        }
        let behind = if self.has_upstream() {
            self.ahead_behind()?.1
        } else {
            0
        };
        let notices = self.update(false)?;
        Ok((behind, notices))
    }

    /// The conflict rule. The rebase failed. Abort it, then keep both versions of every document
    /// that was edited on both sides: the remote version stays at the original path, and the local
    /// version is saved next to it as a disputed copy. Then merge, which is clean now.
    fn resolve_conflicts(&self, result: &mut SyncResult) -> Result<()> {
        let _ = run(&self.dir, None, &["rebase", "--abort"]);
        let _ = run(&self.dir, None, &["merge", "--abort"]);
        // A rebase stops at the first local commit that conflicts, and `sync` makes one commit for
        // each edit. A trial merge of the two tips finds the conflicts of all commits at once.
        let trial = run(
            &self.dir,
            Some(&self.author()),
            &["merge", "--no-commit", "--no-ff", "--quiet", "@{u}"],
        )?;
        let conflicted = run(&self.dir, None, &["diff", "--name-only", "--diff-filter=U"])?;
        let paths: Vec<String> = conflicted
            .stdout
            .lines()
            .map(str::to_string)
            .filter(|l| !l.is_empty())
            .collect();
        let _ = run(&self.dir, None, &["merge", "--abort"]);
        if paths.is_empty() && trial.ok {
            // The tips join without a conflict: merge them.
            let merged = run(
                &self.dir,
                Some(&self.author()),
                &["merge", "--no-edit", "--quiet", "@{u}"],
            )?;
            if merged.ok {
                return Ok(());
            }
            let _ = run(&self.dir, None, &["merge", "--abort"]);
            return Err(Error::sync(format!(
                "The merge failed: {}",
                cmd::first_line(&merged.stderr)
            ))
            .with_next("Look at the memory repository folder, or ask someone who knows Git."));
        }
        if paths.is_empty() {
            let why = cmd::first_line(&trial.stderr);
            return Err(Error::sync(format!(
                "The local copy and the remote cannot be joined: {why}"
            ))
            .with_next("Look at the memory repository folder, or ask someone who knows Git."));
        }
        if let Some(bad) = paths.iter().find(|p| !p.ends_with(".md")) {
            return Err(Error::sync(format!(
                "The file {bad} was changed on both sides and is not a document."
            ))
            .with_next("Join the two changes by hand in the memory repository folder."));
        }
        let mut to_commit = Vec::new();
        for p in &paths {
            let local = run_ok(&self.dir, None, &["show", &format!("HEAD:{p}")])
                .map(|o| o.stdout)
                .ok();
            let remote_exists =
                run(&self.dir, None, &["cat-file", "-e", &format!("@{{u}}:{p}")])?.ok;
            // Put the remote version at the original path.
            if remote_exists {
                run_ok(&self.dir, None, &["checkout", "@{u}", "--", p])?;
            } else {
                // The remote deleted it. The local version becomes the copy only.
                let _ = run(&self.dir, None, &["rm", "-q", "-f", "--", p]);
            }
            to_commit.push(p.clone());
            if let Some(local_text) = local {
                let copy = conflict_copy(p, &local_text, &self.author(), &today());
                self.write_raw(&copy.0, &copy.1)?;
                to_commit.push(copy.0.clone());
                result.conflicts.push((p.clone(), copy.0));
            }
        }
        let message = commit_message(Verb::Sync, &conflict_title(&paths));
        if let Some(h) = self.commit_change(&Change {
            paths: to_commit,
            message,
        })? {
            result.commits.push(h);
        }
        let merged = run(
            &self.dir,
            Some(&self.author()),
            &["merge", "--no-edit", "--quiet", "@{u}"],
        )?;
        if !merged.ok {
            let _ = run(&self.dir, None, &["merge", "--abort"]);
            return Err(Error::sync(format!(
                "The merge failed: {}",
                cmd::first_line(&merged.stderr)
            ))
            .with_next("Look at the memory repository folder, or ask someone who knows Git."));
        }
        for (orig, copy) in &result.conflicts {
            result.notices.push(format!(
                "Two people changed {orig} at the same time. Both versions are kept: the other version is in {orig}, yours is in {copy}. Merge the two."
            ));
        }
        Ok(())
    }

    fn write_raw(&self, rel: &str, content: &str) -> Result<()> {
        let p = self.dir.join(rel);
        std::fs::write(&p, content).map_err(|e| Error::setup(format!("Cannot write {rel}: {e}")))
    }
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn conflict_title(paths: &[String]) -> String {
    match paths.first() {
        Some(p) if paths.len() == 1 => format!("Conflict in {}", p.trim_end_matches(".md")),
        _ => format!("Conflicts in {} documents", paths.len()),
    }
}

/// The name and the text of the conflict copy of a document.
/// Name: `<name>-conflict-<7 hex>.md`, with the hex from the SHA-256 of the local text.
/// The copy has a new title, `disputed: true`, and a reason.
pub fn conflict_copy(
    path: &str,
    local_text: &str,
    author: &cmd::Author,
    today: &str,
) -> (String, String) {
    let mut hasher = Sha256::new();
    hasher.update(local_text.as_bytes());
    let hex: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()[..7]
        .to_string();
    let stem = path.trim_end_matches(".md");
    let copy_path = format!("{stem}-conflict-{hex}.md");
    let id = stem.to_string();
    match format::parse(local_text) {
        Ok(mut doc) => {
            let limit = format::Limits::default().title_max_words;
            let title = doc
                .frontmatter
                .get_text("title")
                .unwrap_or("Untitled")
                .to_string();
            let words: Vec<&str> = title.split_whitespace().collect();
            let keep = words.len().min(limit.saturating_sub(2));
            let new_title = format!("{} [conflict {hex}]", words[..keep].join(" "));
            doc.frontmatter.set_text("title", new_title.trim());
            doc.frontmatter.set_bool("disputed", true);
            doc.frontmatter.set_text(
                "dispute_reason",
                &format!("Edited at the same time as {id}. Merge the two."),
            );
            doc.frontmatter.set_date("updated", today);
            doc.frontmatter
                .set_text("updated_by", &format!("{} <{}>", author.name, author.email));
            (copy_path, doc.render())
        }
        Err(_) => (copy_path, local_text.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn author() -> cmd::Author {
        cmd::Author {
            name: "Jane Doe".into(),
            email: "jane@example.org".into(),
        }
    }

    const DOC: &str = "---\ntitle: Deploy process\ndescription: How we deploy\nstatus: active\ncreated: 2026-11-01\nupdated: 2026-11-02\ncreated_by: \"A B <a@example.org>\"\nupdated_by: \"A B <a@example.org>\"\n---\nLocal text\n";

    #[test]
    fn the_conflict_copy_name_and_flags() {
        let (path, text) = conflict_copy("ops/deploy.md", DOC, &author(), "2026-11-14");
        assert!(path.starts_with("ops/deploy-conflict-") && path.ends_with(".md"));
        assert_eq!(path.len(), "ops/deploy-conflict-".len() + 7 + 3);
        assert!(format::id::check_path(&path).is_empty(), "{path}");
        let d = format::parse(&text).unwrap();
        assert!(d
            .frontmatter
            .get_text("title")
            .unwrap()
            .starts_with("Deploy process [conflict "));
        assert_eq!(d.frontmatter.get_bool("disputed"), Some(true));
        assert!(d
            .frontmatter
            .get_text("dispute_reason")
            .unwrap()
            .contains("ops/deploy"));
        assert_eq!(d.frontmatter.get_text("updated"), Some("2026-11-14"));
        assert!(d.body.contains("Local text"));
        let findings = format::check(&path, &text, &format::Limits::default(), None);
        assert!(!format::has_errors(&findings), "{findings:?}");
    }

    #[test]
    fn the_copy_title_stays_within_the_word_limit() {
        let long = vec!["word"; 15].join(" ");
        let doc = DOC.replace("Deploy process", &long);
        let (path, text) = conflict_copy("a.md", &doc, &author(), "2026-11-14");
        let findings = format::check(&path, &text, &format::Limits::default(), None);
        assert!(!format::has_errors(&findings), "{findings:?}");
    }

    #[test]
    fn the_same_text_gives_the_same_name() {
        let a = conflict_copy("a.md", DOC, &author(), "2026-11-14").0;
        let b = conflict_copy("a.md", DOC, &author(), "2026-11-15").0;
        assert_eq!(a, b);
    }
}
