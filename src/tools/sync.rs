//! The `sync` tool: publish manual edits, pull, and push.

use crate::config::Kind;
use crate::error::{Code, Error};
use crate::format::Severity;
use crate::manual::{self, Outcome};
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::writes;
use serde_json::{json, Value};
use std::cell::RefCell;

pub struct Sync;

fn finding_json(f: &crate::format::Finding) -> Value {
    json!({
        "path": f.path,
        "rule": f.rule,
        "severity": if f.severity == Severity::Error { "error" } else { "warning" },
        "message": f.message,
    })
}

fn blocked_json(o: &Outcome) -> Vec<Value> {
    o.blocked
        .iter()
        .map(|b| {
            json!({
                "id": b.id,
                "path": b.path,
                "findings": b.findings.iter().map(finding_json).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn skipped_json(o: &Outcome) -> Vec<Value> {
    o.skipped
        .iter()
        .map(|s| json!({"path": s.path, "reason": s.reason}))
        .collect()
}

/// The ID of a path below the repository folder.
fn rel_to_id(kb: &crate::kb::KnowledgeBase, rel: &str) -> String {
    let prefix = kb
        .root
        .strip_prefix(&kb.repo_dir)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let below = if prefix.is_empty() {
        rel
    } else {
        rel.strip_prefix(&format!("{prefix}/")).unwrap_or(rel)
    };
    below.trim_end_matches(".md").to_string()
}

fn summary(committed: usize, blocked: usize) -> String {
    let edits = if committed == 1 { "edit" } else { "edits" };
    let mut m = match committed {
        0 => "No manual edits to publish.".to_string(),
        n => format!("{n} manual {edits} published."),
    };
    if blocked > 0 {
        let docs = if blocked == 1 {
            "document"
        } else {
            "documents"
        };
        m.push_str(&format!(
            " {blocked} {docs} failed the checks and stays unpublished."
        ));
    }
    m
}

impl Tool for Sync {
    fn name(&self) -> &'static str {
        "sync"
    }

    fn description(&self) -> &'static str {
        "Publishes documents that people edited by hand: checks them, commits the ones that pass, pulls, and pushes. With read-only access it only pulls."
    }

    // `sync` stays listed when writes are paused or read-only. It checks the limits itself.
    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {"preview": {"type": "boolean"}},
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let preview = writes::get_bool(&args, "preview");
        let repo = ctx.repo()?;
        let kb = ctx.kb()?;
        let author = writes::author_string(ctx)?;
        let today = writes::today();
        let can_write = env.effective.check_tool("sync", Kind::Write).is_ok();

        // Read-only or paused: only pull.
        if !can_write && !preview {
            let (pulled, mut notices) = repo.pull_only()?;
            notices.push("Writes are not allowed, so nothing was committed or pushed.".to_string());
            let mut out = ToolOutput::new(
                "Pulled. Nothing was committed or pushed.",
                json!({"committed": [], "blocked": [], "skipped": [], "pulled": pulled, "pushed": false, "conflicts": []}),
            );
            for n in notices {
                out = out.notice(n);
            }
            return Ok(out);
        }

        // Preview: check, and change nothing. No pull, no push.
        if preview {
            let tx = crate::gitsync::Tx::new_for_sync(&repo);
            let o = manual::prepare(&tx, &kb, &env.effective, &author, &today, false)?;
            let committed: Vec<Value> = o
                .passing
                .iter()
                .map(|e| json!({"id": e.id, "kind": e.kind.name()}))
                .collect();
            return Ok(ToolOutput::new(
                summary(committed.len(), o.blocked.len()),
                json!({
                    "committed": committed,
                    "blocked": blocked_json(&o),
                    "skipped": skipped_json(&o),
                    "pulled": 0,
                    "pushed": false,
                    "conflicts": [],
                    "preview": true,
                }),
            ));
        }

        let outcome: RefCell<Option<Outcome>> = RefCell::new(None);
        let result = repo.sync(|tx| {
            let o = manual::prepare(tx, &kb, &env.effective, &author, &today, true)?;
            if o.change.is_none() && !o.blocked.is_empty() {
                let first = &o.blocked[0];
                let detail = first
                    .findings
                    .iter()
                    .find(|f| f.is_error())
                    .map(|f| f.message.clone())
                    .unwrap_or_default();
                return Err(Error::new(
                    Code::ValidationFailed,
                    format!("{} is not published: {detail}", first.id),
                )
                .with_next("Fix the document, then run sync again."));
            }
            if o.change.is_some() {
                writes::approve(env, "sync", Kind::Write, "Publish manual edits.")?;
            }
            let change = o.change.clone();
            *outcome.borrow_mut() = Some(o);
            Ok(change)
        })?;
        let o = outcome.into_inner().expect("the hook ran");
        let commit = result.commits.first().cloned();
        let committed: Vec<Value> = o
            .passing
            .iter()
            .map(|e| json!({"id": e.id, "kind": e.kind.name(), "commit": commit}))
            .collect();
        let conflicts: Vec<Value> = result
            .conflicts
            .iter()
            .map(|(orig, copy)| json!({"id": rel_to_id(&kb, orig), "copy": rel_to_id(&kb, copy)}))
            .collect();
        let mut out = ToolOutput::new(
            summary(committed.len(), o.blocked.len()),
            json!({
                "committed": committed,
                "blocked": blocked_json(&o),
                "skipped": skipped_json(&o),
                "pulled": result.pulled,
                "pushed": result.pushed,
                "conflicts": conflicts,
            }),
        );
        for n in result.notices {
            out = out.notice(n);
        }
        for b in &o.blocked {
            if let Some(f) = b.findings.iter().find(|f| f.is_error()) {
                out = out.notice(format!("{} is not published: {}", b.id, f.message));
            }
        }
        Ok(out)
    }
}
