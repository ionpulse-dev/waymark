//! The `delete` tool: remove a document and fix every link to it.

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
        "Deletes a document and fixes every link to it in the same commit. The file stays in the Git history. Needs the version from show as base."
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
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(ctx, env, "delete", Kind::Destructive, preview, |tx, kb| {
            let (rel, doc) = writes::read_doc(tx, kb, &id)?;
            tx.check_base(&rel, Some(&base))?;
            let title = doc.frontmatter.get_text("title").unwrap_or("").to_string();
            let file_name = id.rsplit('/').next().unwrap_or(&id).to_string();
            let all = kb.documents();
            let same_name = all
                .iter()
                .filter(|d| d.id.rsplit('/').next() == Some(file_name.as_str()))
                .count();
            let target_id = id.clone();
            let matches = move |name: &str| {
                name == target_id || (!name.contains('/') && name == file_name && same_name == 1)
            };
            let target = Target {
                id: &id,
                title: &title,
                wiki_matches: &matches,
            };

            let mut changes = Vec::new();
            let mut links_fixed = Vec::new();
            for other in all.iter().filter(|d| d.id != id) {
                let Ok(text) = kb.read(&other.id) else {
                    continue;
                };
                let Ok(mut d) = format::parse(&text) else {
                    continue;
                };
                let in_props = linkfix::fix_properties(&mut d.frontmatter, &target);
                let (body, in_body) =
                    linkfix::fix_body(&format!("{}.md", other.id), &d.body, &target);
                if in_props + in_body == 0 {
                    continue;
                }
                if !env.effective.allows_folder(&other.id) {
                    return Err(Error::new(
                        Code::AccessDenied,
                        format!("The document \"{}\" links to this one and is outside the allowed folders.", other.id),
                    )
                    .with_next("Ask someone with wider access to delete this document."));
                }
                d.body = body;
                writes::touch(&mut d, &today, &author);
                if in_body > 0 {
                    links_fixed.push(json!({"id": other.id, "where": "body", "count": in_body}));
                }
                if in_props > 0 {
                    links_fixed
                        .push(json!({"id": other.id, "where": "property", "count": in_props}));
                }
                changes.push(FileChange::Write {
                    id: other.id.clone(),
                    text: d.render(),
                    action: "update",
                });
            }
            changes.insert(0, FileChange::Remove { id: id.clone() });

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
            data.insert("links_fixed".into(), json!(links_fixed));
            Ok(Plan {
                first: None,
                revert: None,
                changes,
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
