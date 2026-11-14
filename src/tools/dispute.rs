//! The `dispute` tool: mark a document as disputed, or end the dispute.

use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes::{self, FileChange, Plan};
use serde_json::{json, Map, Value};

pub struct Dispute;

impl Tool for Dispute {
    fn name(&self) -> &'static str {
        "dispute"
    }

    fn description(&self) -> &'static str {
        "Marks a document as disputed with a reason. With clear true it ends the dispute. Needs the version from show as base."
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
                "reason": {"type": "string", "description": "One line. Required unless clear is true."},
                "clear": {"type": "boolean"},
                "preview": {"type": "boolean"}
            },
            "required": ["id", "base"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let id = writes::get_str(&args, "id").unwrap_or("").to_string();
        let base = writes::get_str(&args, "base").unwrap_or("").to_string();
        let reason = writes::get_str(&args, "reason").map(|r| r.trim().to_string());
        let clear = writes::get_bool(&args, "clear");
        let preview = writes::get_bool(&args, "preview");
        match (&reason, clear) {
            (Some(_), true) => {
                return Err(Error::invalid("Send reason or clear, not both.")
                    .with_next("Send reason to dispute, or clear: true to end the dispute."))
            }
            (None, false) => {
                return Err(Error::invalid("A reason is required.")
                    .with_next("Send reason, or clear: true to end a dispute."))
            }
            _ => {}
        }
        if let Some(r) = &reason {
            writes::check_reason(r, ctx.kb()?.settings.documents.description_max_words)?;
        }
        writes::require_visible(env, &id)?;
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(ctx, env, "dispute", Kind::Write, preview, |tx, kb| {
            let (rel, mut doc) = writes::read_doc(tx, kb, &id)?;
            tx.check_base(&rel, Some(&base))?;
            let title = doc.frontmatter.get_text("title").unwrap_or("").to_string();
            let disputed = doc.frontmatter.get_bool("disputed") == Some(true);
            let message;
            if let Some(r) = &reason {
                if disputed {
                    let old = doc.frontmatter.get_text("dispute_reason").unwrap_or("");
                    return Err(Error::validation(format!(
                        "This document is already disputed: {old}."
                    ))
                    .with_next(
                        "Fix it with update_document, or end the dispute with clear: true.",
                    ));
                }
                let mut problems = writes::Problems::default();
                let custom = crate::lint::secrets::compile(&kb.settings.extra_secret_patterns)?;
                for hit in crate::lint::secrets::scan(r, &custom) {
                    problems.add(
                        format!(
                            "The reason looks like it contains a secret (rule {}, line {}).",
                            hit.rule, hit.line
                        ),
                        "Remove the secret, or replace it with a placeholder.",
                    );
                }
                if !problems.is_empty() {
                    return Err(problems.into_error());
                }
                doc.frontmatter.set_bool("disputed", true);
                doc.frontmatter.set_text("dispute_reason", r);
                message = format!("Disputed \"{title}\" (`{id}`).");
            } else {
                if !disputed {
                    return Err(Error::validation("This document is not disputed.")
                        .with_next("Nothing to clear."));
                }
                doc.frontmatter.remove("disputed");
                doc.frontmatter.remove("dispute_reason");
                message = format!("Ended the dispute of \"{title}\" (`{id}`).");
            }
            writes::touch(&mut doc, &today, &author);
            let text = doc.render();
            let mut problems = writes::Problems::default();
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
            data.insert("disputed".into(), json!(reason.is_some()));
            Ok(Plan {
                first: None,
                changes: vec![FileChange::Write {
                    id: id.clone(),
                    text: text.clone(),
                    action: "update",
                }],
                verb: Verb::Dispute,
                title: title.clone(),
                commit_body: None,
                summary: message.clone(),
                message,
                data,
                text: Some(text),
                notices,
            })
        })
    }
}
