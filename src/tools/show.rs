//! The `show` tool: one document with its properties, body, version, backlinks, and replacement.

use crate::config::Kind;
use crate::error::{Code, Error};
use crate::format;
use crate::kb::KnowledgeBase;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::store::backlinks::backlinks;
use crate::store::{Scope, Store};
use serde_json::{json, Value};

pub struct Show;

fn not_found(id: &str) -> Error {
    Error::not_found(format!("There is no document with the ID \"{id}\"."))
        .with_next("Use search or list to find the ID.")
}

/// The ID of a path below the repository folder, if it lies below the root.
fn id_of_rel(kb: &KnowledgeBase, rel: &str) -> Option<String> {
    let prefix = kb.root.strip_prefix(&kb.repo_dir).ok()?;
    let prefix = prefix.to_string_lossy();
    let below = if prefix.is_empty() {
        rel
    } else {
        rel.strip_prefix(&format!("{prefix}/"))?
    };
    below.strip_suffix(".md").map(str::to_string)
}

fn age_days(updated: &str) -> Value {
    chrono::NaiveDate::parse_from_str(updated, "%Y-%m-%d")
        .map(|d| json!((chrono::Local::now().date_naive() - d).num_days().max(0)))
        .unwrap_or(Value::Null)
}

impl Tool for Show {
    fn name(&self) -> &'static str {
        "show"
    }

    fn description(&self) -> &'static str {
        "Returns one document: its properties, body, version, backlinks, and the documents that replace it. The version is the base for a change. Changes nothing."
    }

    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {"id": {"type": "string", "description": "The path of the document below the root, without .md."}},
            "required": ["id"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let asked = args["id"].as_str().unwrap_or("").to_string();
        let repo = ctx.repo()?;
        let kb = ctx.kb()?;
        // An invalid ID is `invalid_arguments`.
        kb.path_of(&asked)?;
        let mut notices = repo.refresh_if_stale();
        let store = Store::load(&kb);
        let limit_scope = Scope::limit(&env.effective);
        if !limit_scope.allows(&asked) {
            return Err(not_found(&asked));
        }

        let mut id = asked.clone();
        let mut moved_from = None;
        if store.get(&id).is_none() {
            let rel = kb.repo_relative(&id)?;
            let new_rel = repo.renamed_to(&rel)?;
            let new_id = new_rel
                .and_then(|r| id_of_rel(&kb, &r))
                .filter(|n| store.get(n).is_some() && limit_scope.allows(n));
            match new_id {
                Some(n) => {
                    notices.push(format!("This document moved from `{id}` to `{n}`."));
                    moved_from = Some(id.clone());
                    id = n;
                }
                None => return Err(not_found(&asked)),
            }
        }
        let doc = store.get(&id).ok_or_else(|| not_found(&asked))?;
        if !doc.parsed() {
            let first = format::check_bytes(
                &format!("{id}.md"),
                doc.raw.as_bytes(),
                &kb.settings.documents,
                None,
            )
            .into_iter()
            .find(|f| f.is_error())
            .map(|f| f.message)
            .unwrap_or_else(|| "The document cannot be read.".to_string());
            return Err(Error::new(
                Code::ValidationFailed,
                format!("The document \"{id}\" cannot be read: {first}"),
            )
            .with_next("Fix the file by hand, or run lint."));
        }

        // The read budget: a new document costs one.
        {
            let mut opened = ctx.opened.lock().unwrap();
            if !opened.contains(&id) {
                if opened.len() >= env.effective.max_documents_opened {
                    return Err(Error::new(
                        Code::AccessDenied,
                        format!(
                            "The read budget of this session is used up: {} documents are open.",
                            env.effective.max_documents_opened
                        ),
                    )
                    .with_next("Work with the documents that you have read. A new session has a new budget."));
                }
                opened.insert(id.clone());
            }
        }

        let rel = kb.repo_relative(&id)?;
        let version = repo.last_commit(&rel)?;
        let last_editor = repo
            .log(1, None, &[rel.as_str()])?
            .first()
            .map(|e| format!("{} <{}>", e.author_name, e.author_email));
        let pending = repo.dirty_paths()?.contains(&rel);
        let all_backlinks = backlinks(&store, &id, &|other| limit_scope.allows(other));
        let replaced_by: Vec<String> = {
            let mut v: Vec<String> = store
                .docs()
                .filter(|d| limit_scope.allows(&d.id) && d.property_ids("supersedes").contains(&id))
                .map(|d| d.id.clone())
                .collect();
            v.sort();
            v
        };

        let mut data = json!({
            "id": id,
            "title": doc.title(),
            "description": doc.description(),
            "status": doc.status(),
            "created": doc.text("created").unwrap_or(""),
            "updated": doc.updated(),
            "age_days": age_days(doc.updated()),
            "created_by": doc.text("created_by").unwrap_or(""),
            "updated_by": doc.text("updated_by").unwrap_or(""),
            "last_editor": last_editor,
            "disputed": doc.disputed(),
            "tags": doc.tags(),
            "sources": doc.list("sources"),
            "related": doc.property_ids("related"),
            "supersedes": doc.property_ids("supersedes"),
            "anchors": doc.list("anchors"),
            "body": doc.body.trim_matches('\n'),
            "version": version,
            "backlinks": all_backlinks.iter().take(50).collect::<Vec<_>>(),
            "backlinks_total": all_backlinks.len(),
            "replaced_by": replaced_by,
            "pending_edit": pending,
            "moved_from": moved_from,
        });
        if doc.disputed() {
            data["dispute_reason"] = json!(doc.text("dispute_reason").unwrap_or(""));
        }
        for key in ["trust", "verified_by", "verified"] {
            if let Some(v) = doc.text(key) {
                data[key] = json!(v);
            }
        }
        let mut out = ToolOutput::new("Document found.", data);
        for n in notices {
            out = out.notice(n);
        }
        Ok(out)
    }
}
