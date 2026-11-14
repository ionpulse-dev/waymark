//! The `reject` tool: retire a document that failed review.

use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes::{self, FileChange, Plan};
use serde_json::{json, Map, Value};

pub struct Reject;

impl Tool for Reject {
    fn name(&self) -> &'static str {
        "reject"
    }

    fn description(&self) -> &'static str {
        "Rejects a document: sets its status to deprecated, clears its trust, and keeps the file. The reason goes into the commit message. Exists only when the knowledge base uses verification."
    }

    fn kind(&self) -> Kind {
        Kind::Write
    }

    fn needs_verification(&self) -> bool {
        true
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "base": {"type": "string", "description": "The version from show."},
                "reason": {"type": "string", "description": "One line."},
                "preview": {"type": "boolean"}
            },
            "required": ["id", "base", "reason"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let id = writes::get_str(&args, "id").unwrap_or("").to_string();
        let base = writes::get_str(&args, "base").unwrap_or("").to_string();
        let reason = writes::get_str(&args, "reason")
            .unwrap_or("")
            .trim()
            .to_string();
        let preview = writes::get_bool(&args, "preview");
        writes::check_reason(&reason, ctx.memrepo()?.settings.documents.description_max_words)?;
        writes::require_visible(env, &id)?;
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(ctx, env, "reject", Kind::Write, preview, |tx, memrepo| {
            let (rel, mut doc) = writes::read_doc(tx, memrepo, &id)?;
            tx.check_base(&rel, Some(&base))?;
            let title = doc.frontmatter.get_text("title").unwrap_or("").to_string();
            if doc.frontmatter.get_text("status") == Some("deprecated") {
                return Err(Error::validation("This document is already deprecated.")
                    .with_next("Nothing to do."));
            }
            let custom = crate::lint::secrets::compile(&memrepo.settings.extra_secret_patterns)?;
            let mut problems = writes::Problems::default();
            for hit in crate::lint::secrets::scan(&reason, &custom) {
                problems.add(
                    format!(
                        "The reason looks like it contains a secret ({}, line {}).",
                        hit.rule, hit.line
                    ),
                    "Remove the secret, or replace it with a placeholder.",
                );
            }
            let fm = &mut doc.frontmatter;
            fm.set_text("status", "deprecated");
            fm.set_text("trust", "unverified");
            fm.remove("verified_by");
            fm.remove("verified");
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
            let mut data = Map::new();
            data.insert("id".into(), json!(id));
            data.insert("title".into(), json!(title));
            data.insert("status".into(), json!("deprecated"));
            Ok(Plan {
                first: None,
                revert: None,
                changes: vec![FileChange::Write {
                    id: id.clone(),
                    text,
                    action: "deprecate",
                }],
                verb: Verb::Deprecate,
                title: title.clone(),
                commit_body: Some(format!("Reason: {reason}\nTrust: rejected")),
                summary: format!("Reject \"{title}\" ({id}): {reason}"),
                message: format!("Rejected \"{title}\" (`{id}`). The file stays."),
                data,
                text: None,
                notices,
            })
        })
    }
}
