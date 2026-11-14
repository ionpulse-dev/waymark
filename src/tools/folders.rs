//! The `folders` tool: the tree of folders with the number of documents
//! (see `specs/features/folders`).

use crate::config::Kind;
use crate::error::Error;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use serde_json::{json, Value};

pub struct Folders;

const MAX_ENTRIES: usize = 200;

impl Tool for Folders {
    fn name(&self) -> &'static str {
        "folders"
    }

    fn description(&self) -> &'static str {
        "Shows the folders of the knowledge base as a tree, with the number of documents in each. Use depth to go deeper, and folder to start below a folder. Changes nothing."
    }

    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folder": {"type": "string", "description": "Start below this folder. Default: the root."},
                "depth": {"type": "integer", "description": "Levels below the start, 1 to 10. Default 2."}
            },
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let start = crate::writes::get_str(&args, "folder")
            .unwrap_or("")
            .trim_matches('/')
            .to_string();
        let depth = match args.get("depth").and_then(Value::as_i64) {
            None => 2,
            Some(n) if (1..=10).contains(&n) => n as usize,
            Some(_) => {
                return Err(Error::invalid("\"depth\" must be from 1 to 10.")
                    .with_next("Send a depth from 1 to 10, or leave it out."))
            }
        };
        let repo = ctx.repo()?;
        let kb = ctx.kb()?;
        let notices = repo.refresh_if_stale();
        let all = kb.folders();
        if !start.is_empty() && !all.iter().any(|f| f.path == start) {
            return Err(
                crate::error::Error::not_found(format!("There is no folder \"{start}\"."))
                    .with_next("Call folders without `folder` to see the folders."),
            );
        }
        let allowed = |id: &str| env.effective.allows_folder(id);
        let under = |path: &str| {
            start.is_empty() || path == start || path.starts_with(&format!("{start}/"))
        };
        let start_level = if start.is_empty() {
            0
        } else {
            start.split('/').count()
        };
        let level_of = |path: &str| path.split('/').count() - start_level;

        // Counts for the allowed documents only.
        let direct = |path: &str| -> usize {
            all.iter()
                .find(|f| f.path == path)
                .map(|f| f.documents.iter().filter(|d| allowed(d)).count())
                .unwrap_or(0)
        };
        let total = |path: &str| -> usize {
            all.iter()
                .filter(|f| f.path == path || f.path.starts_with(&format!("{path}/")))
                .map(|f| f.documents.iter().filter(|d| allowed(d)).count())
                .sum()
        };
        // A folder is visible when a limit allows something in it, or below it.
        let visible = |path: &str| -> bool {
            allowed(&format!("{path}/."))
                || all.iter().any(|f| {
                    f.path.starts_with(&format!("{path}/")) && allowed(&format!("{}/.", f.path))
                })
        };

        let candidates: Vec<&crate::kb::folders::Folder> = all
            .iter()
            .filter(|f| f.path != start && under(&f.path) && visible(&f.path))
            .collect();
        let mut entries = Vec::new();
        let mut truncated = false;
        for f in candidates.iter().filter(|f| level_of(&f.path) <= depth) {
            if entries.len() >= MAX_ENTRIES {
                truncated = true;
                break;
            }
            let has_more = level_of(&f.path) == depth
                && candidates
                    .iter()
                    .any(|c| c.path.starts_with(&format!("{}/", f.path)));
            entries.push(json!({
                "path": f.path,
                "documents": direct(&f.path),
                "total": total(&f.path),
                "has_more": has_more,
            }));
        }
        let mut data = json!({
            "folder": start,
            "depth": depth,
            "folders": entries,
            "truncated": truncated,
        });
        let n = data["folders"].as_array().map(Vec::len).unwrap_or(0);
        if truncated {
            data["next"] = json!(
                "Start below one folder with `folder`, or use a lower depth, to see the rest."
            );
        }
        let mut out = ToolOutput::new(format!("{n} folders."), data);
        for note in notices {
            out = out.notice(note);
        }
        Ok(out)
    }
}
