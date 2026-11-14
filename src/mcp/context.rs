//! The state of one server process: arguments, layer files, and the open repository.

use super::args::Args;
use crate::config::{self, Access, ConfigError, Effective, Limits, Paths};
use crate::error::{Error, Result};
use crate::gitsync::{Author, Repo};
use crate::kb::KnowledgeBase;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// The state of the server after the start (see `join-knowledge-base`).
#[derive(Debug, Clone)]
pub enum JoinState {
    Cloning,
    Ready,
    Empty,
    NotAKnowledgeBase,
    Broken(Error),
}

impl JoinState {
    pub fn name(&self) -> &'static str {
        match self {
            JoinState::Cloning => "cloning",
            JoinState::Ready => "ready",
            JoinState::Empty => "empty",
            JoinState::NotAKnowledgeBase => "not_a_knowledge_base",
            JoinState::Broken(_) => "broken",
        }
    }
}

/// What a tool needs for one call.
pub struct Context {
    pub args: Args,
    pub paths: Paths,
    repo: Mutex<Option<Arc<Repo>>>,
    found_by: Mutex<Option<super::binding::FoundBy>>,
    /// The last failure of the start, and when it happened (see `join-knowledge-base`).
    last_error: Mutex<Option<(Error, std::time::Instant)>>,
    /// The documents that `show` opened in this session: the read budget (see `search-and-read`).
    /// The connection to the client for the current call (see `write-approval`).
    pub channel: Mutex<Option<super::approval::Channel>>,
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
            found_by: Mutex::new(None),
            last_error: Mutex::new(None),
            channel: Mutex::new(None),
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

    /// How the folder was found, once the repository is open.
    pub fn found_by(&self) -> Option<super::binding::FoundBy> {
        *self.found_by.lock().unwrap()
    }

    /// The open repository (see `project-binding`). The folder is found at the first call and
    /// kept. A new clone happens only here, once.
    pub fn repo(&self) -> Result<Arc<Repo>> {
        let wait: u64 = std::env::var("WAYMARK_CLONE_WAIT_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(wait);
        let mut slot = loop {
            match self.repo.try_lock() {
                Ok(g) => break g,
                Err(std::sync::TryLockError::Poisoned(p)) => break p.into_inner(),
                Err(std::sync::TryLockError::WouldBlock) => {
                    if std::time::Instant::now() >= deadline {
                        return Err(Error::setup("The first clone is still running.")
                            .with_next("Call again in a minute."));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        };
        if let Some(r) = slot.as_ref() {
            return Ok(r.clone());
        }
        // A failed clone is tried again at the next call, at most once in 30 seconds.
        if let Some((e, at)) = self.last_error.lock().unwrap().as_ref() {
            if e.code == crate::error::Code::SyncFailed && at.elapsed().as_secs() < 30 {
                return Err(e.clone());
            }
        }
        match self.open_repo() {
            Ok(r) => {
                *self.last_error.lock().unwrap() = None;
                *slot = Some(r.clone());
                Ok(r)
            }
            Err(e) => {
                *self.last_error.lock().unwrap() = Some((e.clone(), std::time::Instant::now()));
                Err(e)
            }
        }
    }

    /// Is the first clone running now? `status` and `initialize` never wait for it.
    pub fn is_cloning(&self) -> bool {
        matches!(
            self.repo.try_lock(),
            Err(std::sync::TryLockError::WouldBlock)
        )
    }

    /// The state of the knowledge base, without waiting (see `join-knowledge-base`).
    pub fn join_state(&self) -> JoinState {
        if self.is_cloning() {
            return JoinState::Cloning;
        }
        let repo = match self.repo() {
            Ok(r) => r,
            Err(e) => return JoinState::Broken(e),
        };
        match crate::kb::layout::find_root(repo.dir()) {
            Err(_) if crate::kb::layout::repo_is_empty(repo.dir()) => JoinState::Empty,
            Err(_) => JoinState::NotAKnowledgeBase,
            Ok(_) => match self.kb() {
                Ok(_) => JoinState::Ready,
                Err(e) => JoinState::Broken(e),
            },
        }
    }

    /// Start the first clone in the background, so that `initialize` answers at once.
    pub fn start_in_background(self: &Arc<Self>) {
        if self.args.repo.is_none() {
            return;
        }
        let ctx = self.clone();
        std::thread::spawn(move || {
            let _ = ctx.repo();
        });
    }

    fn open_repo(&self) -> Result<Arc<Repo>> {
        let found = super::binding::locate(&self.args)?;
        let limit = self.limits(None);
        let effective = &limit.effective;
        // A missing author does not stop reads. A write resolves and checks it again.
        let author = |dir: &Path| {
            Ok::<_, Error>(
                self.resolve_author(dir)
                    .map(|r| r.author())
                    .unwrap_or(Author {
                        name: String::new(),
                        email: String::new(),
                    }),
            )
        };
        let repo = if let Some(url) = &found.clone_url {
            let parent = found.dir.parent().unwrap_or(Path::new("."));
            super::binding::clone_checked(url, &found.dir, effective, author(parent)?)?
        } else {
            if !found.dir.exists() {
                return Err(
                    Error::setup(format!("{} does not exist.", found.dir.display()))
                        .with_next("Pass the folder of a clone with --path, or a URL with --repo."),
                );
            }
            super::binding::check_folder(&self.args, &found.dir, effective)?;
            Repo::open(&found.dir, author(&found.dir)?)?
        };
        let repo = Arc::new(repo);
        *self.found_by.lock().unwrap() = Some(found.found_by);
        Ok(repo)
    }

    /// Resolve the author now: argument, environment, Git settings (see `login-and-identity`).
    pub fn resolve_author(
        &self,
        dir: &Path,
    ) -> std::result::Result<crate::identity::Resolved, crate::identity::AuthorProblem> {
        crate::identity::resolve(
            self.args.author_name.as_deref(),
            self.args.author_email.as_deref(),
            std::env::var("WAYMARK_AUTHOR_NAME").ok().as_deref(),
            std::env::var("WAYMARK_AUTHOR_EMAIL").ok().as_deref(),
            dir,
        )
    }

    /// The author for a write: resolved and checked at every call, then set on the repository.
    pub fn checked_author(&self) -> Result<crate::identity::Resolved> {
        let repo = self.repo()?;
        let resolved = self.resolve_author(repo.dir())?;
        let people = KnowledgeBase::open(repo.dir())
            .map(|k| k.settings.people)
            .unwrap_or_default();
        crate::identity::check_email(&resolved.email, &people)?;
        repo.set_author(resolved.author());
        Ok(resolved)
    }

    /// The knowledge base if it is ready now. It never waits for a clone.
    pub fn kb_if_ready(&self) -> Option<KnowledgeBase> {
        if self.is_cloning() {
            None
        } else {
            self.kb().ok()
        }
    }

    /// The knowledge base in the repository, read from disk at every call.
    /// A different ID than `--kb-id` is a `setup_problem`.
    pub fn kb(&self) -> Result<KnowledgeBase> {
        let repo = self.repo()?;
        if crate::kb::layout::find_root(repo.dir()).is_err()
            && crate::kb::layout::repo_is_empty(repo.dir())
        {
            return Err(Error::setup("This repository is empty.")
                .with_next("This repository is empty. Call `init`."));
        }
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
