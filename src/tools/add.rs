//! The `add` tool: create a document (see `specs/features/write-documents`).

use crate::config::Kind;
use crate::error::{Code, Error};
use crate::format::Document;
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes::{self, FileChange, Plan, TitleCheck};
use serde_json::{json, Map, Value};

pub struct Add;

impl Tool for Add {
    fn name(&self) -> &'static str {
        "add"
    }

    fn description(&self) -> &'static str {
        "Creates a document. Sets the dates and the author, picks a file name, checks the result, commits, and pushes. Returns the ID and version."
    }

    fn kind(&self) -> Kind {
        Kind::Write
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folder": {"type": "string", "description": "Folder of the new document. Empty means the root."},
                "title": {"type": "string"},
                "description": {"type": "string"},
                "body": {"type": "string", "description": "Markdown text. May be empty."},
                "sources": {"type": "array", "items": {"type": "string"}},
                "tags": {"type": "array", "items": {"type": "string"}},
                "related": {"type": "array", "items": {"type": "string"}, "description": "Document IDs."},
                "anchors": {"type": "array", "items": {"type": "string"}},
                "supersedes": {"type": "string", "description": "ID of the document that this one replaces."},
                "preview": {"type": "boolean"}
            },
            "required": ["folder", "title", "description", "body"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let folder = writes::clean_folder(writes::get_str(&args, "folder").unwrap_or(""))?;
        let title = writes::get_str(&args, "title")
            .unwrap_or("")
            .trim()
            .to_string();
        let description = writes::get_str(&args, "description")
            .unwrap_or("")
            .trim()
            .to_string();
        let body = writes::get_str(&args, "body").unwrap_or("").to_string();
        let sources = writes::get_list(&args, "sources").unwrap_or_default();
        let tags = writes::get_list(&args, "tags").unwrap_or_default();
        let related = writes::get_list(&args, "related").unwrap_or_default();
        let anchors = writes::get_list(&args, "anchors").unwrap_or_default();
        let supersedes = writes::get_str(&args, "supersedes").map(str::to_string);
        let preview = writes::get_bool(&args, "preview");
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(ctx, env, "add", Kind::Write, preview, |tx, kb| {
            let ids = kb.id_set();
            let taken: std::collections::HashSet<String> =
                ids.iter().map(|i| i.to_lowercase()).collect();
            let id = writes::slug::new_id(&folder, &title, &|i| taken.contains(&i.to_lowercase()))
                .ok_or_else(|| {
                    Error::validation("No free file name is left for this title.")
                        .with_next("Choose a different title.")
                })?;
            if !env.effective.allows_folder(&id) {
                return Err(Error::new(
                    Code::AccessDenied,
                    "This folder is outside the allowed folders.",
                )
                .with_next("Choose a folder that the limits allow."));
            }
            if kb.is_excluded(&id) {
                return Err(
                    Error::validation("This folder is excluded from the knowledge base.")
                        .with_next("Choose another folder."),
                );
            }

            let mut doc = Document::new(&writes::make_body(&body));
            let fm = &mut doc.frontmatter;
            fm.set_text("title", &title);
            fm.set_text("description", &description);
            fm.set_text("status", "active");
            fm.set_date("created", &today);
            fm.set_date("updated", &today);
            fm.set_text("created_by", &author);
            fm.set_text("updated_by", &author);
            if !sources.is_empty() {
                fm.set_list("sources", &sources);
            }
            if !tags.is_empty() {
                fm.set_list("tags", &tags);
            }
            if !related.is_empty() {
                let links: Vec<String> = related.iter().map(|r| writes::wikilink(r)).collect();
                fm.set_list("related", &links);
            }
            if !anchors.is_empty() {
                fm.set_list("anchors", &anchors);
            }
            if kb.settings.verification {
                fm.set_text("trust", "unverified");
            }

            let mut changes = Vec::new();
            let mut deprecated = Vec::new();
            let mut skip: Vec<String> = Vec::new();
            if let Some(old_id) = &supersedes {
                writes::require_visible(env, old_id)?;
                let (_, mut old) = writes::read_doc(tx, kb, old_id)?;
                doc.frontmatter
                    .set_list("supersedes", &[writes::wikilink(old_id)]);
                skip.push(old_id.clone());
                if old.frontmatter.get_text("status") != Some("deprecated") {
                    old.frontmatter.set_text("status", "deprecated");
                    writes::touch(&mut old, &today, &author);
                    changes.push(FileChange::Write {
                        id: old_id.clone(),
                        text: old.render(),
                        action: "deprecate",
                    });
                    deprecated.push(old_id.clone());
                }
            }

            let text = doc.render();
            let mut ids_after = ids.clone();
            ids_after.insert(id.clone());
            let mut problems = writes::Problems::default();
            let mut notices = Vec::new();
            let mut secret_fields = vec![
                ("title", title.clone()),
                ("description", description.clone()),
                ("body", body.clone()),
            ];
            secret_fields.push(("sources", sources.join("\n")));
            secret_fields.push(("tags", tags.join("\n")));
            let skip_refs: Vec<&str> = skip.iter().map(String::as_str).collect();
            writes::check_document(
                kb,
                &id,
                &text,
                &ids_after,
                &secret_fields,
                Some(TitleCheck { skip: &skip_refs }),
                &mut problems,
                &mut notices,
            )?;
            if !problems.is_empty() {
                return Err(problems.into_error());
            }

            changes.insert(
                0,
                FileChange::Write {
                    id: id.clone(),
                    text: text.clone(),
                    action: "create",
                },
            );
            let mut data = Map::new();
            data.insert("id".into(), json!(id));
            data.insert("title".into(), json!(title));
            data.insert("deprecated".into(), json!(deprecated));
            Ok(Plan {
                first: None,
                revert: None,
                changes,
                verb: Verb::Add,
                title: title.clone(),
                commit_body: None,
                summary: format!("Add \"{title}\" in \"{folder}\"\n\n{body}"),
                message: format!("Added \"{title}\" as `{id}`."),
                data,
                text: Some(text),
                notices,
            })
        })
    }
}
