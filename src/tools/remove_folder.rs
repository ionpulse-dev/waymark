//! The `remove_folder` tool: remove a folder with everything in it
//! (see `specs/features/folders`).

use crate::config::Kind;
use crate::error::{Code, Error};
use crate::format;
use crate::gitsync::Verb;
use crate::kb::folders::GITKEEP;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes::{self, FileChange, Plan};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

pub struct RemoveFolder;

impl Tool for RemoveFolder {
    fn name(&self) -> &'static str {
        "remove_folder"
    }

    fn description(&self) -> &'static str {
        "Removes a folder with all documents in it, in one commit. Needs expect_documents: the number of documents it will delete, as a preview shows. Refuses while a document outside the folder links into it."
    }

    fn kind(&self) -> Kind {
        Kind::Destructive
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folder": {"type": "string", "description": "The folder below the root."},
                "expect_documents": {"type": "integer", "description": "How many documents the folder holds, also in the folders below it. 0 for an empty folder."},
                "preview": {"type": "boolean"}
            },
            "required": ["folder", "expect_documents"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let path = writes::check_folder_path(writes::get_str(&args, "folder").unwrap_or(""))?;
        let expect = match args.get("expect_documents").and_then(Value::as_i64) {
            Some(n) if n >= 0 => n as usize,
            _ => {
                return Err(
                    Error::invalid("The field `expect_documents` is wrong: use 0 or more.")
                        .with_next("Send the number of documents that a preview shows."),
                )
            }
        };
        let preview = writes::get_bool(&args, "preview");
        writes::author_string(ctx)?;

        writes::execute(
            ctx,
            env,
            "remove_folder",
            Kind::Destructive,
            preview,
            |_tx, kb| {
                let prefix = format!("{path}/");
                let inside: Vec<_> = kb
                    .folders()
                    .into_iter()
                    .filter(|f| f.path == path || f.path.starts_with(&prefix))
                    .collect();
                if inside.is_empty() {
                    return Err(Error::not_found(format!("There is no folder \"{path}\"."))
                        .with_next("Call folders to see the folders."));
                }
                let docs: Vec<String> = inside.iter().flat_map(|f| f.documents.clone()).collect();
                let index_files: Vec<String> =
                    inside.iter().flat_map(|f| f.index_files.clone()).collect();
                if let Some(other) = inside.iter().flat_map(|f| f.other_files.iter()).next() {
                    return Err(Error::validation(format!(
                        "The folder holds a file that is not a document: {other}."
                    ))
                    .with_next("Remove that file by hand, then call again."));
                }
                if let Some(hidden) = docs.iter().find(|d| !env.effective.allows_folder(d)) {
                    return Err(Error::new(
                        Code::AccessDenied,
                        format!("The document \"{hidden}\" is outside the allowed folders."),
                    )
                    .with_next("Ask someone with wider access to remove this folder."));
                }
                if docs.is_empty() && !env.effective.allows_folder(&format!("{path}/.")) {
                    return Err(Error::new(
                        Code::AccessDenied,
                        format!("The folder \"{path}\" is outside the allowed folders."),
                    )
                    .with_next("Use a folder that the limits allow."));
                }
                if docs.len() != expect {
                    return Err(Error::validation(format!(
                    "The folder \"{path}\" holds {} document{}, not {expect}. Nothing was changed.",
                    docs.len(),
                    if docs.len() == 1 { "" } else { "s" },
                ))
                    .with_next(format!(
                        "Check the list with preview, then call again with expect_documents {}.",
                        docs.len()
                    )));
                }

                // Links from outside the folder into it.
                let inside_ids: BTreeSet<&String> = docs.iter().chain(index_files.iter()).collect();
                let mut linked = Vec::new();
                for id in &docs {
                    let title = kb
                        .read(id)
                        .ok()
                        .and_then(|t| format::parse(&t).ok())
                        .and_then(|d| d.frontmatter.get_text("title").map(str::to_string))
                        .unwrap_or_default();
                    for l in super::delete::linked_from(kb, id, &title) {
                        let from = l["id"].as_str().unwrap_or("").to_string();
                        if !inside_ids.contains(&from) {
                            let mut l = l;
                            l["to"] = json!(id);
                            linked.push(l);
                        }
                    }
                }
                if !linked.is_empty() {
                    let n = linked
                        .iter()
                        .filter_map(|l| l["id"].as_str())
                        .collect::<BTreeSet<_>>()
                        .len();
                    return Err(Error::validation(format!(
                    "{n} document{} outside the folder still link{} into it. Nothing was changed.",
                    if n == 1 { "" } else { "s" },
                    if n == 1 { "s" } else { "" },
                ))
                .with_next(
                    "Remove or replace the links with update_document, then call remove_folder again.",
                )
                .with_data(json!({ "linked_from": linked })));
                }

                let mut changes = Vec::new();
                for id in docs.iter().chain(index_files.iter()) {
                    changes.push(FileChange::Remove { id: id.clone() });
                }
                for f in &inside {
                    if f.gitkeep {
                        changes.push(FileChange::RemoveFile {
                            path: format!("{}/{GITKEEP}", f.path),
                        });
                    }
                }
                let folders: Vec<String> = inside.iter().map(|f| f.path.clone()).collect();
                let shown: Vec<&String> = docs.iter().take(50).collect();
                let more = docs.len().saturating_sub(50);
                let mut summary = format!(
                    "Remove the folder \"{path}\" with {} document{}.\n\n",
                    docs.len(),
                    if docs.len() == 1 { "" } else { "s" }
                );
                for d in shown {
                    summary.push_str(&format!("- {d}\n"));
                }
                if more > 0 {
                    summary.push_str(&format!("- and {more} more\n"));
                }
                let mut data = Map::new();
                data.insert("folder".into(), json!(path));
                data.insert("documents".into(), json!(docs));
                data.insert("folders".into(), json!(folders));
                Ok(Plan {
                first: None,
                revert: None,
                changes,
                verb: Verb::RemoveFolder,
                title: path.clone(),
                commit_body: Some(format!(
                    "Documents: {}",
                    docs.len()
                )),
                summary,
                message: format!(
                    "Removed the folder `{path}` with {} document{}. The files stay in the Git history.",
                    docs.len(),
                    if docs.len() == 1 { "" } else { "s" }
                ),
                data,
                text: None,
                notices: Vec::new(),
            })
            },
        )
    }
}
