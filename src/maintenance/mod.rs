//! The health checks of `status` (see `specs/features/maintenance`).

pub mod sources;

use crate::config::{self, Limits};
use crate::gitsync::cmd;
use crate::gitsync::probe::Probe;
use crate::identity::{self, redact_text, LOGIN_NEXT};
use crate::kb::KnowledgeBase;
use crate::mcp::context::{Context, JoinState};
use serde_json::{json, Value};
use sources::Layer;

#[derive(Debug, Clone)]
pub struct Check {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
    pub fix: Option<String>,
}

impl Check {
    fn ok(name: &'static str, detail: impl Into<String>) -> Check {
        Check {
            name,
            ok: true,
            detail: detail.into(),
            fix: None,
        }
    }

    fn bad(name: &'static str, detail: impl Into<String>, fix: impl Into<String>) -> Check {
        Check {
            name,
            ok: false,
            detail: detail.into(),
            fix: Some(fix.into()),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "ok": self.ok,
            "detail": redact_text(&self.detail),
            "fix": self.fix.as_ref().map(|f| redact_text(f)),
        })
    }
}

/// The result of all checks.
pub struct Report {
    pub checks: Vec<Check>,
    pub state: JoinState,
    pub kb: Option<KnowledgeBase>,
    pub author: Option<identity::Resolved>,
}

fn git_check() -> Check {
    match cmd::run_timeout(std::path::Path::new("."), &["--version"], 5) {
        Ok(Some(o)) if o.ok => Check::ok("git", o.stdout.trim()),
        Ok(None) => Check::bad(
            "git",
            "git did not answer in 5 seconds",
            "Install the Xcode command line tools: run `xcode-select --install`, or install Git with Homebrew.",
        ),
        _ => Check::bad(
            "git",
            "git does not work",
            "Install the Xcode command line tools: run `xcode-select --install`, or install Git with Homebrew.",
        ),
    }
}

fn clone_check(state: &JoinState) -> Check {
    match state {
        JoinState::Cloning => Check::bad(
            "clone",
            "cloning",
            "The first clone is still running. Call `status` again in a minute.",
        ),
        JoinState::Ready => Check::ok("clone", "ready"),
        JoinState::Empty => Check::ok("clone", "The repository is empty. Call `init`."),
        JoinState::NotAKnowledgeBase => Check::ok("clone", "not_a_knowledge_base"),
        JoinState::Broken(e) => Check::bad(
            "clone",
            e.message.clone(),
            e.next
                .clone()
                .unwrap_or_else(|| "Fix the problem and call `status` again.".into()),
        ),
    }
}

fn working_tree_check(repo: &crate::gitsync::Repo) -> Check {
    let git_dir = repo.dir().join(".git");
    let half = ["rebase-merge", "rebase-apply", "MERGE_HEAD"]
        .iter()
        .any(|n| git_dir.join(n).exists());
    let pending = cmd::run(repo.dir(), None, &["status", "--porcelain"])
        .ok()
        .filter(|o| o.ok)
        .map(|o| o.stdout.lines().count())
        .unwrap_or(0);
    if half {
        Check::bad(
            "working_tree",
            format!("{pending} files with pending changes"),
            format!(
                "A previous Git operation did not finish. Open the folder `{}` in a terminal, run `git status`, and finish or abort it.",
                repo.dir().display()
            ),
        )
    } else {
        Check::ok(
            "working_tree",
            format!("{pending} files with pending changes"),
        )
    }
}

fn settings_check(ctx: &Context, kb: Option<&KnowledgeBase>) -> Check {
    let mut read = 0;
    let mut files: Vec<std::path::PathBuf> = ctx.paths.system.clone();
    files.push(ctx.paths.personal.clone());
    for p in files {
        match config::read_layer(&p) {
            Ok(Some(_)) => read += 1,
            Ok(None) => {}
            Err(e) => {
                return Check::bad(
                    "settings",
                    format!("{}: {}", e.path.display(), e.message),
                    format!(
                        "The file `{}` has a syntax error. Fix it. waymark does not ignore a broken file, because that would loosen a limit.",
                        e.path.display()
                    ),
                )
            }
        }
    }
    if kb.is_some() {
        read += 1;
    }
    Check::ok("settings", format!("{read} files read"))
}

