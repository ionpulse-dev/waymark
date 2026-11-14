//! The facts of a session: the memory repository, the limits, and the notices (see `session-start`).

use super::binding::FoundBy;
use super::context::Context;
use super::repo_url;
use crate::config::Effective;
use crate::error::Error;
use serde_json::{json, Value};

pub struct KbFacts {
    pub name: String,
    pub id: String,
    pub slug: String,
    pub path: String,
    pub remote: Option<String>,
    pub found_by: Option<FoundBy>,
}

pub struct Facts {
    pub memrepo: Option<KbFacts>,
    pub problem: Option<Error>,
    pub settings: Value,
    pub notices: Vec<String>,
}

/// The `settings` object. The keys and their order are fixed (see `Effective::to_json`).
pub fn settings_json(e: &Effective) -> Value {
    e.to_json()
}

fn local_notices(
    effective: &Effective,
    memrepo_notices: &[String],
    has_remote: Option<bool>,
    found_by: Option<FoundBy>,
) -> Vec<String> {
    let mut out = Vec::new();
    if effective.paused {
        out.push(crate::notices::text::PAUSED.to_string());
    }
    if has_remote == Some(false) {
        out.push("This repository has no remote.".to_string());
    }
    out.extend(memrepo_notices.iter().cloned());
    match found_by {
        Some(FoundBy::WorkingFolder) => {
            out.push("The memory repository was found in the working folder.".to_string())
        }
        Some(FoundBy::DefaultFolder) => {
            out.push("The memory repository was found in the default folder.".to_string())
        }
        _ => {}
    }
    out
}

/// Gather the facts. Local data only: no network. With `allow_clone` false, a clone that is still
/// missing counts as "not set up yet", so that `initialize` never waits for a clone.
pub fn facts(ctx: &Context, allow_clone: bool) -> Facts {
    let mut f = facts_inner(ctx, allow_clone);
    f.notices.extend(ctx.args.entry_notices());
    f
}

fn facts_inner(ctx: &Context, allow_clone: bool) -> Facts {
    let build_unbound = |problem: Error| {
        let effective = ctx.limits(None).effective;
        Facts {
            memrepo: None,
            notices: local_notices(&effective, &[], None, None),
            problem: Some(problem),
            settings: settings_json(&effective),
        }
    };
    if let Some(i) = ctx.args.blocking_issue() {
        return build_unbound(Error::needs_update(i.message.clone(), i.next.clone()));
    }
    if ctx.is_cloning() {
        return build_unbound(
            Error::setup("The first clone is still running.").with_next("Call status in a minute."),
        );
    }
    if !allow_clone {
        match super::binding::locate(&ctx.args) {
            Ok(f) if f.clone_url.is_some() => {
                return build_unbound(
                    Error::setup("The memory repository is cloned at the first tool call.")
                        .with_next("Call status after the first tool call."),
                )
            }
            Err(e) => return build_unbound(e),
            _ => {}
        }
    }
    let repo = match ctx.repo() {
        Ok(r) => r,
        Err(e) => return build_unbound(e),
    };
    let memrepo = match ctx.memrepo() {
        Ok(k) => k,
        Err(e) => return build_unbound(e),
    };
    let effective = ctx.limits(Some(&memrepo)).effective;
    let origin = super::binding::origin_url(repo.dir());
    let found_by = ctx.found_by();
    Facts {
        notices: local_notices(
            &effective,
            &memrepo.settings.notices,
            Some(origin.is_some()),
            found_by,
        ),
        problem: None,
        settings: settings_json(&effective),
        memrepo: Some(KbFacts {
            name: memrepo.settings.name.clone(),
            id: memrepo.settings.id.clone(),
            slug: memrepo.settings.slug.clone(),
            path: repo.dir().display().to_string(),
            remote: origin.map(|o| repo_url::redact(&o)),
            found_by,
        }),
    }
}

impl KbFacts {
    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "id": self.id,
            "slug": self.slug,
            "path": self.path,
            "remote": self.remote,
            "found_by": self.found_by.map(|f| f.name()),
        })
    }
}
