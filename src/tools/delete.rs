//! The `delete` tool: remove a document. It refuses while another document links to it
//! (see `specs/features/write-documents`).

use crate::config::Kind;
use crate::error::{Code, Error};
use crate::format;
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes::linkfix::{self, Target};
use crate::writes::{self, FileChange, Plan};
use serde_json::{json, Map, Value};

pub struct Delete;

impl Tool for Delete {
    fn name(&self) -> &'static str {
        "delete"
    }

    fn description(&self) -> &'static str {
        "Deletes a document. Refuses while another document links to it, and lists the linking documents: remove those links first. The file stays in the Git history. Needs the version from show as base."
    }

    fn kind(&self) -> Kind {
        Kind::Destructive
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "base": {"type": "string", "description": "The version from show."},
                "preview": {"type": "boolean"}
            },
            "required": ["id", "base"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let id = writes::get_str(&args, "id").unwrap_or("").to_string();
        let base = writes::get_str(&args, "base").unwrap_or("").to_string();
        let preview = writes::get_bool(&args, "preview");
        writes::require_visible(env, &id)?;
        writes::author_string(ctx)?;

        writes::execute(ctx, env, "delete", Kind::Destructive, preview, |tx, kb| {
            let (rel, doc) = writes::read_doc(tx, kb, &id)?;
            tx.check_base(&rel, Some(&base))?;
            let title = doc.frontmatter.get_text("title").unwrap_or("").to_string();
            let linked = linked_from(kb, &id, &title);
            if !linked.is_empty() {
                if let Some(hidden) = linked
                    .iter()
                    .find(|l| !env.effective.allows_folder(l["id"].as_str().unwrap_or("")))
                {
                    return Err(Error::new(
                        Code::AccessDenied,
                        format!(
                            "The document \"{}\" links to this one and is outside the allowed folders.",
                            hidden["id"].as_str().unwrap_or("")
                        ),
                    )
                    .with_next("Ask someone with wider access to remove that link."));
                }
                let n = linked
                    .iter()
                    .filter_map(|l| l["id"].as_str())
                    .collect::<std::collections::BTreeSet<_>>()
                    .len();
                return Err(Error::validation(format!(
                    "{n} document{} still link{} to `{id}`. Nothing was changed.",
                    if n == 1 { "" } else { "s" },
                    if n == 1 { "s" } else { "" },
                ))
                .with_next(
                    "Remove or replace the links with update_document, then call delete again. Or use deprecate to keep the document.",
                )
                .with_data(json!({ "linked_from": linked })));
            }

            let mut notices = Vec::new();
            if doc
                .frontmatter
                .get_list("supersedes")
                .map(|l| !l.is_empty())
                .unwrap_or(false)
            {
                notices.push("The documents that this one replaced stay deprecated.".to_string());
            }
            let mut data = Map::new();
            data.insert("id".into(), json!(id));
            data.insert("title".into(), json!(title));
            Ok(Plan {
                first: None,
                revert: None,
                changes: vec![FileChange::Remove { id: id.clone() }],
                verb: Verb::Delete,
                title: title.clone(),
                commit_body: None,
                summary: format!("Delete \"{title}\" ({id})\n\n{}", doc.body),
                message: format!(
                    "Deleted \"{title}\" (`{id}`). The file stays in the Git history."
                ),
                data,
                text: None,
                notices,
            })
        })
    }
}

/// The documents that link to `id`, as `{ id, where, count }`: one entry for the body and one for
/// the properties of each linking document.
pub fn linked_from(kb: &crate::kb::KnowledgeBase, id: &str, title: &str) -> Vec<Value> {
    let file_name = id.rsplit('/').next().unwrap_or(id).to_string();
    let all = kb.documents();
    let same_name = all
        .iter()
        .filter(|d| d.id.rsplit('/').next() == Some(file_name.as_str()))
        .count();
    let target_id = id.to_string();
    let matches = move |name: &str| {
        name == target_id || (!name.contains('/') && name == file_name && same_name == 1)
    };
    let target = Target {
        id,
        title,
        wiki_matches: &matches,
    };
    let mut out = Vec::new();
    for other in all.iter().filter(|d| d.id != id) {
        let Ok(text) = kb.read(&other.id) else {
            continue;
        };
        let Ok(mut d) = format::parse(&text) else {
            continue;
        };
        // A generated index file lists every document. It never blocks a delete.
        let is_index = (other.id == "index" || other.id.ends_with("/index"))
            && d.frontmatter
                .get_list("tags")
                .map(|t| t.iter().any(|x| x == "index"))
                .unwrap_or(false);
        if is_index {
            continue;
        }
        let in_props = linkfix::fix_properties(&mut d.frontmatter, &target);
        let (_, in_body) = linkfix::fix_body(&format!("{}.md", other.id), &d.body, &target);
        if in_body > 0 {
            out.push(json!({"id": other.id, "where": "body", "count": in_body}));
        }
        if in_props > 0 {
            out.push(json!({"id": other.id, "where": "property", "count": in_props}));
        }
    }
    out
}
