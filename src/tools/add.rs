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
        "add_document"
    }

    fn description(&self) -> &'static str {
        "Creates a document. Sets the dates and the author, uses name or makes a file name from the title, checks the result, commits, and pushes. Returns id, title, deprecated, commit, and version."
    }

    fn kind(&self) -> Kind {
        Kind::Write
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folder": {
                    "type": "string",
                    "description": "Folder of the new document. Empty means the root."
                },
                "name": {
                    "type": "string",
                    "description": "File name without .md, such as readme: lowercase letters, digits, and -. Default: made from the title."
                },
                "title": {
                    "type": "string",
                    "description": "One sentence. At most 15 words by default."
                },
                "description": {
                    "type": "string",
                    "description": "What the document contains and when to read it. At most 50 words by default."
                },
                "body": {
                    "type": "string",
                    "description": "Markdown text. May be empty."
                },
                "sources": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Ticket IDs or URLs."
                },
                "tags": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Tags without # and without spaces."
                },
                "related": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "IDs of documents to read too."
                },
                "anchors": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Pointers to what the document describes, as git:<repo>/<path>@<commit> or url:<address>."
                },
                "supersedes": {
                    "type": "string",
                    "description": "ID of the document that this one replaces. That document becomes deprecated."
                },
                "preview": {
                    "type": "boolean",
                    "description": "true: dry run. Writes nothing."
                }
            },
            "required": [
                "folder",
                "title",
                "description",
                "body"
            ],
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
        let name = match writes::get_str(&args, "name") {
            Some(n) => Some(writes::slug::check_name(n)?),
            None => None,
        };
        let preview = writes::get_bool(&args, "preview");
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(
            ctx,
            env,
            "add_document",
            Kind::Write,
            preview,
            |tx, memrepo| {
                let ids = memrepo.id_set();
                let taken: std::collections::HashSet<String> =
                    ids.iter().map(|i| i.to_lowercase()).collect();
                let id = match &name {
                    Some(n) => {
                        let id = if folder.is_empty() {
                            n.clone()
                        } else {
                            format!("{folder}/{n}")
                        };
                        if taken.contains(&id.to_lowercase()) {
                            return Err(Error::validation(format!(
                                "A document with the name `{id}` exists."
                            ))
                            .with_next("Choose another name, or update that document."));
                        }
                        id
                    }
                    None => writes::slug::new_id(&folder, &title, &|i| {
                        taken.contains(&i.to_lowercase())
                    })
                    .ok_or_else(|| {
                        Error::validation("No free file name is left for this title.")
                            .with_next("Choose a different title.")
                    })?,
                };
                if !env.effective.allows_folder(&id) {
                    return Err(Error::new(
                        Code::AccessDenied,
                        "This folder is outside the allowed folders.",
                    )
                    .with_next("Choose a folder that the limits allow."));
                }
                if memrepo.is_excluded(&id) {
                    return Err(Error::validation(
                        "This folder is excluded from the memory repository.",
                    )
                    .with_next("Choose another folder."));
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
                if memrepo.settings.verification {
                    fm.set_text("trust", "unverified");
                }

                let mut changes = Vec::new();
                let mut deprecated = Vec::new();
                let mut skip: Vec<String> = Vec::new();
                if let Some(old_id) = &supersedes {
                    let s = writes::apply_supersedes(
                        tx, memrepo, env, &mut doc, None, old_id, &today, &author,
                    )?;
                    skip.push(old_id.clone());
                    if let Some(change) = s.deprecate {
                        changes.push(change);
                        deprecated.push(old_id.clone());
                    }
                }

                let text = doc.render();
                let mut ids_after = ids.clone();
                ids_after.insert(id.clone());
                let mut problems = writes::Problems::default();
                let mut notices = Vec::new();
                let mut scan_fields = vec![
                    ("title", title.clone()),
                    ("description", description.clone()),
                    ("body", body.clone()),
                ];
                scan_fields.push(("sources", sources.join("\n")));
                scan_fields.push(("tags", tags.join("\n")));
                let skip_refs: Vec<&str> = skip.iter().map(String::as_str).collect();
                writes::check_document_links(
                    memrepo,
                    &id,
                    &text,
                    &ids_after,
                    &scan_fields,
                    Some(TitleCheck { skip: &skip_refs }),
                    writes::NewLinks::All,
                    &mut problems,
                    &mut notices,
                )?;
                if !problems.is_empty() {
                    return Err(problems.into_error());
                }

                // Short wikilinks to a document with the same file name keep their target.
                let dirty = tx.repo().dirty_paths()?;
                let is_dirty = |i: &str| {
                    memrepo
                        .repo_relative(i)
                        .map(|r| dirty.contains(&r))
                        .unwrap_or(false)
                };
                let links_updated = writes::shortlinks::add_fixes_for_new_ids(
                    memrepo,
                    Some(env),
                    &ids,
                    std::slice::from_ref(&id),
                    &is_dirty,
                    &today,
                    &author,
                    &mut changes,
                )?;

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
                if !links_updated.is_empty() {
                    data.insert("links_updated".into(), json!(links_updated));
                }
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
            },
        )
    }
}
