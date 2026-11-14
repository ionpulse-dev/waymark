//! The `index` tool: generate index files for folders.

use super::read_common::folders_arg;
use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::Verb;
use crate::indexgen::{self, Action};
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::store::{Scope, Store};
use crate::writes::{self, FileChange, Plan};
use serde_json::{json, Map, Value};

pub struct Index;

impl Tool for Index {
    fn name(&self) -> &'static str {
        "index"
    }

    fn description(&self) -> &'static str {
        "Generates an index file for each folder: a list of its subfolders and documents. Changes only the files that are out of date, in one commit."
    }

    fn kind(&self) -> Kind {
        Kind::Write
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folders": {"type": "array", "items": {"type": "string"}},
                "preview": {"type": "boolean"}
            },
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let folders = folders_arg(&args);
        let preview = writes::get_bool(&args, "preview");
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(ctx, env, "index", Kind::Write, preview, |_tx, kb| {
            let store = Store::load(kb);
            let scope = Scope::new(&env.effective, &folders);
            let actions = indexgen::plan(&store, &scope, &author, &today);

            let mut changes = Vec::new();
            let mut written = Vec::new();
            let mut unchanged = 0;
            let mut skipped = Vec::new();
            let mut ids_after = kb.id_set();
            for a in &actions {
                if let Action::Write { id, .. } = a {
                    ids_after.insert(id.clone());
                }
            }
            let mut problems = writes::Problems::default();
            let mut notices = Vec::new();
            for a in actions {
                match a {
                    Action::Write { id, text, created } => {
                        writes::check_document(
                            kb,
                            &id,
                            &text,
                            &ids_after,
                            &[],
                            None,
                            &mut problems,
                            &mut notices,
                        )?;
                        written.push(id.clone());
                        changes.push(FileChange::Write {
                            id,
                            text,
                            action: if created { "create" } else { "update" },
                        });
                    }
                    Action::Unchanged(_) => unchanged += 1,
                    Action::Skipped { id, reason } => {
                        skipped.push(json!({"id": id, "reason": reason}))
                    }
                }
            }
            if !problems.is_empty() {
                return Err(problems.into_error());
            }
            let title = match written.as_slice() {
                [one] => match indexgen::folder_of_index(one) {
                    "" => "Index".to_string(),
                    f => format!("Index of {f}"),
                },
                _ => "Indexes".to_string(),
            };
            let mut data = Map::new();
            data.insert("written".into(), json!(written));
            data.insert("unchanged".into(), json!(unchanged));
            data.insert("skipped".into(), json!(skipped));
            Ok(Plan {
                changes,
                verb: Verb::Update,
                title,
                commit_body: None,
                summary: format!("Write {} index files.", written.len()),
                message: format!(
                    "{} index files written, {unchanged} unchanged.",
                    written.len()
                ),
                data,
                text: None,
                notices,
            })
        })
    }
}
