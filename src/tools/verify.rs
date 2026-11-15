//! The `verify` tool: mark a document as verified.

use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::Verb;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes::{self, FileChange, Plan};
use serde_json::{json, Map, Value};

pub struct Verify;

impl Tool for Verify {
    fn name(&self) -> &'static str {
        "verify_document"
    }

    fn description(&self) -> &'static str {
        "Marks an active document as verified by you, with today's date. Commits and pushes. Needs the version from show as base. Returns id, title, trust, verified_by, verified, and commit. Exists only when the memory repository uses verification."
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
                "id": {
                    "type": "string",
                    "description": "Document ID: its path without .md."
                },
                "base": {
                    "type": "string",
                    "description": "Version from show. Fails if the document changed."
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
        writes::require_visible(env, &id)?;
        let author = writes::author_string(ctx)?;
        let today = writes::today();

        writes::execute(
            ctx,
            env,
            "verify_document",
            Kind::Write,
            preview,
            |tx, memrepo| {
                let (rel, mut doc) = writes::read_doc(tx, memrepo, &id)?;
                tx.check_base(&rel, Some(&base))?;
                let fm = &mut doc.frontmatter;
                let title = fm.get_text("title").unwrap_or("").to_string();
                if fm.get_text("status") != Some("active") {
                    return Err(
                        Error::validation("Only an active document can be verified.")
                            .with_next("Nothing to verify."),
                    );
                }
                if fm.get_bool("disputed") == Some(true) {
                    return Err(Error::validation("This document is disputed.").with_next(
                    "Fix it with update_document, or end the dispute with dispute_document and clear: true.",
                ));
                }
                if fm.get_text("trust") == Some("verified") {
                    return Err(Error::validation("This document is already verified.")
                        .with_next("Nothing to do."));
                }
                fm.set_text("trust", "verified");
                fm.set_text("verified_by", &author);
                fm.set_date("verified", &today);
                writes::touch(&mut doc, &today, &author);
                let text = doc.render();
                let mut problems = writes::Problems::default();
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
                data.insert("trust".into(), json!("verified"));
                data.insert("verified_by".into(), json!(author));
                data.insert("verified".into(), json!(today));
                Ok(Plan {
                    first: None,
                    revert: None,
                    changes: vec![FileChange::Write {
                        id: id.clone(),
                        text,
                        action: "update",
                    }],
                    verb: Verb::Update,
                    title: title.clone(),
                    commit_body: Some("Trust: verified".to_string()),
                    summary: format!("Verify \"{title}\" ({id})"),
                    message: format!("Verified \"{title}\" (`{id}`)."),
                    data,
                    text: None,
                    notices,
                })
            },
        )
    }
}
