//! The `restore_document` tool: bring a deprecated document back (see `specs/features/restore-and-replace`).

use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::store::Store;
use crate::writes::{self, FileChange, Plan};
use serde_json::{json, Map, Value};

pub struct Restore;

impl Tool for Restore {
    fn name(&self) -> &'static str {
        "restore_document"
    }

    fn description(&self) -> &'static str {
        "Brings a deprecated document back: sets its status to active. Needs the version from show as base. Commits and pushes. Returns id, title, status, commit, and version."
    }

    fn kind(&self) -> Kind {
        Kind::Write
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "The document ID: its path below the root, without .md."},
                "base": {"type": "string", "description": "The version from show. The call fails if the document changed since."},
                "reason": {"type": "string", "description": "Why the document is back. One line. Optional."},
                "preview": {"type": "boolean", "description": "true shows what the call would do and writes nothing."}
            },
            "required": ["id", "base"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let id = writes::get_str(&args, "id").unwrap_or("").to_string();
        let base = writes::get_str(&args, "base").unwrap_or("").to_string();
        let reason = writes::get_str(&args, "reason").map(|r| r.trim().to_string());
        let preview = writes::get_bool(&args, "preview");
        if let Some(r) = &reason {
            writes::check_reason(r, ctx.memrepo()?.settings.documents.description_max_words)?;
        }
        writes::require_visible(env, &id)?;
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(
            ctx,
            env,
            "restore_document",
            Kind::Write,
            preview,
            |tx, memrepo| {
                let (rel, mut doc) = writes::read_doc(tx, memrepo, &id)?;
                tx.check_base(&rel, Some(&base))?;
                let title = doc.frontmatter.get_text("title").unwrap_or("").to_string();
                if doc.frontmatter.get_text("status") != Some("deprecated") {
                    return Err(Error::validation("This document is already active.")
                        .with_next("Nothing to restore."));
                }
                let custom =
                    crate::lint::secrets::compile(&memrepo.settings.extra_secret_patterns)?;
                let mut problems = writes::Problems::default();
                if let Some(r) = &reason {
                    for hit in crate::lint::secrets::scan(r, &custom) {
                        problems.add(
                            format!(
                                "The reason looks like it contains a secret (rule {}, line {}).",
                                hit.rule, hit.line
                            ),
                            "Remove the secret, or replace it with a placeholder.",
                        );
                    }
                }
                doc.frontmatter.set_text("status", "active");
                // A document that was rejected needs a new review.
                if memrepo.settings.verification {
                    let fm = &mut doc.frontmatter;
                    fm.set_text("trust", "unverified");
                    fm.remove("verified_by");
                    fm.remove("verified");
                }
                writes::touch(&mut doc, &today, &author);
                let text = doc.render();
                let mut notices = Vec::new();
                writes::check_document(
                    memrepo,
                    &id,
                    &text,
                    &memrepo.id_set(),
                    &[],
                    None,
                    &mut problems,
                    &mut notices,
                )?;
                if !problems.is_empty() {
                    return Err(problems.into_error());
                }
                // An active document that replaces this one: the standard says the replaced
                // document should be deprecated. The person decides, so this is a notice.
                let store = Store::load(memrepo);
                let mut replacing: Vec<&str> = store
                    .docs()
                    .filter(|d| {
                        d.status() == "active"
                            && env.effective.allows_folder(&d.id)
                            && d.property_ids("supersedes").contains(&id)
                    })
                    .map(|d| d.id.as_str())
                    .collect();
                replacing.sort_unstable();
                if !replacing.is_empty() {
                    notices.push(format!(
                        "{} replaces this document. Deprecate one of them if only one should stay.",
                        replacing
                            .iter()
                            .map(|r| format!("`{r}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                let mut data = Map::new();
                data.insert("id".into(), json!(id));
                data.insert("title".into(), json!(title));
                data.insert("status".into(), json!("active"));
                Ok(Plan {
                    first: None,
                    revert: None,
                    changes: vec![FileChange::Write {
                        id: id.clone(),
                        text,
                        action: "restore",
                    }],
                    verb: Verb::Restore,
                    title: title.clone(),
                    commit_body: reason.as_ref().map(|r| format!("Reason: {r}")),
                    summary: match &reason {
                        Some(r) => format!("Restore \"{title}\" ({id}): {r}"),
                        None => format!("Restore \"{title}\" ({id})"),
                    },
                    message: format!("Restored \"{title}\" (`{id}`). It is active again."),
                    data,
                    text: None,
                    notices,
                })
            },
        )
    }
}
