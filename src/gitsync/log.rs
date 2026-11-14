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
