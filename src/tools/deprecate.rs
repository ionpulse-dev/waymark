//! The `deprecate` tool: retire a document and keep the file.

use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes::{self, FileChange, Plan};
use serde_json::{json, Map, Value};

pub struct Deprecate;

impl Tool for Deprecate {
    fn name(&self) -> &'static str {
        "deprecate"
    }

    fn description(&self) -> &'static str {
        "Retires a document: sets its status to deprecated and keeps the file. The reason goes into the commit message. Needs the version from show as base."
    }

    fn kind(&self) -> Kind {
        Kind::Write
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
        writes::check_reason(&reason, ctx.kb()?.settings.documents.description_max_words)?;
        writes::require_visible(env, &id)?;
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(ctx, env, "deprecate", Kind::Write, preview, |tx, kb| {
            let (rel, mut doc) = writes::read_doc(tx, kb, &id)?;
            tx.check_base(&rel, Some(&base))?;
            let title = doc.frontmatter.get_text("title").unwrap_or("").to_string();
            if doc.frontmatter.get_text("status") == Some("deprecated") {
                return Err(Error::validation("This document is already deprecated.")
                    .with_next("Nothing to do."));
            }
            let custom = crate::lint::secrets::compile(&kb.settings.extra_secret_patterns)?;
            let mut problems = writes::Problems::default();
            for hit in crate::lint::secrets::scan(&reason, &custom) {
                problems.add(
                    format!(
                        "The reason looks like it contains a secret (rule {}, line {}).",
                        hit.rule, hit.line
                    ),
                    "Remove the secret, or replace it with a placeholder.",
                );
            }
            doc.frontmatter.set_text("status", "deprecated");
            writes::touch(&mut doc, &today, &author);
            let text = doc.render();
            let mut notices = Vec::new();
            writes::check_document(
                kb,
                &id,
                &text,
                &kb.id_set(),
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
                changes: vec![FileChange::Write {
                    id: id.clone(),
                    text,
                    action: "deprecate",
                }],
                verb: Verb::Deprecate,
                title: title.clone(),
                commit_body: Some(format!("Reason: {reason}")),
                summary: format!("Deprecate \"{title}\" ({id}): {reason}"),
                message: format!("Deprecated \"{title}\" (`{id}`). The file stays."),
                data,
                text: None,
                notices,
            })
        })
    }
}
