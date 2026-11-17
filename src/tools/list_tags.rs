//! The `list_tags` tool: the tags with the number of documents for each (see `specs/features/manage-tags`).

use super::read_common::{folders_arg, offset_arg, status_ok};
use crate::config::Kind;
use crate::error::Error;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::store::{Scope, Store};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub struct ListTags;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 200;

impl Tool for ListTags {
    fn name(&self) -> &'static str {
        "list_tags"
    }

    fn description(&self) -> &'static str {
        "Lists the tags with the number of documents for each. Returns tags (tag, documents), total_tags, truncated, and next_offset. Changes nothing."
    }

    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folders": {"type": "array", "items": {"type": "string"}, "description": "Only these folders. Default: all."},
                "status": {"type": "string", "enum": ["active", "deprecated", "any"], "description": "Which documents to count: active, deprecated, or any. Default active."},
                "limit": {"type": "integer", "description": "How many tags. Default 50, at most 200."},
                "offset": {"type": "integer", "description": "How many results to skip. Default 0."}
            },
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let limit = match args.get("limit").and_then(Value::as_i64) {
            None => DEFAULT_LIMIT,
            Some(n) if n >= 1 => (n as usize).min(MAX_LIMIT),
            Some(_) => {
                return Err(Error::invalid("\"limit\" must be 1 or more.")
                    .with_next("Send a limit of 1 to 200, or leave it out."))
            }
        };
        let offset = offset_arg(&args)?;
        let status = crate::writes::get_str(&args, "status").unwrap_or("active");

        let repo = ctx.repo()?;
        let memrepo = ctx.memrepo()?;
        let notices = repo.refresh_for_read();
        let store = Store::load(&memrepo);
        let scope = Scope::new(&env.effective, &folders_arg(&args));

        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for d in store
            .docs()
            .filter(|d| scope.allows(&d.id) && status_ok(status, d.status()))
        {
            let mut tags = d.tags();
            tags.sort();
            tags.dedup();
            for t in tags {
                *counts.entry(t).or_default() += 1;
            }
        }
        let mut all: Vec<(String, usize)> = counts.into_iter().collect();
        all.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let total = all.len();
        let tags: Vec<Value> = all
            .iter()
            .skip(offset)
            .take(limit)
            .map(|(t, n)| json!({"tag": t, "documents": n}))
            .collect();
        let next = offset + tags.len();
        let truncated = next < total;
        let mut data = json!({"tags": tags, "total_tags": total, "truncated": truncated});
        if truncated {
            data["next_offset"] = json!(next);
        }
        let mut out = ToolOutput::new(
            match total {
                1 => "1 tag.".to_string(),
                n => format!("{n} tags."),
            },
            data,
        );
        for n in notices {
            out = out.notice(n);
        }
        Ok(out)
    }
}
