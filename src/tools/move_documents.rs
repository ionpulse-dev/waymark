//! The `move` tool: move or rename a document or a folder, and fix every link.

use crate::config::Kind;
use crate::error::{Code, Error};
use crate::format::{self, id};
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes::linkfix::{retarget_body, retarget_properties, Remap};
use crate::writes::{self, FileChange, FirstCommit, Plan};
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub struct Move;

/// All files below a folder, as paths below the root.
fn files_below(dir: &Path, root: &Path, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        if p.is_dir() {
            files_below(&p, root, out);
        } else if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_string_lossy().to_string());
        }
    }
}

impl Tool for Move {
    fn name(&self) -> &'static str {
        "move"
    }

    fn description(&self) -> &'static str {
        "Moves or renames a document, or a whole folder (a path with a trailing slash). Rewrites every link to it and keeps the Git history. Makes one commit for the move and one for the link fixes."
    }

    fn kind(&self) -> Kind {
        Kind::Write
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "from": {"type": "string", "description": "A document ID, or a folder path with a trailing slash."},
                "to": {"type": "string", "description": "The new ID, or the new folder path with a trailing slash."},
                "preview": {"type": "boolean"}
            },
            "required": ["from", "to"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let from = writes::get_str(&args, "from").unwrap_or("").to_string();
        let to = writes::get_str(&args, "to").unwrap_or("").to_string();
        let preview = writes::get_bool(&args, "preview");
        let is_folder = from.ends_with('/');
        if is_folder != to.ends_with('/') {
            return Err(Error::invalid("\"from\" and \"to\" must be the same kind.")
                .with_next("Use two document IDs, or two folder paths that end with a slash."));
        }
        if from.trim_matches('/') == to.trim_matches('/') {
            return Err(Error::invalid("\"from\" and \"to\" are equal.")
                .with_next("Send a new location in \"to\"."));
        }
        if to.ends_with(".md") || from.ends_with(".md") {
            return Err(Error::invalid("A document ID has no .md.")
                .with_next("Send the path without the .md ending."));
        }
        let from_key = from.trim_end_matches('/').to_string();
        let to_key = to.trim_end_matches('/').to_string();
        if from_key.is_empty() || to_key.is_empty() {
            return Err(Error::validation("The root folder cannot move.")
                .with_next("Move a folder below the root."));
        }
        writes::clean_folder(&from_key).map_err(|_| {
            Error::invalid(format!("\"{from}\" is not a valid location."))
                .with_next("Use a path below the root.")
        })?;
        writes::clean_folder(&to_key).map_err(|_| {
            Error::invalid(format!("\"{to}\" is not a valid location."))
                .with_next("Use a path below the root, without .. and hidden names.")
        })?;
        if is_folder && (to_key == from_key || to_key.starts_with(&format!("{from_key}/"))) {
            return Err(Error::validation("A folder cannot move into itself.")
                .with_next("Choose a location outside the folder."));
        }
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(ctx, env, "move", Kind::Write, preview, |tx, memrepo| {
            let all = memrepo.documents();
            let old_ids: HashSet<String> = all.iter().map(|d| d.id.clone()).collect();

            // What moves: (old ID, new ID) for documents.
            let mut pairs: Vec<(String, String)> = Vec::new();
            let mut file_moves: Vec<(String, String)> = Vec::new(); // below the root
            if is_folder {
                let prefix = format!("{from_key}/");
                for d in all.iter().filter(|d| d.id.starts_with(&prefix)) {
                    let new_id = format!("{to_key}/{}", &d.id[prefix.len()..]);
                    pairs.push((d.id.clone(), new_id));
                }
                let dir = memrepo.root.join(&from_key);
                let mut files = Vec::new();
                files_below(&dir, &memrepo.root, &mut files);
                for f in files {
                    let new_f = format!("{to_key}/{}", &f[from_key.len() + 1..]);
                    file_moves.push((f, new_f));
                }
                // An empty folder (only a `.gitkeep`) can move too.
                if pairs.is_empty() && file_moves.is_empty() {
                    return Err(Error::not_found(format!("There is no folder \"{from}\"."))
                        .with_next("Use folders to see the folders."));
                }
                if memrepo.root.join(&to_key).exists() {
                    return Err(
                        Error::validation(format!("The folder \"{to}\" exists already."))
                            .with_next(
                                "Choose a path that does not exist. A folder is never merged.",
                            ),
                    );
                }
            } else {
                if !old_ids.contains(&from_key) {
                    return Err(writes::not_found(&from_key));
                }
                if memrepo.root.join(format!("{to_key}.md")).exists() {
                    return Err(Error::validation(format!(
                        "The document \"{to_key}\" exists already."
                    ))
                    .with_next("Choose an ID that does not exist."));
                }
                pairs.push((from_key.clone(), to_key.clone()));
                file_moves.push((format!("{from_key}.md"), format!("{to_key}.md")));
            }

            // Names, limits, and edits that wait.
            let dirty = tx.repo().dirty_paths()?;
            for (old, new) in &pairs {
                let findings = id::check_path(&format!("{new}.md"));
                if let Some(f) = findings.first() {
                    return Err(Error::validation(format!(
                        "The new name \"{new}\" is not valid: {}",
                        f.message
                    ))
                    .with_next("Use lowercase letters, digits, and - in names."));
                }
                for check in [old, new] {
                    if !env.effective.allows_folder(check) {
                        return Err(Error::new(
                            Code::AccessDenied,
                            format!("\"{check}\" is outside the allowed folders."),
                        )
                        .with_next("Choose locations that the limits allow."));
                    }
                }
                if dirty.contains(&memrepo.repo_relative(old)?) {
                    return Err(Error::new(
                        Code::StaleBase,
                        format!("The document \"{old}\" has manual edits that are not published."),
                    )
                    .with_next(
                        "This document has manual edits that are not published. Run `sync`.",
                    ));
                }
            }
            let moved: HashMap<String, String> = pairs.iter().cloned().collect();
            let remap = Remap::new(moved.clone(), &old_ids);

            // Link fixes, computed on the text before the move and written at the new place.
            let mut changes = Vec::new();
            let mut links_updated = Vec::new();
            for d in &all {
                let old_id = &d.id;
                let new_id = remap.new_id(old_id).to_string();
                let Ok(text) = memrepo.read(old_id) else {
                    continue;
                };
                let Ok(mut doc) = format::parse(&text) else {
                    continue;
                };
                let in_props = retarget_properties(&mut doc.frontmatter, &remap);
                let (body, in_body) = retarget_body(old_id, &new_id, &doc.body, &remap);
                let count = in_props + in_body;
                if count == 0 {
                    continue;
                }
                if !env.effective.allows_folder(old_id) || !env.effective.allows_folder(&new_id) {
                    return Err(Error::new(
                        Code::AccessDenied,
                        format!("The document \"{old_id}\" links to a moved document and is outside the allowed folders."),
                    )
                    .with_next("Ask someone with wider access to do this move."));
                }
                doc.body = body;
                writes::touch(&mut doc, &today, &author);
                links_updated.push(json!({"id": new_id, "count": count}));
                changes.push(FileChange::Write {
                    id: new_id,
                    text: doc.render(),
                    action: "update",
                });
            }

            // The history check: each moved document must be followable.
            let mut history = Vec::new();
            for (old, new) in &pairs {
                if let Some(c) = tx.repo().last_commit(&memrepo.repo_relative(old)?)? {
                    history.push((memrepo.repo_relative(new)?, c));
                }
            }
            let rel = |p: &str| -> Result<String, Error> {
                let prefix = memrepo
                    .root
                    .strip_prefix(&memrepo.repo_dir)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                Ok(if prefix.is_empty() {
                    p.to_string()
                } else {
                    format!("{prefix}/{p}")
                })
            };
            let moves: Vec<(String, String)> = file_moves
                .iter()
                .map(|(a, b)| Ok((rel(a)?, rel(b)?)))
                .collect::<Result<_, Error>>()?;

            let title = if is_folder {
                format!("{to_key}/")
            } else {
                text_title(memrepo, &from_key)
            };
            let total = pairs.len();
            let shown: Vec<Value> = pairs
                .iter()
                .take(100)
                .map(|(a, b)| json!({"from": a, "to": b}))
                .collect();
            let mut data = Map::new();
            data.insert(
                "kind".into(),
                json!(if is_folder { "folder" } else { "document" }),
            );
            data.insert("from".into(), json!(from));
            data.insert("to".into(), json!(to));
            data.insert("moved".into(), json!(shown));
            data.insert("moved_total".into(), json!(total));
            let fixed = links_updated.len();
            data.insert("links_updated".into(), json!(links_updated));
            let notices =
                vec!["Links from outside this memory repository are not checked.".to_string()];
            let docs = if total == 1 { "document" } else { "documents" };
            Ok(Plan {
                revert: None,
                first: Some(FirstCommit {
                    moves,
                    title: title.clone(),
                    history,
                }),
                changes,
                verb: Verb::Update,
                title: title.clone(),
                commit_body: None,
                summary: format!("Move {total} {docs}, update links in {fixed} documents"),
                message: format!("Moved {total} {docs} to \"{to}\"."),
                data,
                text: None,
                notices,
            })
        })
    }
}

/// The title of a document, for the commit message. The ID if it has none.
fn text_title(memrepo: &crate::memrepo::MemRepo, id: &str) -> String {
    memrepo
        .read(id)
        .ok()
        .and_then(|t| format::parse(&t).ok())
        .and_then(|d| d.frontmatter.get_text("title").map(str::to_string))
        .unwrap_or_else(|| id.to_string())
}
