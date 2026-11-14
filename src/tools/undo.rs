//! The `undo` tool: revert one of your writes with a new commit.

use crate::activity;
use crate::config::Kind;
use crate::error::{Code, Error};
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes::{self, Plan};
use serde_json::{json, Map, Value};

pub struct Undo;

impl Tool for Undo {
    fn name(&self) -> &'static str {
        "undo"
    }

    fn description(&self) -> &'static str {
        "Reverts one of your writes with a new commit and pushes it. Give the commit from activity. History stays. Refuses if later changes conflict."
    }

    fn kind(&self) -> Kind {
        Kind::Write
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "commit": {"type": "string", "description": "A commit from activity."},
                "preview": {"type": "boolean"}
            },
            "required": ["commit"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let commit = writes::get_str(&args, "commit")
            .unwrap_or("")
            .to_lowercase();
        let preview = writes::get_bool(&args, "preview");
        if !(7..=40).contains(&commit.len()) || !commit.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::invalid("\"commit\" is not a commit hash.")
                .with_next("Send a commit from activity: 7 to 40 hex characters."));
        }

        writes::execute(ctx, env, "undo", Kind::Write, preview, |tx, memrepo| {
            let repo = tx.repo();
            let commits = repo.all_commits()?;
            let entries = activity::entries(memrepo, &commits, &repo.author().email);
            let entry = entries
                .iter()
                .find(|e| {
                    e.hashes
                        .iter()
                        .any(|h| h.starts_with(&commit) || commit.starts_with(h.as_str()))
                })
                .ok_or_else(|| {
                    Error::not_found("No write with this commit is in your activity.")
                        .with_next("Run activity to see your writes.")
                })?;
            if entry.verb == Verb::Init {
                return Err(
                    Error::validation("A write of the tool `init` cannot be undone.")
                        .with_next("Delete the repository or reset it on the Git host."),
                );
            }
            if entry.undone {
                return Err(Error::validation(format!(
                    "This write was already undone by `{}`.",
                    entry.undone_by.clone().unwrap_or_default()
                ))
                .with_next("Nothing to do. To bring it back, undo the undo commit."));
            }
            for id in &entry.ids {
                if !env.effective.allows_folder(id) {
                    return Err(Error::new(
                        Code::AccessDenied,
                        format!("The document \"{id}\" is outside the allowed folders."),
                    )
                    .with_next("Ask someone with wider access."));
                }
            }
            let dirty = repo.dirty_paths()?;
            if dirty.iter().any(|p| entry.paths.contains(p)) {
                return Err(Error::new(
                    Code::StaleBase,
                    "A document of this write has manual edits that are not published.",
                )
                .with_next("Run `sync` first."));
            }

            // Try the revert. Then take it back: `execute` applies it after the approval.
            let hashes: Vec<String> = entry.hashes.clone();
            let files = match repo.revert_no_commit(&hashes)? {
                Ok(f) => f,
                Err(_) => {
                    let later: Vec<String> = commits[..entry.index]
                        .iter()
                        .filter(|c| c.files.iter().any(|(_, p)| entry.paths.contains(p)))
                        .filter(|c| !entry.hashes.iter().any(|h| c.hash.starts_with(h.as_str())))
                        .take(5)
                        .map(|c| format!("{} by {}: {}", c.short, c.name, c.subject))
                        .collect();
                    return Err(Error::new(
                        Code::StaleBase,
                        format!("Later changes block a clean revert: {}.", later.join("; ")),
                    )
                    .with_next("Change the document with `update_document` instead."));
                }
            };
            repo.abort_revert();
            let changes: Vec<Value> = files
                .iter()
                .filter_map(|(status, path)| {
                    let id = activity::id_of(memrepo, path)?;
                    let action = match status {
                        'A' => "restored",
                        'D' => "removed",
                        _ => "reverted",
                    };
                    Some(json!({"id": id, "action": action}))
                })
                .collect();

            let mut data = Map::new();
            data.insert("reverted".into(), json!(hashes));
            data.insert("title".into(), json!(entry.title));
            data.insert("changes".into(), json!(changes));
            Ok(Plan {
                first: None,
                revert: Some(hashes.clone()),
                changes: Vec::new(),
                verb: Verb::Undo,
                title: entry.title.clone(),
                commit_body: Some(format!("Reverts: {}", hashes.join(" "))),
                summary: format!("Undo \"{}\" ({})", entry.title, hashes.join(" ")),
                message: format!("Undid \"{}\".", entry.title),
                data,
                text: None,
                notices: Vec::new(),
            })
        })
    }
}
