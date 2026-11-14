//! How the server finds its memory repository (see `project-binding`).
//!
//! One process serves one memory repository. The folder is found at the first use and then kept.
//! The entry names it: `--path` is the folder, and `--repo` alone uses `~/.waymark/<name>`.
//! The server never searches, so it never guesses.

use super::args::Args;
use super::repo_url;
use crate::config::Effective;
use crate::error::{Code, Error, Result};
use crate::gitsync::cmd;
use std::path::{Path, PathBuf};

/// How the folder was found. `status` shows it as `found_by`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoundBy {
    PathArgument,
    RepoArgument,
}

impl FoundBy {
    pub fn name(self) -> &'static str {
        match self {
            FoundBy::PathArgument => "path_argument",
            FoundBy::RepoArgument => "repo_argument",
        }
    }
}

pub struct Found {
    pub dir: PathBuf,
    pub found_by: FoundBy,
    /// The URL to clone, if the folder is missing or empty.
    pub clone_url: Option<String>,
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// `~/.waymark/<name>` for a repository URL.
pub fn default_clone_dir(url: &str) -> Result<PathBuf> {
    let name = repo_url::name(url).ok_or_else(|| {
        Error::setup(format!(
            "Cannot get a folder name from the URL {}.",
            repo_url::redact(url)
        ))
        .with_next("Add --path <folder> to the MCP entry.")
    })?;
    Ok(home()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".waymark")
        .join(name))
}

/// A leading `~` is the home folder. A relative path is resolved against the working folder.
fn expand_path(p: &Path) -> PathBuf {
    let text = p.to_string_lossy();
    if text == "~" || text.starts_with("~/") {
        if let Some(h) = home() {
            return h.join(text.trim_start_matches('~').trim_start_matches('/'));
        }
    }
    if p.is_relative() {
        if let Ok(cwd) = std::env::current_dir() {
            return cwd.join(p);
        }
    }
    p.to_path_buf()
}

/// A folder that is missing, or has no entries.
fn is_missing_or_empty(dir: &Path) -> bool {
    match std::fs::read_dir(dir) {
        Ok(mut d) => d.next().is_none(),
        Err(_) => !dir.exists(),
    }
}

fn nothing_named() -> Error {
    Error::setup("The MCP entry names no memory repository.")
        .with_next("Add --repo <git url> or --path <folder> to the MCP entry.")
}

/// Pick the folder. No network, no clone.
pub fn locate(args: &Args) -> Result<Found> {
    match (&args.repo, &args.path) {
        (None, None) => Err(nothing_named()),
        (repo, Some(p)) => {
            let dir = expand_path(p);
            let clone_url = match repo {
                Some(url) if is_missing_or_empty(&dir) => Some(url.clone()),
                Some(_) => None,
                None if is_missing_or_empty(&dir) => {
                    return Err(Error::setup(format!(
                        "{} is missing or empty, and the MCP entry has no --repo.",
                        dir.display()
                    ))
                    .with_next("Add --repo <git url> to clone the memory repository into it."))
                }
                None => None,
            };
            Ok(Found {
                dir,
                found_by: FoundBy::PathArgument,
                clone_url,
            })
        }
        (Some(url), None) => {
            let dir = default_clone_dir(url)?;
            let clone_url = is_missing_or_empty(&dir).then(|| url.clone());
            Ok(Found {
                dir,
                found_by: FoundBy::RepoArgument,
                clone_url,
            })
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
        .with_next("Use the main clone of the memory repository."));
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
            let what = if args.path.is_some() {
                "Remote mismatch"
            } else {
                "Name collision"
            };
            let fix = if args.path.is_some() {
                "Fix --repo or --path in the MCP entry."
            } else {
                "Add --path <folder> to the MCP entry to choose another folder."
            };
            return Err(Error::setup(format!(
                "{what}: the folder {} has the remote {}, not {}.",
                dir.display(),
                repo_url::redact(have),
                repo_url::redact(want)
            ))
            .with_next(format!("{fix} waymark did not change the folder.")));
        }
    }
    if let Some(have) = &origin {
        if let Some(e) = denied_host(have, effective) {
            return Err(e);
        }
    }
    Ok(())
}

/// Remove temporary clone folders of dead processes, and the old form without a process number.
fn remove_stale_clones(dir: &Path, name: &str) {
    let Some(parent) = dir.parent() else { return };
    let Ok(rd) = std::fs::read_dir(parent) else {
        return;
    };
    let prefix = format!("{name}.waymark-clone");
    for e in rd.filter_map(|e| e.ok()) {
        let file = e.file_name().to_string_lossy().to_string();
        let Some(rest) = file.strip_prefix(&prefix) else {
            continue;
        };
        let alive = rest
            .strip_prefix('-')
            .filter(|p| p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty())
            .is_some_and(|pid| {
                std::process::Command::new("kill")
                    .args(["-0", pid])
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            });
        if !alive {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}

/// Clone to a temporary folder next to the target, then rename it, so that a cut clone never
/// counts as a memory repository.
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
    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::setup(format!("Cannot create {}: {e}", parent.display())))?;
    }
    remove_stale_clones(dir, &name);
    let tmp = dir.with_file_name(format!("{name}.waymark-clone-{}", std::process::id()));
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
        // Another server was faster: use its clone if it is the same repository.
        if dir.join(".git").is_dir() {
            if let Some(have) = origin_url(dir) {
                if repo_url::same_remote(url, &have) {
                    return crate::gitsync::Repo::open(dir, author);
                }
            }
        }
        return Err(Error::setup(format!(
            "Cannot move the clone to {}: {e}",
            dir.display()
        )));
    }
    crate::gitsync::Repo::open(dir, author)
}
