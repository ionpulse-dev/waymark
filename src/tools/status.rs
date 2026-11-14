//! The `status` tool: the state of the setup, with health checks
//! (see `specs/features/maintenance` and `session-start`).

use crate::config::Kind;
use crate::error::Error;
use crate::identity::redact_text;
use crate::maintenance::{limit_sources, run_checks};
use crate::mcp::context::{Context, JoinState};
use crate::mcp::result::ToolOutput;
use crate::mcp::session;
use crate::mcp::tool::{Env, Tool};
use serde_json::{json, Value};

pub struct Status;

impl Tool for Status {
    fn name(&self) -> &'static str {
        "status"
    }

    fn description(&self) -> &'static str {
        "Shows the state of the setup: the knowledge base in use, the limits that apply, and a list of checks with a fix for each problem. Changes nothing."
    }

    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn input_schema(&self) -> Value {
        json!({ "type": "object", "properties": {}, "additionalProperties": false })
    }

    fn always_listed(&self) -> bool {
        true
    }

    fn call(&self, ctx: &Context, _env: &Env, _args: Value) -> Result<ToolOutput, Error> {
        let report = run_checks(ctx);
        let failed: Vec<&str> = report
            .checks
            .iter()
            .filter(|c| !c.ok)
            .map(|c| c.name)
            .collect();
        let healthy = failed.is_empty();

        // The facts of the knowledge base, when it is ready.
        let mut kb_json = json!({
            "state": report.state.name(),
            "name": null, "id": null, "slug": null, "path": null, "root": null,
        });
        let mut extra: Vec<String> = Vec::new();
        let mut problem = Value::Null;
        match &report.state {
            JoinState::Ready => {
                let f = session::facts(ctx, true);
                if let Some(k) = &f.kb {
                    kb_json = k.to_json();
                    kb_json["state"] = json!("ready");
                    if let Some(kb) = &report.kb {
                        kb_json["verification"] = json!(kb.settings.verification);
                        let root = kb.root.strip_prefix(&kb.repo_dir).unwrap_or(&kb.root);
                        kb_json["root"] = json!(root.to_string_lossy());
                    }
                }
                extra = f.notices;
            }
            JoinState::Broken(e) => {
                kb_json["reason"] = json!(redact_text(&e.message));
                problem = json!({"code": e.code.as_str(), "message": redact_text(&e.message), "next": e.next});
            }
            JoinState::Empty | JoinState::NotAKnowledgeBase => {
                if let Ok(Some(repo)) = ctx.repo().map(Some) {
                    kb_json["path"] = json!(repo.dir().display().to_string());
                }
                if let Err(e) = ctx.kb() {
                    problem =
                        json!({"code": e.code.as_str(), "message": e.message, "next": e.next});
                }
            }
            JoinState::Cloning => {}
        }

        let message = if healthy {
            "All checks are ok.".to_string()
        } else {
            format!("{} checks failed: {}.", failed.len(), failed.join(", "))
        };
        let repo = ctx.repo().ok();
        let last_pull = repo
            .as_ref()
            .and_then(|r| r.last_pull())
            .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
        let author = report.author.as_ref().map(|a| {
            json!({
                "name": a.name, "email": a.email,
                "name_source": a.name_source.name(), "email_source": a.email_source.name()
            })
        });
        let mut data = json!({
            "healthy": healthy,
            "version": env!("CARGO_PKG_VERSION"),
            "knowledge_base": kb_json,
            "checks": report.checks.iter().map(|c| c.to_json()).collect::<Vec<_>>(),
            "limit_sources": limit_sources(ctx, report.kb.as_ref()),
            "last_pull": last_pull,
            "last_error": ctx.last_session_error().map(|e| redact_text(&e)),
        });
        if let Some(a) = author {
            data["author"] = a;
        }
        if !problem.is_null() {
            data["problem"] = problem;
        }
        // The registry adds the paused notice and the notices of the settings file.
        let mut out = ToolOutput::new(message, data);
        for c in report.checks.iter().filter(|c| !c.ok) {
            out = out.notice(format!(
                "Check `{}` failed. {}",
                c.name,
                redact_text(c.fix.as_deref().unwrap_or(&c.detail))
            ));
        }
        for n in extra {
            if n != "Writes are paused." && !n.contains("unknown key") {
                out = out.notice(n);
            }
        }
        Ok(out)
    }
}
