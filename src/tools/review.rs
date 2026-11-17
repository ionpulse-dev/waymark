//! The `review` tool: documents that wait for review.

use crate::config::Kind;
use crate::error::Error;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::store::{Scope, Store};
use crate::writes;
use serde_json::{json, Value};

pub struct Review;

impl Tool for Review {
    fn name(&self) -> &'static str {
        "review"
    }

    fn description(&self) -> &'static str {
        "Lists active documents that are unverified or disputed, disputed first, oldest first. Returns entries (id, title, trust, disputed, updated, version) and total. Changes nothing. Exists only when the memory repository uses verification."
    }

    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn needs_verification(&self) -> bool {
        true
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folder": {
                    "type": "string",
                    "description": "Only documents in this folder. Default: all."
                },
                "limit": {
                    "type": "integer",
                    "description": "How many entries. Default 10, at most 50."
                }
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
        let folders: Vec<String> = writes::get_str(&args, "folder")
            .map(|f| vec![f.to_string()])
            .unwrap_or_default();
        let repo = ctx.repo()?;
        let memrepo = ctx.memrepo()?;
        let notices = repo.refresh_for_read();
        let store = Store::load(&memrepo);
        let scope = Scope::new(&env.effective, &folders);
        let mut docs: Vec<_> = store
            .docs()
            .filter(|d| scope.allows(&d.id) && d.status() == "active")
            .filter(|d| d.disputed() || d.text("trust") != Some("verified"))
            .collect();
        docs.sort_by(|a, b| {
            b.disputed()
                .cmp(&a.disputed())
                .then_with(|| a.updated().cmp(b.updated()))
                .then_with(|| a.id.cmp(&b.id))
        });
        let total = docs.len();
        let mut entries = Vec::new();
        for d in docs.into_iter().take(limit) {
            let rel = memrepo.repo_relative(&d.id)?;
            entries.push(json!({
                "id": d.id,
                "title": d.title(),
                "trust": d.text("trust").unwrap_or("unverified"),
                "disputed": d.disputed(),
                "dispute_reason": if d.disputed() { json!(d.text("dispute_reason").unwrap_or("")) } else { Value::Null },
                "updated": d.updated(),
                "updated_by": d.text("updated_by").unwrap_or(""),
                "version": repo.last_commit(&rel)?,
            }));
        }
        let mut out = ToolOutput::new(
            format!("{} of {total} documents wait for review.", entries.len()),
            json!({"entries": entries, "total": total}),
        );
        for n in notices {
            out = out.notice(n);
        }
        Ok(out)
    }
}
