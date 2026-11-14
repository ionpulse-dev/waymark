//! The state of one server process: arguments, layer files, and the open repository.

use super::args::Args;
use crate::config::{self, Access, ConfigError, Effective, Limits, Paths};
use crate::error::{Error, Result};
use crate::gitsync::{cmd, Author, Repo};
use crate::kb::{self, KnowledgeBase};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// What a tool needs for one call.
pub struct Context {
    pub args: Args,
    pub paths: Paths,
    repo: Mutex<Option<Arc<Repo>>>,
    /// The documents that `show` opened in this session: the read budget (see `search-and-read`).
    pub opened: Mutex<std::collections::HashSet<String>>,
}

/// The limits for one call, and the problem that stopped them from loading, if any.
pub struct Limit {
    pub effective: Effective,
    pub problem: Option<ConfigError>,
}

impl Context {
    pub fn new(args: Args) -> Context {
        Context::with_paths(args, Paths::from_env())
    }

    pub fn with_paths(args: Args, paths: Paths) -> Context {
        Context {
            args,
            paths,
            repo: Mutex::new(None),
            opened: Mutex::new(std::collections::HashSet::new()),
        }
    }

    /// The effective limits. They load at every call, so an edit of a file applies at once.
    /// A broken layer file makes the access read-only and is reported in `problem`.
    pub fn limits(&self, kb: Option<&KnowledgeBase>) -> Limit {
        let entry = match self.args.entry_layer() {
            Ok(l) => l,
            Err(message) => {
                return Limit {
                    effective: read_only(),
                    problem: Some(ConfigError {
                        path: PathBuf::from("the command line"),
                        message,
                    }),
                }
            }
        };
        match config::load_file_layers(&self.paths) {
            Ok(files) => Limit {
                effective: config::effective(&files, kb.map(|k| &k.settings.limits), Some(&entry)),
                problem: None,
            },
            Err(e) => Limit {
                effective: read_only(),
                problem: Some(e),
            },
        }
    }

    /// The folder of the clone, if it can be named without running Git.
    pub fn repo_dir(&self) -> Option<PathBuf> {
        if let Some(p) = &self.args.path {
            return Some(p.clone());
        }
        if let Some(url) = &self.args.repo {
            return Some(default_clone_dir(url));
        }
        // The working folder and its Git root.
        if let Ok(cwd) = std::env::current_dir() {
            if let Ok(out) = cmd::run(&cwd, None, &["rev-parse", "--show-toplevel"]) {
                if out.ok {
                    let top = PathBuf::from(out.stdout.trim());
                    if kb::find_root(&top).is_ok() {
                        return Some(top);
                    }
                }
            }
        }
        // Exactly one `~/Projects/*-knowledge-base`.
        let projects = home()?.join("Projects");
        let found: Vec<PathBuf> = std::fs::read_dir(projects)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().ends_with("-knowledge-base"))
                    .unwrap_or(false)
                    && p.join(".git").exists()
            })
            .collect();
        (found.len() == 1).then(|| found[0].clone())
    }

    /// The open repository. Clones `--repo` on the first call when the folder does not exist.
    pub fn repo(&self) -> Result<Arc<Repo>> {
        let mut slot = self.repo.lock().unwrap();
        if let Some(r) = slot.as_ref() {
            return Ok(r.clone());
        }
        let dir = self.repo_dir().ok_or_else(|| {
            Error::setup("No knowledge base is set up.")
                .with_next("Add --repo <git-url> to the MCP entry, or use --path <folder>.")
        })?;
        let repo = if dir.join(".git").exists() {
            let author = Author::resolve(
                self.args.author_name.as_deref(),
                self.args.author_email.as_deref(),
                &dir,
            )?;
            Repo::open(&dir, author)?
        } else if let Some(url) = &self.args.repo {
            if dir.exists()
                && std::fs::read_dir(&dir)
                    .map(|mut d| d.next().is_some())
                    .unwrap_or(false)
            {
                return Err(Error::setup(format!(
                    "{} exists and is not a Git repository.",
                    dir.display()
                ))
                .with_next("Choose another folder with --path, or remove this one."));
            }
            let author = Author::resolve(
                self.args.author_name.as_deref(),
                self.args.author_email.as_deref(),
                dir.parent().unwrap_or(Path::new(".")),
            )?;
            Repo::clone_from(url, &dir, author)?
        } else {
            return Err(
                Error::setup(format!("{} is not a Git repository.", dir.display()))
                    .with_next("Pass the folder of a clone with --path, or a URL with --repo."),
            );
        };
        let repo = Arc::new(repo);
        *slot = Some(repo.clone());
        Ok(repo)
    }

    /// The knowledge base in the repository, read from disk at every call.
    /// A different ID than `--kb-id` is a `setup_problem`.
    pub fn kb(&self) -> Result<KnowledgeBase> {
        let repo = self.repo()?;
        let kb = KnowledgeBase::open(repo.dir())?;
        if let Some(want) = &self.args.kb_id {
            if !want.eq_ignore_ascii_case(&kb.settings.id) {
                return Err(Error::setup(
                    "This repository is not the knowledge base that the entry names.",
                )
                .with_next("Check --kb-id and --repo in the MCP entry."));
            }
        }
        Ok(kb)
    }
}

fn read_only() -> Effective {
    Effective::from_layers(&[Limits {
        access: Some(Access::ReadOnly),
        ..Limits::default()
    }])
}

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// `~/Projects/<slug>-knowledge-base` from the last part of a Git URL.
pub fn default_clone_dir(url: &str) -> PathBuf {
    let last = url
        .trim_end_matches('/')
        .rsplit(['/', ':'])
        .next()
        .unwrap_or("knowledge");
    let name = last.trim_end_matches(".git");
    let slug = kb::slug_from_name(name);
    let slug = slug.trim_end_matches("-knowledge-base").to_string();
    let slug = if slug.is_empty() {
        "knowledge".to_string()
    } else {
        slug
    };
    home()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Projects")
        .join(format!("{slug}-knowledge-base"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_folder_comes_from_the_url() {
        let d = default_clone_dir("git@github.com:example/team-knowledge-base.git");
        assert!(d.ends_with("Projects/team-knowledge-base"));
        let d = default_clone_dir("https://example.org/acme/notes.git");
        assert!(d.ends_with("Projects/notes-knowledge-base"));
    }
}
