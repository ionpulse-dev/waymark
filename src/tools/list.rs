//! The `list` tool: list documents by filter.

use super::read_common::{
    cap, entry, folders_arg, level_arg, limit_arg, offset_arg, status_ok, Filter,
};
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
        "Lists documents, sorted by ID, with filters. Returns results (id, title, status, updated), total_matches, truncated, and next_offset. Changes nothing."
    }

    fn kind(&self) -> Kind {
        Kind::Read
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
                "status": {
                    "type": "string",
                    "enum": [
                        "active",
                        "deprecated",
                        "any"
                    ],
                    "description": "Which documents: active, deprecated, or any. Default any."
                },
                "disputed": {
                    "type": "boolean",
                    "description": "true returns only disputed documents."
                },
                "tag": {
                    "type": "string",
                    "description": "Only documents with this tag."
                },
                "author": {
                    "type": "string",
                    "description": "Only documents whose created_by or updated_by contains this text, ignoring case."
                },
                "updated_after": {
                    "type": "string",
                    "description": "Only documents updated on this day or later. YYYY-MM-DD."
                },
                "updated_before": {
                    "type": "string",
                    "description": "Only documents updated on this day or earlier. YYYY-MM-DD."
                },
                "level": {
                    "type": "integer",
                    "enum": [
                        0,
                        1,
                        2
                    ],
                    "description": "Detail of each result: 0 ID only, 1 adds title, status, and updated, 2 adds description. Default 1."
                },
                "limit": {
                    "type": "integer",
                    "description": "How many results. Default and maximum: the max_results limit."
                },
                "offset": {
                    "type": "integer",
                    "description": "How many results to skip. Default 0."
                }
            },
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let level = level_arg(&args, 1);
        let status = crate::writes::get_str(&args, "status").unwrap_or("any");
        let tag = crate::writes::get_str(&args, "tag");
        let filter = Filter::from_args(&args)?;
        let limit = limit_arg(&args, env.effective.max_results)?;
        let offset = offset_arg(&args)?;

        let repo = ctx.repo()?;
        let memrepo = ctx.memrepo()?;
        let notices = repo.refresh_if_stale();
        let store = Store::load(&memrepo);
        let scope = Scope::new(&env.effective, &folders_arg(&args));

        let mut docs: Vec<_> = store
            .docs()
            .filter(|d| scope.allows(&d.id) && status_ok(status, d.status()))
            .filter(|d| filter.matches(d))
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
