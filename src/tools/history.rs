//! The `history` tool: the versions of one document (see `specs/features/document-history`).

use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::message::parse_subject;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::store::{Scope, Store};
use serde_json::{json, Value};

pub struct History;

impl Tool for History {
    fn name(&self) -> &'static str {
        "history"
    }

    fn description(&self) -> &'static str {
        "Lists the versions of one document, newest first. Returns entries (version, time, author, verb, subject) and next_before. Changes nothing."
    }

    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "id": {"type": "string", "description": "Document ID: its path without .md."},
                "limit": {"type": "integer", "description": "How many versions. Default 10, at most 50."},
                "before": {"type": "string", "description": "A version from an earlier result. Shows older versions."}
            },
            "required": ["id"],
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let id = args["id"].as_str().unwrap_or("").to_string();
        let limit = match args.get("limit").and_then(Value::as_i64) {
            None => 10,
            Some(n) if n >= 1 => (n as usize).min(50),
            Some(_) => {
                return Err(Error::invalid("\"limit\" must be 1 or more.")
                    .with_next("Send a limit of 1 to 50."))
            }
        };
        let before = crate::writes::get_str(&args, "before");

        let repo = ctx.repo()?;
        let memrepo = ctx.memrepo()?;
        // An invalid ID is `invalid_arguments`.
        memrepo.path_of(&id)?;
        let notices = repo.refresh_for_read();
        let store = Store::load(&memrepo);
        let not_found = || {
            Error::not_found(format!("There is no document with the ID \"{id}\"."))
                .with_next("Use search to find the ID.")
        };
        if !Scope::limit(&env.effective).allows(&id) || store.get(&id).is_none() {
            return Err(not_found());
        }
        let rel = memrepo.repo_relative(&id)?;
        let mut versions = repo.file_history(&rel)?;
        if let Some(b) = before {
            let Some(pos) = versions
                .iter()
                .position(|(e, _)| e.hash.starts_with(b) || b.starts_with(&e.short))
            else {
                return Err(
                    Error::invalid("\"before\" is not a version of this document.")
                        .with_next("Use a version from an earlier history result."),
                );
            };
            versions.drain(..=pos);
        }
        let more = versions.len() > limit;
        versions.truncate(limit);
        let next_before = if more {
            versions.last().map(|(e, _)| e.short.clone())
        } else {
            None
        };
        let entries: Vec<Value> = versions
            .iter()
            .map(|(e, _)| {
                json!({
                    "version": e.short,
                    "time": e.date,
                    "author": format!("{} <{}>", e.author_name, e.author_email),
                    "verb": parse_subject(&e.subject).map(|(v, _)| v.as_str()),
                    "subject": e.subject,
                })
            })
            .collect();
        let message = match entries.len() {
            0 => "No versions found.".to_string(),
            1 => "1 version found.".to_string(),
            n => format!("{n} versions found."),
        };
        let mut out = ToolOutput::new(
            message,
            json!({"id": id, "entries": entries, "next_before": next_before}),
        );
        for n in notices {
            out = out.notice(n);
        }
        Ok(out)
    }
}
