//! Notices in tool results (see `specs/features/notifications-and-review`).
//!
//! The registry hands every result through `finalize`. It adds the system notices, removes
//! repeats, and applies the cap of three sessions.

pub mod text;

use crate::mcp::context::Context;
use crate::state::State;
use crate::store::changes::{changes_since, Kind};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Mutex;

/// A notice about the documents of the person.
#[derive(Debug, Clone)]
pub struct DocNotice {
    pub text: String,
}

#[derive(Default)]
pub struct NoticeBook {
    /// The keys of the notices shown in this session.
    seen: Mutex<HashSet<String>>,
    /// The notices about documents, with the commit they were computed for.
    cache: Mutex<Option<(String, Vec<DocNotice>)>>,
}

impl NoticeBook {
    /// Forget the computed notices, for example after `updates` stored a new time.
    pub fn invalidate(&self) {
        *self.cache.lock().unwrap() = None;
    }
}

fn key_of(text: &str) -> String {
    let h = Sha256::digest(text.as_bytes());
    h.iter().take(4).map(|b| format!("{b:02x}")).collect()
}

/// The notices about the documents of the person: disputes, and changes by others.
fn document_notices(ctx: &Context) -> Vec<DocNotice> {
    let (Some(memrepo), Ok(repo)) = (ctx.memrepo_if_ready(), ctx.repo()) else {
        return Vec::new();
    };
    let Some(head) = repo.head() else {
        return Vec::new();
    };
    if let Some((h, list)) = ctx.notices.cache.lock().unwrap().as_ref() {
        if *h == head {
            return list.clone();
        }
    }
    let own = ctx
        .resolve_author(repo.dir())
        .map(|a| a.email)
        .unwrap_or_default();
    if own.is_empty() {
        return Vec::new();
    }
    let state = State::for_memrepo(&memrepo.settings.id);
    let since = state
        .get_str("last_updates")
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
        .map(|t| t.with_timezone(&chrono::Utc))
        .unwrap_or(chrono::Utc::now() - chrono::Duration::days(7));
    let base = repo.base_commit_before(&since.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
    let changes = match changes_since(&repo, &memrepo, &base, &own) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "waymark: cannot check your documents for notices: {}",
                e.message
            );
            return Vec::new();
        }
    };
    let mut disputed = Vec::new();
    let mut changed = Vec::new();
    for c in changes.iter().filter(|c| c.mine) {
        match c.kind {
            Kind::Disputed => disputed.push(c.id.clone()),
            Kind::Changed if c.same_content => {}
            Kind::Changed | Kind::Moved | Kind::Deprecated | Kind::Deleted => {
                changed.push(c.id.clone())
            }
            Kind::New => {}
        }
    }
    let mut list = Vec::new();
    if !disputed.is_empty() {
        list.push(DocNotice {
            text: text::disputed(&disputed),
        });
    }
    if !changed.is_empty() {
        list.push(DocNotice {
            text: text::changed(&changed),
        });
    }
    *ctx.notices.cache.lock().unwrap() = Some((head, list.clone()));
    list
}

/// All open notices that the system knows, with no repeat filter and no cap. `status` uses it.
pub fn open_items(ctx: &Context) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(repo) = ctx.repo() {
        if repo.last_fetch_failed() {
            out.push(text::CLONE_BEHIND.to_string());
        }
    }
    out.extend(document_notices(ctx).into_iter().map(|n| n.text));
    out
}

/// Count a notice about documents for this session. `false` if it was shown in 3 sessions already.
fn count_session(ctx: &Context, text: &str, current: &[String]) -> bool {
    let Some(memrepo) = ctx.memrepo_if_ready() else {
        return true;
    };
    let state = State::for_memrepo(&memrepo.settings.id);
    let key = key_of(text);
    let mut map = state
        .get("notice_sessions")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    let n = map.get(&key).and_then(Value::as_u64).unwrap_or(0);
    if n >= 3 {
        return false;
    }
    // Keep only the keys of notices that are still open.
    let keep: HashSet<String> = current.iter().map(|t| key_of(t)).collect();
    map.retain(|k, _| keep.contains(k));
    map.insert(key, json!(n + 1));
    let _ = state.set("notice_sessions", Value::Object(map));
    true
}

/// The final list of notices for one result. `tool_notices` are the notices that the registry and
/// the tool made. The result is empty if there is nothing to say.
pub fn finalize(ctx: &Context, tool: &str, tool_notices: Vec<String>) -> Vec<String> {
    let is_status = tool == "status";
    let mut candidates: Vec<String> = tool_notices
        .into_iter()
        .map(|n| {
            if n.contains("may be behind") {
                text::CLONE_BEHIND.to_string()
            } else {
                n
            }
        })
        .collect();
    let mut docs: Vec<String> = Vec::new();
    if ctx.memrepo_if_ready().is_some() {
        if let Ok(repo) = ctx.repo() {
            if repo.last_fetch_failed() {
                candidates.push(text::CLONE_BEHIND.to_string());
            }
        }
        if tool != "updates" && tool != "status" {
            docs = document_notices(ctx).into_iter().map(|n| n.text).collect();
        }
    }
    if is_status {
        // `status` lists everything that is open (see `maintenance`).
        candidates.extend(open_items(ctx));
    }
    let mut out: Vec<String> = Vec::new();
    {
        let mut seen = ctx.notices.seen.lock().unwrap();
        for n in candidates {
            if out.contains(&n) {
                continue;
            }
            if is_status || seen.insert(key_of(&n)) {
                out.push(n);
            }
        }
        let current = docs.clone();
        for n in docs {
            if seen.contains(&key_of(&n)) {
                continue;
            }
            seen.insert(key_of(&n));
            if count_session(ctx, &n, &current) {
                out.push(n);
            }
        }
    }
    out
}
