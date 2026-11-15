//! The `search` tool: full-text search, ranked. Without a query it lists documents (see `lean-tools`).

use super::read_common::{
    cap, entry, folders_arg, level_arg, limit_arg, offset_arg, status_ok, Filter,
};
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
        "Searches the documents for words and returns the best matches, ranked, with a snippet. Without a query it lists the documents sorted by ID. Returns results (id, title, status, updated, description, snippet), total_matches, truncated, and next_offset. Changes nothing."
    }

    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "One or more words. A document must contain all of them. Leave it out to list documents."
                },
                "in": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "enum": [
                            "path",
                            "title",
                            "description",
                            "tags",
                            "body"
                        ]
                    },
                    "description": "Where to look, with a query. Default: all of them."
                },
                "level": {
                    "type": "integer",
                    "enum": [
                        0,
                        1,
                        2
                    ],
                    "description": "Detail of each result: 0 ID only, 1 adds title, status, and updated, 2 adds description and snippet. Default 2 with a query, 1 without."
                },
                "folders": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Only these folders. Default: all."
                },
                "tag": {
                    "type": "string",
                    "description": "Only documents with this tag."
                },
                "status": {
                    "type": "string",
                    "enum": [
                        "active",
                        "deprecated",
                        "any"
                    ],
                    "description": "Which documents: active, deprecated, or any. Default active."
                },
                "disputed": {
                    "type": "boolean",
                    "description": "true returns only disputed documents."
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
        let terms = rank::terms(args["query"].as_str().unwrap_or(""));
        // No `query` at all lists documents. A `query` without words is a mistake.
        let listing = args.get("query").is_none();
        if terms.is_empty() && !listing {
            return Err(Error::invalid("The query is empty.")
                .with_next("Send one or more words in query, or leave query out to list."));
        }
        let parts: Vec<Part> = match crate::writes::get_list(&args, "in") {
            Some(l) if !l.is_empty() => l.iter().filter_map(|p| Part::parse(p)).collect(),
            _ => Part::ALL.to_vec(),
        };
        let level = level_arg(&args, if listing { 1 } else { 2 });
        let status = crate::writes::get_str(&args, "status").unwrap_or("active");
        let tag = crate::writes::get_str(&args, "tag");
        let limit = limit_arg(&args, env.effective.max_results)?;
        let offset = offset_arg(&args)?;
        let filter = Filter::from_args(&args)?;

        let repo = ctx.repo()?;
        let memrepo = ctx.memrepo()?;
        let notices = repo.refresh_if_stale();
        let store = Store::load(&memrepo);
        let scope = Scope::new(&env.effective, &folders_arg(&args));

        let candidates = store
            .docs()
            .filter(|d| scope.allows(&d.id) && status_ok(status, d.status()))
            .filter(|d| tag.is_none_or(|t| d.tags().iter().any(|x| x == t)))
            .filter(|d| filter.matches(d));
        let docs: Vec<_> = if listing {
            let mut docs: Vec<_> = candidates.collect();
            docs.sort_by(|a, b| a.id.cmp(&b.id));
            docs
        } else {
            let mut hits: Vec<_> = candidates
                .filter_map(|d| rank::score(d, &terms, &parts).map(|s| (d, s)))
                .collect();
            rank::sort(&mut hits);
            hits.into_iter().map(|(d, _)| d).collect()
        };
        let total = docs.len();
        let entries = docs
            .iter()
            .skip(offset)
            .take(limit + 1)
            .map(|d| {
                let mut e = entry(d, level);
                if level >= 2 && !listing {
                    e.insert("snippet".into(), json!(rank::snippet(d, &terms)));
                }
                e
            })
            .collect();
        let results = cap(entries, limit, env.effective.max_tokens);
        let next = offset + results.len();
        let truncated = next < total;
        let message = match total {
            _ if listing => format!("{} of {total} documents.", results.len()),
            0 => "No document found.".to_string(),
            1 => "1 document found.".to_string(),
            n => format!("{n} documents found."),
        };
        let mut data = json!({"results": results, "total_matches": total, "truncated": truncated});
        if truncated {
            data["next_offset"] = json!(next);
        }
        let mut out = ToolOutput::new(message, data);
        for n in notices {
            out = out.notice(n);
        }
        Ok(out)
    }
}
