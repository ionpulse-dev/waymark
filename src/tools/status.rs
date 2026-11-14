//! The `status` tool: the state of the setup (see `maintenance` and `session-start`).
//!
//! This is the first version. `maintenance` adds the remaining checks.

use crate::config::Kind;
use crate::error::Error;
use crate::gitsync::cmd;
use crate::mcp::context::Context;
use crate::mcp::result::ToolOutput;
use crate::mcp::session;
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
        let mut extra: Vec<String> = Vec::new();

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
        let mut problem = Value::Null;
        let state = ctx.join_state();
        let cloning = matches!(state, crate::mcp::context::JoinState::Cloning);
        if cloning {
            checks.push(check(
                "clone",
                false,
                "The first clone is running.",
                Some("Call status again in a minute.".into()),
            ));
        }
        match if cloning {
            Err(Error::setup("The first clone is running."))
        } else {
            ctx.repo()
        } {
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
                        let f = session::facts(ctx, true);
                        if let Some(k) = &f.kb {
                            kb_json = k.to_json();
                            kb_json["verification"] = json!(kb.settings.verification);
                        }
                        extra = f.notices;
                    }
                    Err(e) => {
                        problem =
                            json!({"code": e.code.as_str(), "message": e.message, "next": e.next});
                        checks.push(check("knowledge_base", false, e.message, e.next))
                    }
                }
            }
            Err(e) => {
                if !cloning {
                    problem =
                        json!({"code": e.code.as_str(), "message": e.message, "next": e.next});
                    checks.push(check("clone", false, e.message, e.next))
                }
            }
        }
        if kb_json.is_null() {
            kb_json = json!({});
        }
        kb_json["state"] = json!(state.name());
        if let crate::mcp::context::JoinState::Broken(e) = &state {
            kb_json["reason"] = json!(crate::identity::redact_text(&e.message));
        }

        let mut author_json = Value::Null;
        if let Ok(repo) = ctx.repo() {
            match ctx.checked_author() {
                Ok(a) => {
                    author_json = json!({
                        "name": a.name, "email": a.email,
                        "name_source": a.name_source.name(), "email_source": a.email_source.name()
                    });
                    checks.push(check(
                        "author",
                        true,
                        format!("{} <{}>", a.name, a.email),
                        None,
                    ));
                }
                Err(e) => checks.push(check("author", false, e.message, e.next)),
            }
            checks.push(git_login(&repo));
        }

        let healthy = checks.iter().all(|c| c["ok"] == json!(true));
        let message = if cloning {
            "The first clone is still running.".to_string()
        } else if !problem.is_null() {
            "No knowledge base is set up.".to_string()
        } else if healthy {
            format!(
                "Knowledge base \"{}\" is in use. Access is {}.",
                kb_json["name"].as_str().unwrap_or(""),
                env.effective.to_json()["access"].as_str().unwrap_or("")
            )
        } else {
            "The setup has a problem. See the checks.".to_string()
        };
        let mut data = json!({ "healthy": healthy, "knowledge_base": kb_json, "checks": checks });
        if !author_json.is_null() {
            data["author"] = author_json;
        }
        if !problem.is_null() {
            data["problem"] = problem;
        }
        // The registry adds the paused notice and the notices of the settings file. Here come
        // the others, so that `status` shows the same notices as the instructions.
        let mut out = ToolOutput::new(message, data);
        for n in extra {
            if n != "Writes are paused." && !env_notice(&n) {
                out = out.notice(n);
            }
        }
        Ok(out)
    }
}

/// One read call to the remote, with a time-out of 10 seconds. It reads no secret.
fn git_login(repo: &crate::gitsync::Repo) -> Value {
    if !repo.has_remote() {
        return check("git_login", true, "There is no remote.", None);
    }
    let dir = repo.dir().to_path_buf();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(cmd::run(&dir, None, &["ls-remote", "origin", "HEAD"]));
    });
    match rx.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(Ok(o)) if o.ok => check("git_login", true, "The remote answers.", None),
        Ok(Ok(o)) if crate::identity::is_login_failure(&o.stderr) => check(
            "git_login",
            false,
            crate::identity::redact_text(&cmd::first_line(&o.stderr)),
            Some(crate::identity::LOGIN_NEXT.to_string()),
        ),
        Ok(Ok(o)) => check(
            "git_login",
            false,
            crate::identity::redact_text(&cmd::first_line(&o.stderr)),
            Some("Check the remote URL and your network.".to_string()),
        ),
        _ => check(
            "git_login",
            false,
            "The remote did not answer in 10 seconds.",
            Some("Check the remote URL and your network.".to_string()),
        ),
    }
}

/// Notices about the settings file already come from the registry.
fn env_notice(n: &str) -> bool {
    n.contains("unknown key")
}
