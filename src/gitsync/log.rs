//! Read history from Git.

use super::cmd;
use super::Repo;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub hash: String,
    pub short: String,
    pub author_name: String,
    pub author_email: String,
    pub date: String,
    pub subject: String,
}

/// The authors of paths, from Git history (see `Repo::path_authors`).
#[derive(Debug, Default, Clone)]
pub struct PathAuthors {
    pub first: std::collections::HashMap<String, String>,
    pub last: std::collections::HashMap<String, String>,
}

const SEP: char = '\u{1f}';
const FORMAT: &str = "--format=%H%x1f%h%x1f%an%x1f%ae%x1f%aI%x1f%s";

fn parse_entry(line: &str) -> Option<LogEntry> {
    let mut p = line.split(SEP);
    Some(LogEntry {
        hash: p.next()?.to_string(),
        short: p.next()?.to_string(),
        author_name: p.next()?.to_string(),
        author_email: p.next()?.to_string(),
        date: p.next()?.to_string(),
        subject: p.next()?.to_string(),
    })
}

impl Repo {
    /// The short hash of the last commit that touched a path. `None` if the path has no history.
    pub fn last_commit(&self, rel_path: &str) -> Result<Option<String>> {
        let out = cmd::run(
            &self.dir,
            None,
            &["log", "-1", "--format=%h", "--", rel_path],
        )?;
        if !out.ok {
            return Ok(None);
        }
        let h = out.stdout.trim();
        Ok(if h.is_empty() {
            None
        } else {
            Some(h.to_string())
        })
    }

    /// Where a file went, if Git history shows that it was renamed. Follows the chain up to 10
    /// steps. `None` if it was never renamed. Paths are below the repository folder.
    pub fn renamed_to(&self, rel_path: &str) -> Result<Option<String>> {
        let out = cmd::run(
            &self.dir,
            None,
            &["log", "--diff-filter=R", "-M", "--name-status", "--format="],
        )?;
        if !out.ok {
            return Ok(None);
        }
        // Newest first. A rename line is `R<score>\t<old>\t<new>`.
        let renames: Vec<(String, String)> = out
            .stdout
            .lines()
            .filter_map(|l| {
                let mut p = l.split('\t');
                if !p.next()?.starts_with('R') {
                    return None;
                }
                Some((p.next()?.to_string(), p.next()?.to_string()))
            })
            .collect();
        let mut current = rel_path.to_string();
        let mut moved = false;
        for _ in 0..10 {
            match renames.iter().find(|(old, _)| *old == current) {
                Some((_, new)) => {
                    current = new.clone();
                    moved = true;
                }
                None => break,
            }
        }
        Ok(moved.then_some(current))
    }

