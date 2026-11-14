//! The `search` tool: full-text search, ranked.

use super::read_common::{cap, entry, folders_arg, level_arg, limit_arg, status_ok};
use crate::config::Kind;
use crate::error::Error;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::store::rank::{self, Part};
use crate::store::{Scope, Store};
use serde_json::{json, Value};

pub struct Search;

impl Tool for Search {
    fn name(&self) -> &'static str {
        "search"
    }

    fn description(&self) -> &'static str {
        "Searches the documents for words and returns the best matches, ranked, with a snippet. Returns only a few results. Changes nothing."
    }

    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "One or more words. A document must contain all of them."},
                "in": {"type": "array", "items": {"type": "string", "enum": ["path", "title", "description", "tags", "body"]}},
                "level": {"type": "integer", "enum": [0, 1, 2]},
                "folders": {"type": "array", "items": {"type": "string"}},
                "tag": {"type": "string"},
                "status": {"type": "string", "enum": ["active", "deprecated", "any"]},
                "limit": {"type": "integer"}
            },
            "required": ["query"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let terms = rank::terms(args["query"].as_str().unwrap_or(""));
        if terms.is_empty() {
            return Err(
                Error::invalid("The query is empty.").with_next("Send one or more words in query.")
            );
        }
        let parts: Vec<Part> = match crate::writes::get_list(&args, "in") {
            Some(l) if !l.is_empty() => l.iter().filter_map(|p| Part::parse(p)).collect(),
            _ => Part::ALL.to_vec(),
        };
        let level = level_arg(&args, 2);
        let status = crate::writes::get_str(&args, "status").unwrap_or("active");
        let tag = crate::writes::get_str(&args, "tag");
        let limit = limit_arg(&args, env.effective.max_results)?;

        let repo = ctx.repo()?;
        let memrepo = ctx.memrepo()?;
        let notices = repo.refresh_if_stale();
        let store = Store::load(&memrepo);
        let scope = Scope::new(&env.effective, &folders_arg(&args));

        let mut hits: Vec<_> = store
            .docs()
            .filter(|d| scope.allows(&d.id) && status_ok(status, d.status()))
            .filter(|d| tag.is_none_or(|t| d.tags().iter().any(|x| x == t)))
            .filter_map(|d| rank::score(d, &terms, &parts).map(|s| (d, s)))
            .collect();
        rank::sort(&mut hits);
        let total = hits.len();
        let entries = hits
            .iter()
            .take(limit + 1)
            .map(|(d, _)| {
                let mut e = entry(d, level);
                if level >= 2 {
                    e.insert("snippet".into(), json!(rank::snippet(d, &terms)));
                }
                e
            })
            .collect();
        let results = cap(entries, limit, env.effective.max_tokens);
        let truncated = results.len() < total;
        let message = match total {
            0 => "No document found.".to_string(),
            1 => "1 document found.".to_string(),
            n => format!("{n} documents found."),
        };
        let mut out = ToolOutput::new(
            message,
            json!({"results": results, "total_matches": total, "truncated": truncated}),
        );
        for n in notices {
            out = out.notice(n);
        }
        Ok(out)
    }
}
