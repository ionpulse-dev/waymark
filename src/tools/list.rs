//! The `list` tool: list documents by filter.

use super::read_common::{cap, entry, folders_arg, level_arg, limit_arg, status_ok};
use crate::config::Kind;
use crate::error::Error;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::store::{Scope, Store};
use serde_json::{json, Value};

pub struct List;

impl Tool for List {
    fn name(&self) -> &'static str {
        "list"
    }

    fn description(&self) -> &'static str {
        "Lists documents, sorted by ID, with filters for folder, status, tag, and disputes. Returns only a few per call. Use offset to read on. Changes nothing."
    }

    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folders": {"type": "array", "items": {"type": "string"}},
                "status": {"type": "string", "enum": ["active", "deprecated", "any"]},
                "disputed": {"type": "boolean"},
                "tag": {"type": "string"},
                "level": {"type": "integer", "enum": [0, 1, 2]},
                "limit": {"type": "integer"},
                "offset": {"type": "integer"}
            },
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let level = level_arg(&args, 1);
        let status = crate::writes::get_str(&args, "status").unwrap_or("any");
        let tag = crate::writes::get_str(&args, "tag");
        let only_disputed = crate::writes::get_bool(&args, "disputed");
        let limit = limit_arg(&args, env.effective.max_results)?;
        let offset = args.get("offset").and_then(Value::as_i64).unwrap_or(0);
        if offset < 0 {
            return Err(Error::invalid("\"offset\" must be 0 or more.")
                .with_next("Send an offset of 0 or more."));
        }
        let offset = offset as usize;

        let repo = ctx.repo()?;
        let kb = ctx.kb()?;
        let notices = repo.refresh_if_stale();
        let store = Store::load(&kb);
        let scope = Scope::new(&env.effective, &folders_arg(&args));

        let mut docs: Vec<_> = store
            .docs()
            .filter(|d| scope.allows(&d.id) && status_ok(status, d.status()))
            .filter(|d| !only_disputed || d.disputed())
            .filter(|d| tag.is_none_or(|t| d.tags().iter().any(|x| x == t)))
            .collect();
        docs.sort_by(|a, b| a.id.cmp(&b.id));
        let total = docs.len();
        let entries = docs
            .iter()
            .skip(offset)
            .take(limit + 1)
            .map(|d| entry(d, level))
            .collect();
        let results = cap(entries, limit, env.effective.max_tokens);
        let shown = results.len();
        let next = offset + shown;
        let truncated = next < total;
        let mut data = json!({"results": results, "total_matches": total, "truncated": truncated});
        if truncated {
            data["next_offset"] = json!(next);
        }
        let mut out = ToolOutput::new(format!("{shown} of {total} documents."), data);
        for n in notices {
            out = out.notice(n);
        }
        Ok(out)
    }
}
