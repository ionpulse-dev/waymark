//! The `stats` tool: counts the size of the memory repository.

use super::read_common::folders_arg;
use crate::config::Kind;
use crate::error::Error;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::store::stats::{folder_key, group, Metric};
use crate::store::{Scope, Store};
use serde_json::{json, Value};

pub struct Stats;

impl Tool for Stats {
    fn name(&self) -> &'static str {
        "stats"
    }

    fn description(&self) -> &'static str {
        "Counts words, tokens, characters, or documents, in total or by folder or author. Returns metric, total, documents, and the groups. Changes nothing."
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
                "metric": {
                    "type": "string",
                    "enum": [
                        "words",
                        "tokens",
                        "chars",
                        "documents"
                    ],
                    "description": "What to count. Default words."
                },
                "by": {
                    "type": "string",
                    "enum": [
                        "none",
                        "folder",
                        "author"
                    ],
                    "description": "How to group the count. Default none."
                }
            },
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let metric = Metric::parse(crate::writes::get_str(&args, "metric").unwrap_or("words"))
            .unwrap_or(Metric::Words);
        let by = crate::writes::get_str(&args, "by").unwrap_or("none");
        let folders = folders_arg(&args);

        let repo = ctx.repo()?;
        let memrepo = ctx.memrepo()?;
        let notices = repo.refresh_if_stale();
        let store = Store::load(&memrepo);
        let scope = Scope::new(&env.effective, &folders);
        let docs: Vec<_> = store.docs().filter(|d| scope.allows(&d.id)).collect();
        let total: usize = docs.iter().map(|d| metric.of(d)).sum();
        let count = docs.len();

        let mut data = json!({"metric": metric.name(), "total": total, "documents": count});
        if by != "none" {
            let bases: Vec<String> = folders
                .iter()
                .map(|f| f.trim_matches('/').to_string())
                .collect();
            let key: Box<dyn Fn(&crate::store::Doc) -> String> = if by == "folder" {
                Box::new(move |d| folder_key(&d.id, &bases))
            } else {
                Box::new(|d| d.text("created_by").unwrap_or("(unknown)").to_string())
            };
            let groups = group(docs.iter().copied(), metric, key.as_ref());
            let truncated = groups.len() > 50;
            let groups: Vec<Value> = groups
                .into_iter()
                .take(50)
                .map(|(k, v, n)| json!({"key": k, "value": v, "documents": n}))
                .collect();
            data["groups"] = json!(groups);
            data["groups_truncated"] = json!(truncated);
        }
        let message = if metric == Metric::Documents {
            format!("{count} documents.")
        } else {
            format!("{count} documents, {total} {}.", metric.name())
        };
        let mut out = ToolOutput::new(message, data);
        for n in notices {
            out = out.notice(n);
        }
        Ok(out)
    }
}
