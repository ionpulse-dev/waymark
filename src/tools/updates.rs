//! The `updates` tool: what others changed since a time (see `specs/features/updates`).

use crate::config::Kind as ToolKind;
use crate::error::Error;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::state::State;
use crate::store::changes::{changes_since, Change, Kind};
use crate::store::scope::Scope;
use crate::writes;
use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{json, Value};

pub struct Updates;

const MAX: usize = 50;

fn group(c: &Change) -> u8 {
    match (c.mine, c.kind == Kind::Disputed) {
        (true, true) => 0,
        (true, false) => 1,
        _ => 2,
    }
}

impl Tool for Updates {
    fn name(&self) -> &'static str {
        "updates"
    }

    fn description(&self) -> &'static str {
        "Lists the documents that others added, changed, moved, deprecated, disputed, or deleted since a time. Returns since, until (pass it as since next time), changes (id, title, kind, by, date), total, truncated, next_offset, and marked. Without since, it uses the time of your last call. Changes nothing except that time."
    }

    fn kind(&self) -> ToolKind {
        ToolKind::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folders": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Only these folders. Default: all."
                },
                "since": {
                    "type": "string",
                    "description": "A date YYYY-MM-DD or an RFC 3339 time. Default: since your last call, or 7 days."
                },
                "offset": {
                    "type": "integer",
                    "description": "How many changes to skip, to read the rest of a long window. Default 0. Needs since."
                }
            },
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let folders = writes::get_list(&args, "folders").unwrap_or_default();
        let explicit = writes::get_str(&args, "since");
        let explicit_when = explicit
            .map(|s| super::read_common::parse_when("since", s, false))
            .transpose()?;

        let offset = super::read_common::offset_arg(&args)?;
        if offset > 0 && explicit.is_none() {
            return Err(Error::invalid("offset needs since.").with_next(
                "Send the same since as in the first call. A call without since moves the stored time.",
            ));
        }

        let repo = ctx.repo()?;
        let memrepo = ctx.memrepo()?;
        let notices = repo.refresh_if_stale();
        let state = State::for_memrepo(&memrepo.settings.id);
        let now = Utc::now();
        let since = match explicit_when {
            Some(w) => w,
            None => state
                .get_str("last_updates")
                .and_then(|t| DateTime::parse_from_rfc3339(&t).ok())
                .map(|t| t.with_timezone(&Utc))
                .unwrap_or(now - chrono::Duration::days(7)),
        };
        let since_text = since.to_rfc3339_opts(SecondsFormat::Secs, true);
        let until_text = now.to_rfc3339_opts(SecondsFormat::Secs, true);

        let own = ctx
            .resolve_author(repo.dir())
            .map(|a| a.email)
            .unwrap_or_default();
        let base = repo.base_commit_before(&since_text);
        let scope = Scope::new(&env.effective, &folders);
        let mut changes: Vec<Change> = changes_since(&repo, &memrepo, &base, &own)?
            .into_iter()
            .filter(|c| scope.allows(&c.id) || c.from.as_deref().is_some_and(|f| scope.allows(f)))
            .collect();
        changes.sort_by(|a, b| {
            group(a)
                .cmp(&group(b))
                .then_with(|| b.date.cmp(&a.date))
                .then_with(|| a.id.cmp(&b.id))
        });
        let total = changes.len();
        let shown_from = offset.min(total);
        changes.drain(..shown_from);
        let truncated = shown_from + MAX < total;
        changes.truncate(MAX);
        let next_offset = shown_from + changes.len();

        let marked = explicit.is_none();
        let mut extra = Vec::new();
        if marked {
            if let Err(e) = state.set("last_updates", json!(until_text)) {
                extra.push(format!("The time of this call was not stored: {e}."));
            }
            ctx.notices.invalidate();
        }

        let disputed_mine = changes
            .iter()
            .filter(|c| c.mine && c.kind == Kind::Disputed)
            .count();
        let mut message = format!(
            "{total} change{} since {}.",
            if total == 1 { "" } else { "s" },
            &since_text[..10]
        );
        if disputed_mine > 0 {
            message.push_str(&format!(
                " {disputed_mine} of your document{} {} disputed.",
                if disputed_mine == 1 { "" } else { "s" },
                if disputed_mine == 1 { "is" } else { "are" }
            ));
        }
        message.push_str(" waymark did not check these changes for truth.");

        let entries: Vec<Value> = changes
            .iter()
            .map(|c| {
                let mut v = json!({
                    "id": c.id, "title": c.title, "kind": c.kind.name(), "mine": c.mine,
                    "by": c.by, "date": c.date,
                });
                if let Some(r) = &c.reason {
                    v["reason"] = json!(r);
                }
                if c.kind == Kind::Moved {
                    if let Some(f) = &c.from {
                        v["from"] = json!(f);
                    }
                }
                if let Some(t) = &c.trust {
                    v["trust"] = json!(t);
                }
                v
            })
            .collect();
        let mut data = json!({
            "since": since_text, "until": until_text, "changes": entries,
            "total": total, "truncated": truncated, "marked": marked,
        });
        if truncated {
            data["next_offset"] = json!(next_offset);
        }
        let mut out = ToolOutput::new(message, data);
        for n in notices.into_iter().chain(extra) {
            out = out.notice(n);
        }
        Ok(out)
    }
}
