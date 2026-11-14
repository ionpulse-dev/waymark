//! The `status` tool: the state of the setup (see `maintenance` and `session-start`).
//!
//! This is the first version. `maintenance` adds the remaining checks.

use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::cmd;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use serde_json::{json, Value};

pub struct Status;

fn check(name: &str, ok: bool, detail: impl Into<String>, fix: Option<String>) -> Value {
    let mut v = json!({ "name": name, "ok": ok, "detail": detail.into() });
    if let Some(f) = fix {
        v["fix"] = json!(f);
    }
    v
}

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

    fn call(&self, ctx: &Context, env: &Env, _args: Value) -> Result<ToolOutput, Error> {
        let mut checks = Vec::new();

        match cmd::check_git() {
            Ok(v) => checks.push(check("git", true, v, None)),
            Err(e) => checks.push(check("git", false, e.message, e.next)),
        }

        match &env.problem {
            None => checks.push(check(
                "settings",
                true,
                "The settings files are valid.",
                None,
            )),
            Some(p) => checks.push(check(
                "settings",
                false,
                format!("{}: {}", p.path.display(), p.message),
                Some("Fix or remove that file.".into()),
            )),
        }

        let mut kb_json = Value::Null;
        match ctx.repo() {
            Ok(repo) => {
                checks.push(check("clone", true, repo.dir().display().to_string(), None));
                match ctx.kb() {
                    Ok(kb) => {
                        checks.push(check(
                            "knowledge_base",
                            true,
                            kb.settings.name.clone(),
                            None,
                        ));
                        kb_json = json!({
                            "name": kb.settings.name,
                            "id": kb.settings.id,
                            "slug": kb.settings.slug,
                            "path": repo.dir().display().to_string(),
                            "verification": kb.settings.verification,
                        });
                    }
                    Err(e) => checks.push(check("knowledge_base", false, e.message, e.next)),
                }
            }
            Err(e) => checks.push(check("clone", false, e.message, e.next)),
        }

        let healthy = checks.iter().all(|c| c["ok"] == json!(true));
        let message = if healthy {
            "The setup is healthy."
        } else {
            "The setup has a problem. See the checks."
        };
        Ok(ToolOutput::new(
            message,
            json!({ "healthy": healthy, "knowledge_base": kb_json, "checks": checks }),
        ))
    }
}
