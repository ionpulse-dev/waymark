//! The `update_document` tool: change fields or the body of a document.

use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes::{self, FileChange, Plan, TitleCheck};
use serde_json::{json, Map, Value};
use std::collections::HashSet;

pub struct UpdateDocument;

impl Tool for UpdateDocument {
    fn name(&self) -> &'static str {
        "update_document"
    }

    fn description(&self) -> &'static str {
        "Changes fields or the body of a document. Fields that are not sent stay as they are. Needs the version from show as base. Commits and pushes. Returns id, title, changed, commit, and version."
    }

    fn kind(&self) -> Kind {
        Kind::Write
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "The document ID: its path below the root, without .md."
                },
                "base": {
                    "type": "string",
                    "description": "The version from show. The call fails if the document changed since."
                },
                "title": {
                    "type": "string",
                    "description": "The new title."
                },
                "description": {
                    "type": "string",
                    "description": "The new description."
                },
                "body": {
                    "type": "string",
                    "description": "The new Markdown text. It replaces the whole body."
                },
                "sources": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "The new list of sources. It replaces the list."
                },
                "tags": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "The new list of tags. It replaces the list."
                },
                "related": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "The new list of related document IDs. It replaces the list."
                },
                "anchors": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "The new list of anchors, as git:<repo>/<path>@<commit> or url:<address>. It replaces the list."
                },
                "supersedes": {
                    "type": "string",
                    "description": "ID of a document that this one replaces. It is added to the list, and that document becomes deprecated."
                },
                "preview": {
                    "type": "boolean",
                    "description": "true shows what the call would do and writes nothing."
                }
            },
            "required": [
                "id",
                "base"
            ],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let id = writes::get_str(&args, "id").unwrap_or("").to_string();
        let base = writes::get_str(&args, "base").unwrap_or("").to_string();
        let preview = writes::get_bool(&args, "preview");
        const OPTIONAL: [&str; 8] = [
            "title",
            "description",
            "body",
            "sources",
            "tags",
            "related",
            "anchors",
            "supersedes",
        ];
        if !OPTIONAL.iter().any(|k| args.get(*k).is_some()) {
            return Err(Error::invalid("Nothing to change: no field was sent.")
                .with_next("Send at least one of title, description, body, sources, tags, related, anchors, or supersedes."));
        }
        writes::require_visible(env, &id)?;
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(
            ctx,
            env,
            "update_document",
            Kind::Write,
            preview,
            |tx, memrepo| {
                let (rel, mut doc) = writes::read_doc(tx, memrepo, &id)?;
                let old_text = doc.render();
                tx.check_base(&rel, Some(&base))?;
                let old_title = doc.frontmatter.get_text("title").unwrap_or("").to_string();
                let mut changed: Vec<&str> = Vec::new();
                let mut secret_fields: Vec<(&str, String)> = Vec::new();

                for key in ["title", "description"] {
                    if let Some(new) = writes::get_str(&args, key) {
                        let new = new.trim();
                        if doc.frontmatter.get_text(key) != Some(new) {
                            doc.frontmatter.set_text(key, new);
                            changed.push(key);
                        }
                        secret_fields.push((key, new.to_string()));
                    }
                }
                if let Some(new) = writes::get_str(&args, "body") {
                    let stored = writes::make_body(new);
                    if doc.body != stored {
                        doc.body = stored;
                        changed.push("body");
                    }
                    secret_fields.push(("body", new.to_string()));
                }
                for key in ["sources", "tags", "related", "anchors"] {
                    if let Some(mut list) = writes::get_list(&args, key) {
                        if key == "sources" || key == "tags" {
                            secret_fields.push((key, list.join("\n")));
                        }
                        if key == "related" {
                            list = list.iter().map(|r| writes::wikilink(r)).collect();
                        }
                        let current: Vec<String> = doc
                            .frontmatter
                            .get_list(key)
                            .map(<[String]>::to_vec)
                            .unwrap_or_default();
                        if current != list {
                            if list.is_empty() {
                                doc.frontmatter.remove(key);
                            } else {
                                doc.frontmatter.set_list(key, &list);
                            }
                            changed.push(key);
                        }
                    }
                }

                // The document that this one replaces becomes deprecated, as with `add_document`.
                let mut extra: Vec<FileChange> = Vec::new();
                let mut deprecated: Vec<String> = Vec::new();
                let replaced = writes::get_str(&args, "supersedes").map(str::trim);
                if let Some(old_id) = replaced {
                    let s = writes::apply_supersedes(
                        tx,
                        memrepo,
                        env,
                        &mut doc,
                        Some(&id),
                        old_id,
                        &today,
                        &author,
                    )?;
                    if s.linked {
                        changed.push("supersedes");
                    }
                    if let Some(change) = s.deprecate {
                        extra.push(change);
                        deprecated.push(old_id.to_string());
                    }
                }

                let mut data = Map::new();
                data.insert("id".into(), json!(id));
                data.insert("changed".into(), json!(changed));
                data.insert("deprecated".into(), json!(deprecated));
                if changed.is_empty() && extra.is_empty() {
                    data.insert("title".into(), json!(old_title));
                    data.insert("version".into(), json!(tx.last_commit(&rel)?));
                    return Ok(Plan {
                        first: None,
                        revert: None,
                        changes: vec![FileChange::Keep { id: id.clone() }],
                        verb: Verb::Update,
                        title: old_title,
                        commit_body: None,
                        summary: String::new(),
                        message: "No change.".into(),
                        data,
                        text: None,
                        notices: Vec::new(),
                    });
                }

                writes::touch(&mut doc, &today, &author);
                if memrepo.settings.verification
                    && doc.frontmatter.get_text("trust") == Some("verified")
                    && changed
                        .iter()
                        .any(|c| ["title", "description", "body"].contains(c))
                {
                    doc.frontmatter.set_text("trust", "unverified");
                    doc.frontmatter.remove("verified_by");
                    doc.frontmatter.remove("verified");
                }

                let text = doc.render();
                let new_title = doc.frontmatter.get_text("title").unwrap_or("").to_string();
                let ids_after: HashSet<String> = memrepo.id_set();
                let mut problems = writes::Problems::default();
                let mut notices = Vec::new();
                let skip: Vec<&str> = std::iter::once(id.as_str()).chain(replaced).collect();
                writes::check_document_links(
                    memrepo,
                    &id,
                    &text,
                    &ids_after,
                    &secret_fields,
                    if changed.contains(&"title") {
                        Some(TitleCheck { skip: &skip })
                    } else {
                        None
                    },
                    writes::NewLinks::Since(&old_text),
                    &mut problems,
                    &mut notices,
                )?;
                if !problems.is_empty() {
                    return Err(problems.into_error());
                }
                data.insert("title".into(), json!(new_title));
                Ok(Plan {
                    first: None,
                    revert: None,
                    changes: std::iter::once(FileChange::Write {
                        id: id.clone(),
                        text: text.clone(),
                        action: "update",
                    })
                    .chain(extra)
                    .collect(),
                    verb: Verb::Update,
                    title: new_title.clone(),
                    commit_body: None,
                    summary: format!("Update \"{new_title}\" ({id})\n\n{}", doc.body),
                    message: format!("Updated \"{new_title}\" (`{id}`): {}.", changed.join(", ")),
                    data,
                    text: Some(text),
                    notices,
                })
            },
        )
    }
}
