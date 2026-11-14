//! How the server finds its knowledge base (see `project-binding`).
//!
//! One process serves one knowledge base. The folder is found at the first use and then kept.
//! The rules, in order: `--path`, `--repo` (default folder), the working folder and its parents,
//! and the one folder `~/Projects/*-knowledge-base` that holds a knowledge base.

use super::args::Args;
use super::repo_url;
use crate::config::Effective;
use crate::error::{Code, Error, Result};
use crate::gitsync::cmd;
use crate::kb::layout::MARKER;
use std::path::{Path, PathBuf};

/// How the folder was found. `status` shows it as `found_by`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoundBy {
    PathArgument,
    RepoArgument,
    WorkingFolder,
    DefaultFolder,
}

impl FoundBy {
    pub fn name(self) -> &'static str {
        match self {
            FoundBy::PathArgument => "path_argument",
            FoundBy::RepoArgument => "repo_argument",
            FoundBy::WorkingFolder => "working_folder",
            FoundBy::DefaultFolder => "default_folder",
        }
    }
}

pub struct Found {
    pub dir: PathBuf,
    pub found_by: FoundBy,
    /// The URL to clone, if the folder does not exist yet.
    pub clone_url: Option<String>,
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// `~/Projects/<slug>-knowledge-base` for a repository URL.
pub fn default_clone_dir(url: &str) -> PathBuf {
    home()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Projects")
        .join(format!("{}-knowledge-base", repo_url::slug(url)))
}

fn has_marker(dir: &Path) -> bool {
    dir.join(MARKER).is_dir() || dir.join("knowledge").join(MARKER).is_dir()
}

fn git_root(dir: &Path) -> PathBuf {
    match cmd::run(dir, None, &["rev-parse", "--show-toplevel"]) {
        Ok(o) if o.ok => std::fs::canonicalize(o.stdout.trim()).unwrap_or(dir.to_path_buf()),
        _ => dir.to_path_buf(),
    }
}

fn no_kb() -> Error {
    Error::setup(
        "No knowledge base is set up. waymark found none in the working folder, and none or several in ~/Projects.",
    )
    .with_next("Add --repo <git url> or --path <folder> to the MCP entry.")
}

/// Pick the folder. No network, no clone.
pub fn locate(args: &Args) -> Result<Found> {
    if let Some(p) = &args.path {
        let clone_url = if p.exists() { None } else { args.repo.clone() };
        return Ok(Found {
            dir: p.clone(),
            found_by: FoundBy::PathArgument,
            clone_url,
        });
    }
    if let Some(url) = &args.repo {
        let dir = default_clone_dir(url);
        let clone_url = if dir.exists() {
            None
        } else {
            Some(url.clone())
        };
        return Ok(Found {
            dir,
            found_by: FoundBy::RepoArgument,
            clone_url,
        });
    }
    if let Ok(cwd) = std::env::current_dir() {
        let mut cur = Some(cwd.as_path());
        while let Some(d) = cur {
            if has_marker(d) {
                return Ok(Found {
                    dir: git_root(d),
                    found_by: FoundBy::WorkingFolder,
                    clone_url: None,
                });
            }
            cur = d.parent();
        }
    }
    let projects = home().map(|h| h.join("Projects"));
    let mut found: Vec<PathBuf> = projects
        .and_then(|p| std::fs::read_dir(p).ok())
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().ends_with("-knowledge-base"))
                .unwrap_or(false)
                && has_marker(p)
        })
        .collect();
    found.sort();
    match found.len() {
        1 => Ok(Found {
            dir: found.remove(0),
            found_by: FoundBy::DefaultFolder,
            clone_url: None,
        }),
        0 => Err(no_kb()),
        _ => {
            let names: Vec<String> = found.iter().map(|p| p.display().to_string()).collect();
            Err(Error::setup(format!(
                "Several knowledge bases are in ~/Projects: {}.",
                names.join(", ")
            ))
            .with_next("Add --repo <git url> or --path <folder> to the MCP entry."))
        }
    }
}

fn denied_host(url: &str, effective: &Effective) -> Option<Error> {
    let h = repo_url::host(url);
    if effective.allows_host(&h) {
        return None;
    }
    Some(
        Error::new(
            Code::AccessDenied,
            format!(
                "The host \"{h}\" of {} is not in allowed_repo_hosts.",
                repo_url::redact(url)
            ),
        )
        .with_next("Ask the owner of the setup to allow this host, or use another repository."),
    )
}

pub fn origin_url(dir: &Path) -> Option<String> {
    let o = cmd::run(dir, None, &["remote", "get-url", "origin"]).ok()?;
    o.ok.then(|| o.stdout.trim().to_string())
}

/// Check an existing folder: a real Git repository, the right origin, an allowed host.
pub fn check_folder(args: &Args, dir: &Path, effective: &Effective) -> Result<()> {
    let git = dir.join(".git");
    if git.is_file() {
        return Err(Error::setup(format!(
            "{} is a linked Git worktree, which waymark does not support.",
            dir.display()
        ))
        .with_next("Use the main clone of the knowledge base."));
    }
    if !git.is_dir() {
        return Err(
            Error::setup(format!("{} is not a Git repository.", dir.display()))
                .with_next("Pass the folder of a clone with --path, or a URL with --repo."),
        );
    }
    let origin = origin_url(dir);
    if let (Some(want), Some(have)) = (&args.repo, &origin) {
        if !repo_url::same_remote(want, have) {
            return Err(Error::setup(format!(
                "The folder {} has the remote {}, not {}.",
                dir.display(),
                repo_url::redact(have),
                repo_url::redact(want)
            ))
            .with_next(
                "Fix --repo or --path in the MCP entry. waymark did not change the folder.",
            ));
        }
    }
    if let Some(have) = &origin {
        if let Some(e) = denied_host(have, effective) {
            return Err(e);
        }
    }
    Ok(())
}

/// Clone to a temporary folder next to the target, then rename it, so that a cut clone never
/// counts as a knowledge base.
pub fn clone_checked(
    url: &str,
    dir: &Path,
    effective: &Effective,
    author: crate::gitsync::Author,
) -> Result<crate::gitsync::Repo> {
    if let Some(e) = denied_host(url, effective) {
        return Err(e);
    }
    if dir.exists()
        && std::fs::read_dir(dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
    {
        return Err(Error::setup(format!(
            "{} exists and is not a Git repository.",
            dir.display()
        ))
        .with_next("Choose another folder with --path, or remove this one."));
    }
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let tmp = dir.with_file_name(format!("{name}.waymark-clone"));
    if tmp.exists() {
        let _ = std::fs::remove_dir_all(&tmp);
    }
    let cloned = crate::gitsync::Repo::clone_from(url, &tmp, author.clone());
    if let Err(mut e) = cloned {
        let _ = std::fs::remove_dir_all(&tmp);
        e.message = repo_url::redact_in(&e.message, url);
        return Err(e);
    }
    drop(cloned);
    let _ = std::fs::remove_dir(dir);
    if let Err(e) = std::fs::rename(&tmp, dir) {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(Error::setup(format!(
            "Cannot move the clone to {}: {e}",
            dir.display()
        )));
    }
    crate::gitsync::Repo::open(dir, author)
}
