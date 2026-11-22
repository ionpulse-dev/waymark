//! The `update_document` tool: change fields or the body of a document.

use crate::config::Kind;
use crate::error::{Code, Error};
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
                    "description": "Document ID: its path without .md."
                },
                "base": {
                    "type": "string",
                    "description": "Version from show. Fails if the document changed."
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
                    "description": "The new Markdown text. It replaces the whole body. For a small change use replace or append."
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
                "replace": {
                    "type": "object",
                    "properties": {
                        "old": {
                            "type": "string",
                            "description": "A passage of the current body. It must occur exactly once."
                        },
                        "new": {
                            "type": "string",
                            "description": "The text instead. May be empty."
                        }
                    },
                    "required": ["old", "new"],
                    "additionalProperties": false,
                    "description": "Changes one passage of the body. Do not send it with body or append."
                },
                "append": {
                    "type": "string",
                    "description": "Text added at the end of the body. Do not send it with body or replace."
                },
                "add_tags": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Tags to add. Do not send it with tags."
                },
                "remove_tags": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Tags to remove. Do not send it with tags."
                },
                "preview": {
                    "type": "boolean",
                    "description": "true: dry run. Writes nothing."
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
        const OPTIONAL: [&str; 12] = [
            "title",
            "description",
            "body",
            "sources",
            "tags",
            "related",
            "anchors",
            "supersedes",
            "replace",
            "append",
            "add_tags",
            "remove_tags",
        ];
        if !OPTIONAL.iter().any(|k| args.get(*k).is_some()) {
            return Err(Error::invalid("Nothing to change: no field was sent.")
                .with_next("Send at least one of title, description, body, replace, append, sources, tags, add_tags, remove_tags, related, anchors, or supersedes."));
        }
        // One way to change the body, and one way to change the tags (see `lean-tools`).
        let body_ways: Vec<&str> = ["body", "replace", "append"]
            .into_iter()
            .filter(|k| args.get(*k).is_some())
            .collect();
        if body_ways.len() > 1 {
            return Err(Error::invalid(format!(
                "These cannot be sent together: {}.",
                body_ways.join(", ")
            ))
            .with_next("Send only one of body, replace, or append."));
        }
        if args.get("tags").is_some()
            && (args.get("add_tags").is_some() || args.get("remove_tags").is_some())
        {
            return Err(
                Error::invalid("tags cannot be sent with add_tags or remove_tags.")
                    .with_next("Send tags as the new list, or send add_tags and remove_tags."),
            );
        }
        let add_tags = writes::get_list(&args, "add_tags").unwrap_or_default();
        let remove_tags = writes::get_list(&args, "remove_tags").unwrap_or_default();
        if let Some(t) = add_tags.iter().find(|t| remove_tags.contains(t)) {
            return Err(Error::invalid(format!(
                "The tag \"{t}\" is in add_tags and in remove_tags."
            ))
            .with_next("Send each tag in one of the two lists."));
        }
        for t in add_tags.iter().chain(&remove_tags) {
            writes::check_tag(t)?;
        }
        let replace = match args.get("replace") {
            Some(r) => {
                let old = r["old"].as_str().unwrap_or("");
                if old.is_empty() {
                    return Err(Error::invalid("replace.old is empty.")
                        .with_next("Send a passage of the body in replace.old."));
                }
                Some((
                    old.replace("\r\n", "\n").replace('\r', "\n"),
                    r["new"]
                        .as_str()
                        .unwrap_or("")
                        .replace("\r\n", "\n")
                        .replace('\r', "\n"),
                ))
            }
            None => None,
        };
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
                let mut scan_fields: Vec<(&str, String)> = Vec::new();

                for key in ["title", "description"] {
                    if let Some(new) = writes::get_str(&args, key) {
                        let new = new.trim();
                        if doc.frontmatter.get_text(key) != Some(new) {
                            doc.frontmatter.set_text(key, new);
                            changed.push(key);
                        }
                        scan_fields.push((key, new.to_string()));
                    }
                }
                if let Some(new) = writes::get_str(&args, "body") {
                    let stored = writes::make_body(new);
                    if doc.body != stored {
                        doc.body = stored;
                        changed.push("body");
                    }
                    scan_fields.push(("body", new.to_string()));
                }
                if let Some((old, new)) = &replace {
                    let n = doc.body.matches(old.as_str()).count();
                    if n != 1 {
                        let what = if n == 0 {
                            "The passage in replace.old does not occur in the body.".to_string()
                        } else {
                            format!("The passage in replace.old occurs {n} times in the body.")
                        };
                        let next = if n == 0 {
                            "Read the document again with show, then send a passage that occurs once."
                        } else {
                            "Add more text around the passage, so that it occurs once."
                        };
                        return Err(Error::new(Code::ValidationFailed, what).with_next(next));
                    }
                    let stored = writes::make_body(&doc.body.replacen(old.as_str(), new, 1));
                    if doc.body != stored {
                        doc.body = stored;
                        changed.push("body");
                    }
                    scan_fields.push(("body", new.clone()));
                }
                if let Some(text) = writes::get_str(&args, "append") {
                    let text = text.replace("\r\n", "\n").replace('\r', "\n");
                    let base = doc.body.trim_end_matches('\n');
                    let joined = if base.trim().is_empty() {
                        text.clone()
                    } else {
                        format!("{base}\n{text}")
                    };
                    let stored = writes::make_body(&joined);
                    if doc.body != stored {
                        doc.body = stored;
                        changed.push("body");
                    }
                    scan_fields.push(("body", text));
                }
                if !add_tags.is_empty() || !remove_tags.is_empty() {
                    let current: Vec<String> = doc
                        .frontmatter
                        .get_list("tags")
                        .map(<[String]>::to_vec)
                        .unwrap_or_default();
                    let mut list = current.clone();
                    for t in &add_tags {
                        if !list.contains(t) {
                            list.push(t.clone());
                        }
                    }
                    list.retain(|t| !remove_tags.contains(t));
                    scan_fields.push(("tags", add_tags.join("\n")));
                    if list != current {
                        if list.is_empty() {
                            doc.frontmatter.remove("tags");
                        } else {
                            doc.frontmatter.set_list("tags", &list);
                        }
                        changed.push("tags");
                    }
                }
                for key in ["sources", "tags", "related", "anchors"] {
                    if let Some(mut list) = writes::get_list(&args, key) {
                        if key == "sources" || key == "tags" {
                            scan_fields.push((key, list.join("\n")));
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
                    &scan_fields,
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
