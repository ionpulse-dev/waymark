//! The `activity` tool: the recent writes of the person, from Git history.

use crate::activity;
use crate::config::Kind;
use crate::error::Error;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes;
use serde_json::{json, Value};

pub struct Activity;

impl Tool for Activity {
    fn name(&self) -> &'static str {
        "activity"
    }

    fn description(&self) -> &'static str {
        "Lists the recent writes under your account, newest first, with the commit to give to undo. Changes nothing."
    }

    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "limit": {"type": "integer", "description": "Default 10, at most 50."},
                "since": {"type": "string", "description": "A date YYYY-MM-DD."},
                "before": {"type": "string", "description": "A commit from an earlier result. Shows older entries."}
            },
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let limit = match args.get("limit").and_then(Value::as_i64) {
            None => 10,
            Some(n) if n >= 1 => (n as usize).min(50),
            Some(_) => {
                return Err(Error::invalid("\"limit\" must be 1 or more.")
                    .with_next("Send a limit of 1 to 50."))
            }
        };
        let since = writes::get_str(&args, "since");
        if let Some(s) = since {
            if !crate::format::fields::is_valid_date(s) {
                return Err(Error::invalid("\"since\" is not a date.")
                    .with_next("Use the form YYYY-MM-DD."));
            }
        }
        let before = writes::get_str(&args, "before");

        let repo = ctx.repo()?;
        let memrepo = ctx.memrepo()?;
        let notices = repo.refresh_if_stale();
        let commits = repo.all_commits()?;
        let mut entries = activity::entries(&memrepo, &commits, &repo.author().email);
        entries.retain(|e| e.ids.iter().any(|id| env.effective.allows_folder(id)));
        if let Some(s) = since {
            entries.retain(|e| e.time.get(..10).unwrap_or("") >= s);
        }
        if let Some(b) = before {
            let Some(pos) = commits
                .iter()
                .position(|c| c.hash.starts_with(b) || b.starts_with(&c.short))
            else {
                return Err(Error::invalid(
                    "\"before\" is not a commit of this memory repository.",
                )
                .with_next("Use a commit from an earlier activity result."));
            };
            entries.retain(|e| e.index > pos);
        }
        let more = entries.len() > limit;
        entries.truncate(limit);
        let next_before = if more {
            entries.last().map(|e| e.commit.clone())
        } else {
            None
        };
        let list: Vec<Value> = entries
            .iter()
            .map(|e| {
                json!({
                    "commit": e.commit,
                    "time": e.time,
                    "verb": e.verb.as_str(),
                    "title": e.title,
                    "folder": e.folder,
                    "ids": e.ids.iter().take(5).collect::<Vec<_>>(),
                    "files": e.files,
                    "undone": e.undone,
                    "later_changes": e.later_changes,
                })
            })
            .collect();
        let message = match list.len() {
            0 => "No writes found.".to_string(),
            1 => "1 write found.".to_string(),
            n => format!("{n} writes found."),
        };
        let mut out = ToolOutput::new(
            message,
            json!({"entries": list, "next_before": next_before}),
        );
        for n in notices {
            out = out.notice(n);
        }
        Ok(out)
    }
}
