//! The `lint` tool: check documents and list findings.

use super::read_common::folders_arg;
use crate::config::Kind;
use crate::error::Error;
use crate::format::Severity;
use crate::lint;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::tool::{Env, Tool};
use crate::store::{Scope, Store};
use serde_json::{json, Value};

pub struct Lint;

const MAX_FINDINGS: usize = 100;

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}

impl Tool for Lint {
    fn name(&self) -> &'static str {
        "lint"
    }

    fn description(&self) -> &'static str {
        "Checks documents for format, size, links, secrets, personal data, duplicate titles, and authors. Returns checked, errors, warnings, and findings. Changes nothing."
    }

    fn kind(&self) -> Kind {
        Kind::Read
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "folders": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Only these folders. Default: all."
                }
            },
            "additionalProperties": false
        })
    }

    fn call(&self, ctx: &Context, env: &Env, args: Value) -> Result<ToolOutput, Error> {
        let repo = ctx.repo()?;
        let memrepo = ctx.memrepo()?;
        let notices = repo.refresh_if_stale();
        let store = Store::load(&memrepo);
        let scope = Scope::new(&env.effective, &folders_arg(&args));
        let report = lint::run(&memrepo, &store, &|id| scope.allows(id), Some(&repo))?;

        let errors = report
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();
        let warnings = report.findings.len() - errors;
        let truncated = report.findings.len() > MAX_FINDINGS;
        let findings: Vec<Value> = report
            .findings
            .iter()
            .take(MAX_FINDINGS)
            .map(|f| {
                json!({
                    "path": f.path,
                    "rule": f.rule,
                    "severity": if f.severity == Severity::Error { "error" } else { "warning" },
                    "message": f.message,
                })
            })
            .collect();
        let message = format!(
            "{} checked: {}, {}.",
            plural(report.checked, "document"),
            plural(errors, "error"),
            plural(warnings, "warning")
        );
        let mut out = ToolOutput::new(
            message,
            json!({
                "checked": report.checked,
                "errors": errors,
                "warnings": warnings,
                "findings": findings,
                "truncated": truncated,
            }),
        );
        for n in notices {
            out = out.notice(n);
        }
        Ok(out)
    }
}