/// Run the checks in their fixed order. A check that cannot run is left out.
pub fn run_checks(ctx: &Context) -> Report {
    let mut checks = Vec::new();
    let state = ctx.join_state();
    let git = git_check();
    let git_ok = git.ok;
    checks.push(git);
    let mut report = Report {
        checks: Vec::new(),
        state: state.clone(),
        kb: None,
        author: None,
    };
    if !git_ok {
        report.checks = checks;
        return report;
    }
    checks.push(clone_check(&state));
    if matches!(state, JoinState::Cloning) {
        report.checks = checks;
        return report;
    }
    // The rest needs an open repository.
    let Ok(repo) = ctx.repo() else {
        report.checks = checks;
        return report;
    };
    checks.push(working_tree_check(&repo));

    match repo.probe_remote(10) {
        Probe::NoRemote => {
            checks.push(Check::ok("remote", "no remote"));
            checks.push(Check::ok("git_login", "no remote"));
        }
        Probe::Reachable => {
            checks.push(Check::ok("remote", "reachable"));
            checks.push(Check::ok("git_login", "The remote answers."));
        }
        Probe::LoginRefused(line) => {
            checks.push(Check::ok("remote", "reachable"));
            checks.push(Check::bad("git_login", line, LOGIN_NEXT));
        }
        Probe::Unreachable(line) => {
            let url = crate::mcp::binding::origin_url(repo.dir())
                .map(|u| crate::identity::redact_url(&u))
                .unwrap_or_default();
            checks.push(Check::bad(
                "remote",
                format!("not reachable: {line}"),
                format!("The remote is not reachable. Check the network and the URL {url}."),
            ));
        }
    }

    let kb = KnowledgeBase::open(repo.dir()).ok();
    if let Some(k) = &kb {
        match &ctx.args.kb_id {
            None => checks.push(Check::ok("kb_id", "no ID expected")),
            Some(want) if want.eq_ignore_ascii_case(&k.settings.id) => {
                checks.push(Check::ok("kb_id", k.settings.id.clone()))
            }
            Some(want) => checks.push(Check::bad(
                "kb_id",
                format!("The ID is {}.", k.settings.id),
                format!(
                    "This repository has the ID `{}`, and the entry expects `{want}`. Check `--repo` and `--kb-id`.",
                    k.settings.id
                ),
            )),
        }
    }
    checks.push(settings_check(ctx, kb.as_ref()));
    match ctx.resolve_author(repo.dir()) {
        Ok(a) => {
            let people = kb
                .as_ref()
                .map(|k| k.settings.people.clone())
                .unwrap_or_default();
            match identity::check_email(&a.email, &people) {
                Ok(()) => checks.push(Check::ok("author", format!("{} <{}>", a.name, a.email))),
                Err(p) => checks.push(Check::bad("author", p.message, p.next)),
            }
            report.author = Some(a);
        }
        Err(p) => checks.push(Check::bad("author", p.message, p.next)),
    }
    report.kb = kb;
    report.checks = checks;
    report
}

/// The layers in the order system, shared, personal, entry, for `limit_sources`.
pub fn limit_sources(ctx: &Context, kb: Option<&KnowledgeBase>) -> Value {
    let mut system: Vec<Limits> = Vec::new();
    for p in &ctx.paths.system {
        if let Ok(Some(l)) = config::read_layer(p) {
            system.push(l);
        }
    }
    let personal = config::read_layer(&ctx.paths.personal).ok().flatten();
    let shared = kb.map(|k| k.settings.limits.clone());
    let entry = ctx.args.entry_layer().unwrap_or_default();
    let mut layers: Vec<Layer> = system
        .iter()
        .map(|l| Layer {
            name: "system",
            limits: l,
        })
        .collect();
    if let Some(s) = &shared {
        layers.push(Layer {
            name: "shared",
            limits: s,
        });
    }
    if let Some(p) = &personal {
        layers.push(Layer {
            name: "personal",
            limits: p,
        });
    }
    layers.push(Layer {
        name: "entry",
        limits: &entry,
    });
    let effective = ctx.limits(kb).effective;
    sources::limit_sources(&layers, &effective)
}