    /// For each path in history: the email of the author of the commit that added it, and of the
    /// commit that changed it last. One pass over the history. A rename moves the entries.
    /// Emails are in lowercase. Paths are below the repository folder.
    pub fn path_authors(&self) -> Result<PathAuthors> {
        let out = cmd::run(
            &self.dir,
            None,
            &[
                "log",
                "--reverse",
                "--no-merges",
                "-M",
                "--name-status",
                "--format=%x1e%ae",
            ],
        )?;
        let mut result = PathAuthors::default();
        if !out.ok {
            return Ok(result);
        }
        for commit in out.stdout.split('\u{1e}').skip(1) {
            let mut lines = commit.lines();
            let email = lines.next().unwrap_or("").trim().to_lowercase();
            for line in lines.filter(|l| !l.trim().is_empty()) {
                let mut p = line.split('\t');
                let status = p.next().unwrap_or("");
                match status.chars().next() {
                    Some('A') => {
                        if let Some(path) = p.next() {
                            result.first.insert(path.to_string(), email.clone());
                            result.last.insert(path.to_string(), email.clone());
                        }
                    }
                    Some('M') | Some('T') => {
                        if let Some(path) = p.next() {
                            result.last.insert(path.to_string(), email.clone());
                            result
                                .first
                                .entry(path.to_string())
                                .or_insert_with(|| email.clone());
                        }
                    }
                    Some('R') => {
                        if let (Some(old), Some(new)) = (p.next(), p.next()) {
                            let f = result.first.remove(old);
                            let l = result.last.remove(old);
                            if let Some(f) = f {
                                result.first.insert(new.to_string(), f);
                            }
                            if let Some(l) = l {
                                result.last.insert(new.to_string(), l);
                            }
                        }
                    }
                    Some('D') => {
                        if let Some(path) = p.next() {
                            result.first.remove(path);
                            result.last.remove(path);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(result)
    }

    /// Does the history of a path reach back to a commit? Uses `git log --follow`, so it sees
    /// through a rename. `commit` is a short or full hash.
    pub fn history_reaches(&self, rel_path: &str, commit: &str) -> Result<bool> {
        // A test hook: tests set this to see how a failed history check behaves.
        if std::env::var_os("WAYMARK_TEST_HIDE_HISTORY").is_some() {
            return Ok(false);
        }
        let out = cmd::run(
            &self.dir,
            None,
            &["log", "--follow", "--format=%h", "--", rel_path],
        )?;
        Ok(out.ok
            && out
                .stdout
                .lines()
                .any(|l| commit.starts_with(l.trim()) || l.trim().starts_with(commit)))
    }

    /// The short hash of HEAD, or `None` in a repository without a commit.
    pub fn head(&self) -> Option<String> {
        cmd::run(&self.dir, None, &["rev-parse", "--short", "HEAD"])
            .ok()
            .filter(|o| o.ok)
            .map(|o| o.stdout.trim().to_string())
    }

    /// The text of a file at a commit. `None` if it did not exist there.
    pub fn show_at(&self, commit: &str, rel_path: &str) -> Result<Option<String>> {
        let spec = format!("{commit}:{rel_path}");
        let out = cmd::run(&self.dir, None, &["show", &spec])?;
        Ok(if out.ok { Some(out.stdout) } else { None })
    }

    /// The newest commits, optionally limited to one author email and to some paths.
    pub fn log(
        &self,
        max: usize,
        author_email: Option<&str>,
        paths: &[&str],
    ) -> Result<Vec<LogEntry>> {
        let n = format!("-n{max}");
        let mut args: Vec<String> = vec!["log".into(), n, FORMAT.into(), "--no-merges".into()];
        if let Some(e) = author_email {
            args.push(format!("--author=<{e}>"));
        }
        if !paths.is_empty() {
            args.push("--".into());
            args.extend(paths.iter().map(|s| s.to_string()));
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = cmd::run(&self.dir, None, &refs)?;
        if !out.ok {
            // A repository without commits has no log.
            return Ok(Vec::new());
        }
        Ok(out.stdout.lines().filter_map(parse_entry).collect())
    }

    /// The files that a commit changed: (status letter, path). A rename is `R` with the new path.
    pub fn files_of(&self, commit: &str) -> Result<Vec<(char, String)>> {
        let out = cmd::run_ok(
            &self.dir,
            None,
            &["show", "--name-status", "--format=", "-M", commit],
        )?;
        Ok(out
            .stdout
            .lines()
            .filter_map(|l| {
                let mut p = l.split('\t');
                let status = p.next()?.chars().next()?;
                let last = p.next_back()?;
                Some((status, last.to_string()))
            })
            .collect())
    }

    /// Commits after a date, newest first (`since` is an ISO date or a Git date string).
    pub fn log_since(&self, since: &str, paths: &[&str]) -> Result<Vec<LogEntry>> {
        let mut args: Vec<String> = vec![
            "log".into(),
            FORMAT.into(),
            "--no-merges".into(),
            format!("--since={since}"),
        ];
        if !paths.is_empty() {
            args.push("--".into());
            args.extend(paths.iter().map(|s| s.to_string()));
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = cmd::run(&self.dir, None, &refs)?;
        Ok(if out.ok {
            out.stdout.lines().filter_map(parse_entry).collect()
        } else {
            Vec::new()
        })
    }
}

/// A commit with its message body and the files it changed (see `Repo::all_commits`).
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub short: String,
    pub email: String,
    pub name: String,
    pub time: String,
    pub subject: String,
    pub body: String,
    /// (status letter, path). A rename is `R` with the new path, and the old path as `D`.
    pub files: Vec<(char, String)>,
}

impl Repo {
    /// All commits without merges, newest first, with message body and changed files.
    pub fn all_commits(&self) -> Result<Vec<CommitInfo>> {
        let out = cmd::run(
            &self.dir,
            None,
            &[
                "log",
                "--no-merges",
                "-M",
                "--name-status",
                "--format=%x1e%H%x1f%h%x1f%ae%x1f%an%x1f%aI%x1f%s%x1f%b%x1f",
            ],
        )?;
        if !out.ok {
            return Ok(Vec::new());
        }
        let mut commits = Vec::new();
        for chunk in out.stdout.split('\u{1e}').skip(1) {
            let mut parts = chunk.splitn(8, '\u{1f}');
            let (
                Some(hash),
                Some(short),
                Some(email),
                Some(name),
                Some(time),
                Some(subject),
                Some(body),
                Some(rest),
            ) = (
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
                parts.next(),
            )
            else {
                continue;
            };
            let mut files = Vec::new();
            for line in rest.lines().filter(|l| !l.trim().is_empty()) {
                let mut p = line.split('\t');
                let status = p.next().unwrap_or("").chars().next().unwrap_or('M');
                if status == 'R' || status == 'C' {
                    if let (Some(old), Some(new)) = (p.next(), p.next()) {
                        files.push(('D', old.to_string()));
                        files.push(('A', new.to_string()));
                    }
                } else if let Some(path) = p.next() {
                    files.push((status, path.to_string()));
                }
            }
            commits.push(CommitInfo {
                hash: hash.to_string(),
                short: short.to_string(),
                email: email.to_lowercase(),
                name: name.to_string(),
                time: time.to_string(),
                subject: subject.to_string(),
                body: body.trim().to_string(),
                files,
            });
        }
        Ok(commits)
    }

    /// Revert commits without committing. `hashes` are newest first. Returns the staged changes
    /// (status letter, path), or the first line of the error if the revert conflicts. A conflict
    /// leaves the working tree as it was.
    pub fn revert_no_commit(
        &self,
        hashes: &[String],
    ) -> Result<std::result::Result<Vec<(char, String)>, String>> {
        let mut args = vec!["revert", "--no-commit"];
        args.extend(hashes.iter().map(String::as_str));
        let out = cmd::run(&self.dir, Some(&self.author()), &args)?;
        if !out.ok {
            self.abort_revert();
            return Ok(Err(cmd::first_line(&format!(
                "{}\n{}",
                out.stderr, out.stdout
            ))));
        }
        let diff = cmd::run_ok(
            &self.dir,
            None,
            &["diff", "--cached", "--name-status", "-M"],
        )?;
        let mut files = Vec::new();
        for line in diff.stdout.lines() {
            let mut p = line.split('\t');
            let status = p.next().unwrap_or("").chars().next().unwrap_or('M');
            if status == 'R' {
                if let (Some(old), Some(new)) = (p.next(), p.next()) {
                    files.push(('D', old.to_string()));
                    files.push(('A', new.to_string()));
                }
            } else if let Some(path) = p.next() {
                files.push((status, path.to_string()));
            }
        }
        Ok(Ok(files))
    }

    /// Drop a revert that was started with `revert_no_commit`. Keeps unrelated local changes.
    pub fn abort_revert(&self) {
        let _ = cmd::run(&self.dir, None, &["revert", "--abort"]);
        let _ = cmd::run(&self.dir, None, &["reset", "-q", "--merge"]);
    }
}
