//! Git calls for `updates`: the base commit of a window and the net change since then.

use super::cmd::{self, run};
use super::Repo;
use crate::error::Result;

/// The hash of the empty tree. It is the base for a window that starts before the first commit.
pub const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// One changed path between the base and `HEAD`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathChange {
    /// `A`, `M`, `D`, or `R`.
    pub status: char,
    pub path: String,
    /// The old path of a rename.
    pub from: Option<String>,
}

/// A commit of a path in the window.
#[derive(Debug, Clone)]
pub struct WindowCommit {
    pub email: String,
    pub name: String,
    pub date: String,
}

impl Repo {
    /// The newest commit at or before a time (an RFC 3339 text), or the empty tree.
    pub fn base_commit_before(&self, when: &str) -> String {
        let out = run(
            &self.dir,
            None,
            &["rev-list", "-1", &format!("--before={when}"), "HEAD"],
        );
        match out {
            Ok(o) if o.ok && !o.stdout.trim().is_empty() => o.stdout.trim().to_string(),
            _ => EMPTY_TREE.to_string(),
        }
    }

    /// The net change of `.md` files between the base and `HEAD`, below a repository path.
    pub fn changed_paths(&self, base: &str, below: &str) -> Result<Vec<PathChange>> {
        if self.head().is_none() {
            return Ok(Vec::new());
        }
        let mut args = vec!["diff", "--name-status", "-M", base, "HEAD"];
        if !below.is_empty() {
            args.push("--");
            args.push(below);
        }
        let out = cmd::run_ok(&self.dir, None, &args)?;
        Ok(out
            .stdout
            .lines()
            .filter_map(|l| {
                let mut p = l.split('\t');
                let status = p.next()?.chars().next()?;
                let first = p.next()?.to_string();
                match p.next() {
                    Some(new) if status == 'R' => Some(PathChange {
                        status,
                        path: new.to_string(),
                        from: Some(first),
                    }),
                    _ => Some(PathChange {
                        status,
                        path: first,
                        from: None,
                    }),
                }
            })
            .collect())
    }

    /// The commits that changed a path in the window, newest first. Follows renames.
    pub fn window_commits(&self, base: &str, path: &str) -> Result<Vec<WindowCommit>> {
        let range = if base == EMPTY_TREE {
            "HEAD".to_string()
        } else {
            format!("{base}..HEAD")
        };
        let out = run(
            &self.dir,
            None,
            &[
                "log",
                "--follow",
                "--no-merges",
                "--format=%ae%x09%an%x09%as",
                &range,
                "--",
                path,
            ],
        )?;
        if !out.ok {
            return Ok(Vec::new());
        }
        Ok(out
            .stdout
            .lines()
            .filter_map(|l| {
                let mut p = l.split('\t');
                Some(WindowCommit {
                    email: p.next()?.to_lowercase(),
                    name: p.next()?.to_string(),
                    date: p.next()?.to_string(),
                })
            })
            .collect())
    }
}
